//! Worker ownership and non-blocking producer capability for wheel output.

use std::collections::HashSet;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_queue::ArrayQueue;
use openlogi_core::config::{
    AppSettings, CurvePreset, HorizontalScrollBaseline, VerticalScrollBaseline,
    VerticalScrollSensitivity,
};
use openlogi_core::math::ScrollLut;
use openlogi_core::scroll::ScrollDelta;
use tracing::warn;

use super::{FRAME_INTERVAL, ScrollEngine, ScrollFrame, ScrollSource, WheelDelta};
use crate::runtime::HidppSessionId;

/// OS-hook callbacks must fail open rather than wait for the worker.
const INPUT_QUEUE_CAPACITY: usize = 256;
/// Bounds graceful process shutdown if platform injection stops returning.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

/// Atomic snapshot of scroll preferences with precomputed LUT tables.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollPreferenceSnapshot {
    /// Vertical baseline distance.
    pub vertical_baseline: VerticalScrollBaseline,
    /// Horizontal baseline distance.
    pub horizontal_baseline: HorizontalScrollBaseline,
    /// Vertical curve preset.
    pub vertical_preset: CurvePreset,
    /// Horizontal curve preset.
    pub horizontal_preset: CurvePreset,
    /// Vertical multiplier.
    pub vertical_multiplier: f32,
    /// Horizontal multiplier.
    pub horizontal_multiplier: f32,
    /// Precomputed vertical lookup table.
    pub vertical_lut: Arc<ScrollLut>,
    /// Precomputed horizontal lookup table.
    pub horizontal_lut: Arc<ScrollLut>,
}

impl From<&AppSettings> for ScrollPreferenceSnapshot {
    fn from(settings: &AppSettings) -> Self {
        let vertical_preset = settings.effective_vertical_preset();
        let horizontal_preset = settings.effective_horizontal_preset();
        let vertical_multiplier = settings.effective_vertical_multiplier();
        let horizontal_multiplier = settings.effective_horizontal_multiplier();
        let vertical_lut = Arc::new(ScrollLut::generate(vertical_preset, vertical_multiplier));
        let horizontal_lut = Arc::new(ScrollLut::generate(
            horizontal_preset,
            horizontal_multiplier,
        ));
        Self {
            vertical_baseline: settings.vertical_scroll_baseline,
            horizontal_baseline: settings.horizontal_scroll_baseline,
            vertical_preset,
            horizontal_preset,
            vertical_multiplier,
            horizontal_multiplier,
            vertical_lut,
            horizontal_lut,
        }
    }
}

/// Atomically published settings read from input callbacks and the scroll
/// worker without taking the orchestrator's config lock.
pub struct ScrollPreferences {
    snapshot: std::sync::RwLock<ScrollPreferenceSnapshot>,
}

impl ScrollPreferences {
    /// Create live scroll settings from full app settings.
    #[must_use]
    pub fn from_app_settings(settings: &AppSettings) -> Self {
        Self {
            snapshot: std::sync::RwLock::new(ScrollPreferenceSnapshot::from(settings)),
        }
    }

    /// Create a live settings cell from validated config values.
    #[must_use]
    pub fn new(smooth_scroll: bool, vertical_sensitivity: VerticalScrollSensitivity) -> Self {
        let settings = AppSettings {
            vertical_curve_preset: if smooth_scroll {
                CurvePreset::Smooth
            } else {
                CurvePreset::None
            },
            vertical_scroll_baseline: vertical_sensitivity,
            ..AppSettings::default()
        };
        Self {
            snapshot: std::sync::RwLock::new(ScrollPreferenceSnapshot::from(&settings)),
        }
    }

    /// Publish updated settings snapshot from AppSettings.
    pub fn publish_app_settings(&self, settings: &AppSettings) {
        if let Ok(mut lock) = self.snapshot.write() {
            *lock = ScrollPreferenceSnapshot::from(settings);
        }
    }

    /// Publish smooth_scroll and vertical_sensitivity.
    pub fn publish(&self, smooth_scroll: bool, vertical_sensitivity: VerticalScrollSensitivity) {
        if let Ok(mut lock) = self.snapshot.write() {
            lock.vertical_preset = if smooth_scroll {
                CurvePreset::Smooth
            } else {
                CurvePreset::None
            };
            lock.vertical_baseline = vertical_sensitivity;
            lock.vertical_lut = Arc::new(ScrollLut::generate(
                lock.vertical_preset,
                lock.vertical_multiplier,
            ));
        }
    }

    /// Load a snapshot of current scroll settings. A poisoned lock still
    /// yields its last written value, mirroring `publish`'s fail-open policy.
    #[must_use]
    pub fn load(&self) -> ScrollPreferenceSnapshot {
        match self.snapshot.read() {
            Ok(lock) => lock.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Whether finite smooth scrolling is currently enabled.
    #[must_use]
    pub fn smooth_scroll_enabled(&self) -> bool {
        self.load().vertical_preset == CurvePreset::Smooth
    }

    /// The current traditional vertical wheel sensitivity.
    #[must_use]
    pub fn vertical_sensitivity(&self) -> VerticalScrollSensitivity {
        self.load().vertical_baseline
    }
}

#[derive(Clone, Copy)]
enum ScrollOutputMode {
    Smooth { at: Instant },
    Direct,
}

struct ScrollInput {
    generation: u64,
    source: ScrollSource,
    impulse: WheelDelta,
    output: ScrollOutputMode,
}

enum ScrollCommand {
    Input(ScrollInput),
    CancelSource(ScrollSource),
}

struct ShutdownRequest {
    done: mpsc::SyncSender<()>,
}

/// Lossless fallback for overflow cancellation and graceful shutdown.
enum ScrollControl {
    CancelOverflowSource(ScrollSource),
    Shutdown(ShutdownRequest),
}

/// Cloneable, non-owning capability for physical input producers.
///
/// Submission is always non-blocking. `false` asks each producer to use its
/// source-appropriate direct-output fallback.
#[derive(Clone)]
pub struct ScrollInputHandle {
    commands: Arc<ArrayQueue<ScrollCommand>>,
    controls: mpsc::Sender<ScrollControl>,
    worker_thread: thread::Thread,
    generation: Arc<AtomicU64>,
    accepting: Arc<AtomicBool>,
    preferences: Arc<ScrollPreferences>,
}

impl ScrollInputHandle {
    /// Queue one ordinary wheel impulse from the current OS-hook thread.
    ///
    /// Pixel input, zero/non-finite distance, a full queue, or an unavailable
    /// worker are rejected so the callback fails open.
    #[must_use]
    pub fn try_hook_scroll(&self, delta: ScrollDelta) -> bool {
        if !self.accepting.load(Ordering::Acquire) {
            return false;
        }
        let Ok(impulse) = WheelDelta::try_from(delta) else {
            return false;
        };
        let preferences = self.preferences.load();
        let vertical_scaled = impulse.y != 0.0
            && (preferences.vertical_baseline != VerticalScrollBaseline::DEFAULT
                || preferences.vertical_preset.is_enabled());
        let horizontal_scaled = impulse.x != 0.0
            && (preferences.horizontal_baseline != HorizontalScrollBaseline::DEFAULT
                || preferences.horizontal_preset.is_enabled());

        let output = if preferences.vertical_preset == CurvePreset::Smooth {
            ScrollOutputMode::Smooth { at: Instant::now() }
        } else if vertical_scaled || horizontal_scaled {
            ScrollOutputMode::Direct
        } else {
            return false;
        };
        self.try_enqueue(ScrollSource::current_hook(), impulse, output)
    }

    /// Queue one diverted thumb-wheel impulse from an active HID++ session.
    ///
    /// Rejection tells the already-diverted caller to inject the distance
    /// directly; unlike an OS hook, there is no physical event to pass through.
    #[must_use]
    pub(crate) fn try_hidpp_scroll(&self, session: &HidppSessionId, delta: ScrollDelta) -> bool {
        if !self.accepting.load(Ordering::Acquire) || !self.preferences.smooth_scroll_enabled() {
            return false;
        }
        let Ok(impulse) = WheelDelta::try_from(delta) else {
            return false;
        };
        self.try_enqueue(
            ScrollSource::Hidpp(session.clone()),
            impulse,
            ScrollOutputMode::Smooth { at: Instant::now() },
        )
    }

    fn try_enqueue(
        &self,
        source: ScrollSource,
        impulse: WheelDelta,
        output: ScrollOutputMode,
    ) -> bool {
        let input = ScrollInput {
            generation: self.generation.load(Ordering::Acquire),
            source,
            impulse,
            output,
        };
        if self.commands.push(ScrollCommand::Input(input)).is_ok() {
            self.wake();
            true
        } else {
            warn!("scroll output queue full — input rejected");
            false
        }
    }

    /// Invalidate every accepted OS-hook animation without blocking.
    pub fn cancel_hooks(&self) {
        self.cancel_all();
    }

    /// Cancel output belonging to one HID++ capture-session incarnation.
    pub(crate) fn cancel_hidpp_session(&self, session: &HidppSessionId) {
        let source = ScrollSource::Hidpp(session.clone());
        let pushed_ring = self
            .commands
            .push(ScrollCommand::CancelSource(source.clone()))
            .is_ok();
        let sent_control = (!pushed_ring)
            && self
                .controls
                .send(ScrollControl::CancelOverflowSource(source))
                .is_ok();
        if pushed_ring || sent_control {
            self.wake();
        }
    }

    fn cancel_all(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.wake();
    }

    fn wake(&self) {
        self.worker_thread.unpark();
    }

    fn stop_accepting(&self) {
        self.accepting.store(false, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}

/// Unique owner of the scroll output worker and its graceful shutdown.
pub struct ScrollRuntime {
    input: ScrollInputHandle,
    controls: mpsc::Sender<ScrollControl>,
    worker: Option<JoinHandle<()>>,
}

impl ScrollRuntime {
    /// Start the dedicated output worker using the live scroll settings.
    pub fn spawn(preferences: Arc<ScrollPreferences>) -> io::Result<Self> {
        Self::spawn_with(preferences, ScrollFrame::post, WheelDelta::post)
    }

    pub(super) fn spawn_with(
        preferences: Arc<ScrollPreferences>,
        mut emit_smooth: impl FnMut(ScrollFrame) + Send + 'static,
        mut emit_direct: impl FnMut(WheelDelta) + Send + 'static,
    ) -> io::Result<Self> {
        let commands = Arc::new(ArrayQueue::new(INPUT_QUEUE_CAPACITY));
        let (controls, control_rx) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let worker_commands = Arc::clone(&commands);
        let worker_generation = Arc::clone(&generation);
        let worker_preferences = Arc::clone(&preferences);
        let worker = thread::Builder::new()
            .name("openlogi-scroll".into())
            .spawn(move || {
                let current_thread = thread::current();
                run_worker(
                    &worker_commands,
                    &control_rx,
                    &worker_generation,
                    &worker_preferences,
                    current_thread,
                    &mut emit_smooth,
                    &mut emit_direct,
                );
            })?;
        let input = ScrollInputHandle {
            commands: Arc::clone(&commands),
            controls: controls.clone(),
            worker_thread: worker.thread().clone(),
            generation: Arc::clone(&generation),
            accepting: Arc::new(AtomicBool::new(true)),
            preferences,
        };
        Ok(Self {
            input,
            controls,
            worker: Some(worker),
        })
    }

    /// Clone the non-owning input capability for physical input producers.
    #[must_use]
    pub fn input(&self) -> ScrollInputHandle {
        self.input.clone()
    }

    /// Reject new input, cancel active output, and join the worker.
    pub fn shutdown(&mut self) {
        let _ = self.shutdown_with_timeout(SHUTDOWN_TIMEOUT);
    }

    fn shutdown_with_timeout(&mut self, timeout: Duration) -> bool {
        let Some(worker) = self.worker.take() else {
            return true;
        };
        self.input.stop_accepting();
        let (done, wait) = mpsc::sync_channel(0);
        if self
            .controls
            .send(ScrollControl::Shutdown(ShutdownRequest { done }))
            .is_err()
        {
            let _ = worker.join();
            return false;
        }
        self.input.wake();
        if wait.recv_timeout(timeout).is_err() {
            warn!("smooth-scroll worker did not shut down before the deadline");
            return false;
        }
        if worker.join().is_err() {
            warn!("smooth-scroll worker panicked during shutdown");
            return false;
        }
        true
    }
}

impl Drop for ScrollRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run_worker(
    commands: &ArrayQueue<ScrollCommand>,
    controls: &mpsc::Receiver<ScrollControl>,
    shared_generation: &AtomicU64,
    preferences: &ScrollPreferences,
    worker_thread: thread::Thread,
    emit_smooth: &mut impl FnMut(ScrollFrame),
    emit_direct: &mut impl FnMut(WheelDelta),
) {
    let mut engine = ScrollEngine::default();
    let mut accel_engine = super::ScrollAccelerationEngine::default();
    let mut display_sync = super::display_link::DisplaySync::new(worker_thread);
    let mut overflow_cancelled_sources = HashSet::new();
    let mut generation = shared_generation.load(Ordering::Acquire);
    loop {
        while let Ok(control) = controls.try_recv() {
            match control {
                ScrollControl::CancelOverflowSource(source) => {
                    engine.cancel_source(&source, emit_smooth);
                    overflow_cancelled_sources.insert(source);
                }
                ScrollControl::Shutdown(request) => {
                    display_sync.stop();
                    engine.cancel_all(emit_smooth);
                    let _ = request.done.send(());
                    return;
                }
            }
        }

        let current_generation = shared_generation.load(Ordering::Acquire);
        if current_generation != generation || !preferences.smooth_scroll_enabled() {
            engine.cancel_all(emit_smooth);
            generation = current_generation;
        }

        let mut had_command = false;
        while let Some(command) = commands.pop() {
            had_command = true;
            match command {
                ScrollCommand::Input(input)
                    if input.generation == generation
                        && !overflow_cancelled_sources.contains(&input.source) =>
                {
                    let prefs = preferences.load();
                    let at = match input.output {
                        ScrollOutputMode::Smooth { at } => at,
                        ScrollOutputMode::Direct => Instant::now(),
                    };
                    let (h_gain_fixed, v_gain_fixed) = accel_engine.compute_gains_fixed(
                        input.impulse.x,
                        input.impulse.y,
                        at,
                        prefs.vertical_preset,
                        &prefs.vertical_lut,
                        prefs.horizontal_preset,
                        &prefs.horizontal_lut,
                    );
                    let gain_x = openlogi_core::math::fixed_to_f32(h_gain_fixed);
                    let gain_y = openlogi_core::math::fixed_to_f32(v_gain_fixed);
                    let scaled_impulse = WheelDelta {
                        x: input.impulse.x * prefs.horizontal_baseline.scroll_multiplier() * gain_x,
                        y: input.impulse.y * prefs.vertical_baseline.scroll_multiplier() * gain_y,
                    };

                    if scaled_impulse.x.is_finite()
                        && scaled_impulse.y.is_finite()
                        && !scaled_impulse.is_zero()
                    {
                        match input.output {
                            ScrollOutputMode::Smooth { .. }
                                if prefs.vertical_preset == CurvePreset::Smooth =>
                            {
                                engine.impulse(input.source, scaled_impulse, at, emit_smooth);
                            }
                            ScrollOutputMode::Direct | ScrollOutputMode::Smooth { .. } => {
                                engine.cancel_source(&input.source, emit_smooth);
                                emit_direct(scaled_impulse);
                            }
                        }
                    }
                }
                ScrollCommand::CancelSource(source) => {
                    engine.cancel_source(&source, emit_smooth);
                }
                ScrollCommand::Input(_) => {}
            }
        }

        // Advance active motions
        let now = Instant::now();
        engine.advance_due(now, emit_smooth);

        // Adaptive display synchronization and parking:
        if let Some(deadline) = engine.next_deadline() {
            display_sync.start();
            let timeout = deadline.saturating_duration_since(Instant::now());
            if !timeout.is_zero() {
                thread::park_timeout(timeout.min(FRAME_INTERVAL));
            }
        } else {
            display_sync.stop();
            if !had_command {
                thread::park();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sensitivity(raw: u8) -> VerticalScrollSensitivity {
        VerticalScrollSensitivity::try_new(raw).expect("test sensitivity is valid")
    }

    fn preferences(smooth_scroll: bool, sensitivity: u8) -> Arc<ScrollPreferences> {
        let mut settings = openlogi_core::config::AppSettings {
            vertical_scroll_baseline: self::sensitivity(sensitivity),
            ..Default::default()
        };
        settings.vertical_curve_preset = if smooth_scroll {
            CurvePreset::Smooth
        } else {
            CurvePreset::None
        };
        settings.horizontal_curve_preset = CurvePreset::None;
        Arc::new(ScrollPreferences::from_app_settings(&settings))
    }

    fn standalone_input(
        capacity: usize,
        preferences: Arc<ScrollPreferences>,
    ) -> (
        ScrollInputHandle,
        Arc<ArrayQueue<ScrollCommand>>,
        mpsc::Receiver<ScrollControl>,
    ) {
        let commands = Arc::new(ArrayQueue::new(capacity));
        let (controls, control_rx) = mpsc::channel();
        (
            ScrollInputHandle {
                commands: Arc::clone(&commands),
                controls,
                worker_thread: thread::current(),
                generation: Arc::new(AtomicU64::new(0)),
                accepting: Arc::new(AtomicBool::new(true)),
                preferences,
            },
            commands,
            control_rx,
        )
    }

    fn queued_input(commands: &ArrayQueue<ScrollCommand>) -> ScrollInput {
        let Some(ScrollCommand::Input(input)) = commands.pop() else {
            panic!("expected scroll input in queue");
        };
        input
    }

    #[test]
    fn callback_submission_rejects_ineligible_input_and_fails_open_when_full() {
        let (disabled, _commands, _controls) = standalone_input(
            1,
            preferences(false, u8::from(VerticalScrollSensitivity::DEFAULT)),
        );
        assert!(!disabled.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 1.0)));

        let (input, _commands, _controls) = standalone_input(
            1,
            preferences(true, u8::from(VerticalScrollSensitivity::DEFAULT)),
        );
        assert!(!input.try_hook_scroll(ScrollDelta::pixels(0.0, 1.0)));
        assert!(input.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 1.0)));
        assert!(!input.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 1.0)));
    }

    #[test]
    fn hook_scales_only_vertical_wheel_distance_and_selects_direct_output() {
        let (input, receiver, _controls) = standalone_input(2, preferences(false, 7));
        assert!(!input.try_hook_scroll(ScrollDelta::wheel_ticks(2.0, 0.0)));
        assert!(input.try_hook_scroll(ScrollDelta::wheel_ticks(2.0, 2.0)));

        let queued = queued_input(&receiver);
        assert_eq!(queued.impulse, WheelDelta { x: 2.0, y: 2.0 });
        assert!(matches!(queued.output, ScrollOutputMode::Direct));
    }

    #[test]
    fn hook_scales_vertical_distance_before_smoothing() {
        let (input, receiver, _controls) = standalone_input(1, preferences(true, 7));
        assert!(input.try_hook_scroll(ScrollDelta::wheel_ticks(2.0, 2.0)));

        let queued = queued_input(&receiver);
        assert_eq!(queued.impulse, WheelDelta { x: 2.0, y: 2.0 });
        assert!(matches!(queued.output, ScrollOutputMode::Smooth { .. }));
    }

    #[test]
    fn hidpp_smoothing_does_not_apply_main_wheel_sensitivity() {
        let (input, receiver, _controls) = standalone_input(1, preferences(true, 7));
        let session = HidppSessionId::new("mouse-a", 7);
        assert!(input.try_hidpp_scroll(&session, ScrollDelta::wheel_ticks(0.0, 2.0)));

        let queued = queued_input(&receiver);
        assert_eq!(queued.impulse, WheelDelta { x: 0.0, y: 2.0 });
        assert!(matches!(queued.output, ScrollOutputMode::Smooth { .. }));
    }

    #[test]
    fn live_preferences_change_hook_admission_and_output_mode() {
        let preferences = preferences(false, u8::from(VerticalScrollSensitivity::DEFAULT));
        let (input, receiver, _controls) = standalone_input(2, Arc::clone(&preferences));
        assert!(!input.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 1.0)));

        preferences.publish(false, sensitivity(7));
        assert!(input.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 2.0)));
        assert!(matches!(
            queued_input(&receiver).output,
            ScrollOutputMode::Direct
        ));

        preferences.publish(true, sensitivity(28));
        assert!(input.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 1.0)));
        let queued = queued_input(&receiver);
        assert_eq!(queued.impulse, WheelDelta { x: 0.0, y: 1.0 });
        assert!(matches!(queued.output, ScrollOutputMode::Smooth { .. }));
    }

    #[test]
    fn hidpp_cancellation_targets_its_session() {
        let (input, commands, _controls) = standalone_input(1, preferences(true, 14));
        let session = HidppSessionId::new("mouse-a", 7);
        input.cancel_hidpp_session(&session);

        let Some(ScrollCommand::CancelSource(ScrollSource::Hidpp(cancelled))) = commands.pop()
        else {
            panic!("expected HID++ source cancellation");
        };
        assert_eq!(cancelled, session);
        assert_eq!(input.generation.load(Ordering::Acquire), 0);
    }

    #[test]
    fn cancellation_overtakes_a_full_queue_without_discarding_another_source() {
        let preferences = preferences(true, 14);
        let (mut input, commands, controls) = standalone_input(2, Arc::clone(&preferences));
        let cancelled = HidppSessionId::new("mouse-a", 7);
        let survivor = HidppSessionId::new("mouse-b", 3);
        assert!(input.try_hidpp_scroll(&cancelled, ScrollDelta::wheel_ticks(1.0, 0.0)));
        assert!(input.try_hidpp_scroll(&survivor, ScrollDelta::wheel_ticks(0.0, 1.0)));

        input.cancel_hidpp_session(&cancelled);
        assert_eq!(
            input.generation.load(Ordering::Acquire),
            0,
            "source-local cancellation must not invalidate unrelated accepted input"
        );

        let generation = Arc::clone(&input.generation);
        let (emitted, frames) = mpsc::channel();
        let commands_ref = Arc::clone(&commands);
        let worker_prefs = Arc::clone(&preferences);
        let worker = thread::spawn(move || {
            let current_thread = thread::current();
            run_worker(
                &commands_ref,
                &controls,
                &generation,
                &worker_prefs,
                current_thread,
                &mut |frame| {
                    emitted.send(frame).expect("frame receiver remains open");
                },
                &mut |_| {},
            );
        });
        input.worker_thread = worker.thread().clone();

        let mut output = Vec::new();
        loop {
            let frame = frames
                .recv_timeout(Duration::from_secs(1))
                .expect("surviving source completes");
            output.push(frame);
            if frame.phase == openlogi_inject::SmoothScrollPhase::Ended {
                break;
            }
        }

        let (done, wait) = mpsc::sync_channel(0);
        input
            .controls
            .send(ScrollControl::Shutdown(ShutdownRequest { done }))
            .expect("worker control channel remains open");
        input.wake();
        wait.recv_timeout(Duration::from_secs(1))
            .expect("worker acknowledges shutdown");
        worker.join().expect("worker exits cleanly");

        let total = output
            .iter()
            .fold(WheelDelta::ZERO, |sum, frame| sum.plus(frame.delta));
        assert!(total.x.abs() < f32::EPSILON, "cancelled source emitted");
        assert!((total.y - 1.0).abs() < f32::EPSILON);
        assert!(
            output
                .iter()
                .all(|frame| frame.phase != openlogi_inject::SmoothScrollPhase::Cancelled)
        );
    }

    #[test]
    fn idle_worker_shutdown_wakes_after_publishing_its_request() {
        let mut runtime = ScrollRuntime::spawn_with(preferences(false, 14), |_| {}, |_| {})
            .expect("spawn scroll worker");
        assert!(runtime.shutdown_with_timeout(Duration::from_millis(100)));
    }

    #[test]
    fn direct_scaled_output_is_emitted_by_the_worker() {
        let (outputs, received) = mpsc::channel();
        let smooth_outputs = outputs.clone();
        let mut runtime = ScrollRuntime::spawn_with(
            preferences(false, 7),
            move |frame| {
                smooth_outputs
                    .send(Err(frame))
                    .expect("test output receiver remains open");
            },
            move |delta| {
                outputs
                    .send(Ok(delta))
                    .expect("test output receiver remains open");
            },
        )
        .expect("spawn scroll worker");
        assert!(
            runtime
                .input()
                .try_hook_scroll(ScrollDelta::wheel_ticks(3.0, 2.0))
        );

        assert_eq!(
            received
                .recv_timeout(Duration::from_secs(1))
                .expect("direct worker output")
                .expect("output must be direct"),
            WheelDelta { x: 3.0, y: 1.0 }
        );
        runtime.shutdown();
    }

    #[test]
    fn generation_invalidation_cancels_started_output_out_of_band() {
        let (frames, received) = mpsc::channel();
        let mut runtime = ScrollRuntime::spawn_with(
            preferences(true, 14),
            move |frame| {
                frames
                    .send(frame)
                    .expect("test frame receiver remains open");
            },
            |_| {},
        )
        .expect("spawn scroll worker");
        let input = runtime.input();
        assert!(input.try_hook_scroll(ScrollDelta::wheel_ticks(0.0, 1.0)));
        assert_eq!(
            received
                .recv_timeout(Duration::from_secs(1))
                .expect("first animation frame")
                .phase,
            openlogi_inject::SmoothScrollPhase::Began
        );

        input.cancel_hooks();
        loop {
            let phase = received
                .recv_timeout(Duration::from_secs(1))
                .expect("cancellation frame")
                .phase;
            if phase == openlogi_inject::SmoothScrollPhase::Cancelled {
                break;
            }
            assert_eq!(phase, openlogi_inject::SmoothScrollPhase::Changed);
        }
        runtime.shutdown();
    }
}
