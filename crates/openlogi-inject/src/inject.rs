//! OS input-event synthesis for each [`Action`], split out of openlogi-core so
//! the core schema stays platform- and IO-free.
//!
//! [`execute`] is the single entry point: it dispatches to the per-platform
//! synthesiser (`macos::execute` / `linux::execute` / `windows::execute`), each
//! of which translates an [`Action`] into the native event(s) — CGEvent/NSEvent
//! on macOS, uinput/D-Bus on Linux, SendInput on Windows.

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::collections::HashMap;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::sync::{LazyLock, Mutex, PoisonError};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use openlogi_core::binding::KeyboardUsage;
use openlogi_core::binding::{Action, KeyCombo};
use openlogi_core::scroll::ScrollDelta;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

/// Which isolated edge of a held keyboard chord to synthesize.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyPhase {
    Down,
    Up,
}

/// One physical keyboard output shared by held chords.
///
/// Logical Cmd and Ctrl are distinct on macOS. Cmd aliases Ctrl on Linux and
/// Windows, so ownership is counted after that platform mapping is resolved.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum HeldKey {
    #[cfg(target_os = "macos")]
    Command,
    Control,
    Shift,
    Alt,
    Meta,
    #[cfg(target_os = "macos")]
    Fn,
    Key(KeyboardUsage),
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[derive(Debug, Default, PartialEq, Eq)]
struct HoldTransition {
    up: Vec<HeldKey>,
    down: Vec<HeldKey>,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct HeldModifiers(u8);

#[cfg(target_os = "macos")]
impl HeldModifiers {
    fn set(&mut self, key: HeldKey, held: bool) {
        let Some(mask) = Self::mask(key) else {
            return;
        };
        if held {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }

    fn contains(self, key: HeldKey) -> bool {
        Self::mask(key).is_some_and(|mask| self.0 & mask != 0)
    }

    fn mask(key: HeldKey) -> Option<u8> {
        match key {
            HeldKey::Command => Some(1 << 0),
            HeldKey::Control => Some(1 << 1),
            HeldKey::Shift => Some(1 << 2),
            HeldKey::Alt => Some(1 << 3),
            HeldKey::Meta => Some(1 << 4),
            HeldKey::Fn => Some(1 << 5),
            HeldKey::Key(_) => None,
        }
    }
}

/// Reference counts for physical keyboard outputs across active chords.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[derive(Default)]
struct HeldOutput {
    owners: HashMap<HeldKey, usize>,
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
impl HeldOutput {
    fn transition(
        &mut self,
        released: Option<&KeyCombo>,
        pressed: Option<&KeyCombo>,
    ) -> HoldTransition {
        let released = released.map_or_else(Vec::new, held_keys);
        let pressed = pressed.map_or_else(Vec::new, held_keys);
        let before = self.owners.clone();

        for key in &released {
            match self.owners.get_mut(key) {
                Some(owners) if *owners > 1 => *owners -= 1,
                Some(_) => {
                    self.owners.remove(key);
                }
                None => {}
            }
        }
        for key in &pressed {
            *self.owners.entry(*key).or_default() += 1;
        }

        HoldTransition {
            up: released
                .into_iter()
                .filter(|key| before.contains_key(key) && !self.owners.contains_key(key))
                .collect(),
            down: pressed
                .into_iter()
                .filter(|key| !before.contains_key(key) && self.owners.contains_key(key))
                .collect(),
        }
    }

    #[cfg(target_os = "macos")]
    fn modifiers(&self) -> HeldModifiers {
        let mut modifiers = HeldModifiers::default();
        for key in [
            HeldKey::Command,
            HeldKey::Control,
            HeldKey::Shift,
            HeldKey::Alt,
            HeldKey::Meta,
            HeldKey::Fn,
        ] {
            modifiers.set(key, self.owners.contains_key(&key));
        }
        modifiers
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
static HELD_OUTPUT: LazyLock<Mutex<HeldOutput>> =
    LazyLock::new(|| Mutex::new(HeldOutput::default()));

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn held_keys(combo: &KeyCombo) -> Vec<HeldKey> {
    let mut keys = Vec::with_capacity(4);
    #[cfg(target_os = "macos")]
    if combo.has_command() {
        keys.push(HeldKey::Command);
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if combo.has_command() || combo.has_control() {
        keys.push(HeldKey::Control);
    }
    #[cfg(target_os = "macos")]
    if combo.has_control() {
        keys.push(HeldKey::Control);
    }
    if combo.has_shift() {
        keys.push(HeldKey::Shift);
    }
    if combo.has_option() {
        keys.push(HeldKey::Alt);
    }
    if combo.has_meta() {
        keys.push(HeldKey::Meta);
    }
    #[cfg(target_os = "macos")]
    if combo.has_fn() {
        keys.push(HeldKey::Fn);
    }
    keys.push(HeldKey::Key(combo.key()));
    keys
}

/// Synthesise the OS-level event for `action`.
///
/// On macOS, key events are posted via `CGEventPost(kCGHIDEventTap, …)`
/// using virtual key codes from the standard US keyboard layout, and the
/// `LeftClick`/`RightClick`/`MiddleClick` variants synthesise a mouse click
/// at the current cursor location. The WindowServer actions (`MissionControl`,
/// `AppExpose`, `ShowDesktop`, `LaunchpadShow`) are posted straight to the
/// Dock via `CoreDockSendNotification`. Device-side actions (`CycleDpiPresets`,
/// `SetDpiPreset`, `ToggleSmartShift`) have no CGEvent equivalent and are
/// handled at the hook/HID layer, logging a trace here.
///
/// On Linux, key and scroll events are injected via a lazily-created `uinput`
/// virtual device. Mouse clicks inject `BTN_*` events. macOS-only window
/// manager actions (`MissionControl`, `AppExpose`, `ShowDesktop`,
/// `LaunchpadShow`) have no universal Linux equivalent and are silently
/// skipped (debug-logged). `CustomShortcut` maps macOS `kVK_*` codes to
/// Linux key codes; macOS Cmd maps to Ctrl.
///
/// On Windows, key and mouse events are synthesised via `SendInput`. The
/// macOS window-manager actions map to their Windows equivalents (e.g.
/// `MissionControl` → Win+Tab, `ShowDesktop` → Win+D); `CustomShortcut`
/// maps macOS `kVK_*` codes to Windows virtual-key codes, with Cmd mapped to
/// Ctrl.
///
/// On other platforms a warning is logged and the function returns
/// immediately — the binary compiles clean on all targets.
///
/// # Manual verification
///
/// `execute` is intentionally excluded from the automated test suite because
/// it would need to intercept the OS event queue. Smoke-test it manually:
/// bind a button to any action in the GUI and confirm the expected system event
/// fires when the button is pressed (or use the `inject_action` example).
fn launch_application(target: &openlogi_core::binding::ApplicationTarget) {
    let raw_path = target.path().trim();
    if raw_path.is_empty() {
        tracing::warn!("cannot launch empty application path");
        return;
    }

    let expanded = shellexpand::tilde(raw_path);
    let path = expanded.as_ref();

    #[cfg(target_os = "macos")]
    {
        let is_app_bundle = std::path::Path::new(path.trim_end_matches('/'))
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("app"));
        let mut cmd = std::process::Command::new("/usr/bin/open");
        if is_app_bundle {
            cmd.arg("-a").arg(path);
        } else {
            cmd.arg(path);
        }
        match cmd.spawn() {
            Ok(mut child) => {
                let _ = std::thread::spawn(move || {
                    let _ = child.wait();
                });
            }
            Err(error) => {
                tracing::warn!(
                    %error,
                    path,
                    "could not open configured application, folder, or URL via /usr/bin/open"
                );
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        // `xdg-open`/`gio` refuse (or "open in text editor") bare executables,
        // which is exactly what AppImages, ELF binaries, and scripts picked via
        // Browse are — run those directly. Folders, `.desktop` entries, and
        // URLs still go through the desktop opener.
        if is_executable_file(std::path::Path::new(path)) {
            match std::process::Command::new(path).spawn() {
                Ok(mut child) => {
                    let _ = std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                }
                Err(error) => {
                    tracing::warn!(
                        %error,
                        path,
                        "could not execute application file directly"
                    );
                }
            }
        } else if let Err(error) = opener::open(path) {
            tracing::warn!(
                %error,
                path,
                "could not open configured application, folder, or URL"
            );
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        if let Err(error) = opener::open(path) {
            tracing::warn!(
                %error,
                path,
                "could not open configured application, folder, or URL"
            );
        }
    }
}

/// A regular file carrying an execute bit: an AppImage, ELF binary, or
/// script that must be spawned directly instead of handed to the desktop
/// opener.
#[cfg(target_os = "linux")]
fn is_executable_file(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

pub fn execute(action: &Action) {
    if let Some(target) = action.application_target() {
        launch_application(target);
        return;
    }

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            macos::execute(action);
        } else if #[cfg(target_os = "linux")] {
            linux::execute(action);
        } else if #[cfg(target_os = "windows")] {
            windows::execute(action);
        } else {
            tracing::warn!(
                action = action.label(),
                "execute unsupported on this platform"
            );
        }
    }
}

/// Synthesise the down edge of `combo`, leaving its output held.
///
/// Every successful lifecycle start must be paired with [`release_hold`],
/// including cancellation and shutdown paths. Prefer [`execute`] when the
/// caller does not own a matching terminal event.
pub fn press_hold(combo: &KeyCombo) {
    hold_transition(None, Some(combo));
}

/// Synthesise the up edge matching a prior [`press_hold`].
///
/// Each successful [`press_hold`] must be released exactly once. The lifecycle
/// owner provides that guarantee; duplicate releases would consume ownership
/// retained for an overlapping chord.
pub fn release_hold(combo: &KeyCombo) {
    hold_transition(Some(combo), None);
}

/// Replace one held chord without releasing physical keys shared by both.
pub fn replace_hold(old: &KeyCombo, new: &KeyCombo) {
    hold_transition(Some(old), Some(new));
}

fn hold_transition(released: Option<&KeyCombo>, pressed: Option<&KeyCombo>) {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let mut output = HELD_OUTPUT.lock().unwrap_or_else(PoisonError::into_inner);
            let modifiers = output.modifiers();
            let transition = output.transition(released, pressed);
            let modifiers = macos::hold_keys(&transition.up, KeyPhase::Up, modifiers);
            let modifiers = macos::hold_keys(&transition.down, KeyPhase::Down, modifiers);
            debug_assert_eq!(modifiers, output.modifiers());
        } else if #[cfg(target_os = "linux")] {
            let mut output = HELD_OUTPUT.lock().unwrap_or_else(PoisonError::into_inner);
            let transition = output.transition(released, pressed);
            linux::hold_keys(&transition.up, KeyPhase::Up);
            linux::hold_keys(&transition.down, KeyPhase::Down);
        } else if #[cfg(target_os = "windows")] {
            let mut output = HELD_OUTPUT.lock().unwrap_or_else(PoisonError::into_inner);
            let transition = output.transition(released, pressed);
            windows::hold_keys(&transition.up, KeyPhase::Up);
            windows::hold_keys(&transition.down, KeyPhase::Down);
        } else {
            tracing::warn!(
                "held shortcut output unsupported on this platform"
            );
        }
    }
}

/// Navigate the browser identified by `pid` backwards or forwards using the
/// Accessibility API (`AXPress` on the "Go back" / "Go forward" toolbar button).
///
/// Call this from the gesture watcher **at the moment the button press arrives**
/// so `pid` reflects the correct frontmost app rather than whatever happens to
/// be frontmost when the async dispatch completes. Returns `true` on success.
/// No-op (returns `false`) on non-macOS platforms.
#[must_use]
pub fn ax_navigate_browser(pid: i32, forward: bool) -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::ax_browser_navigate(forward, Some(pid))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (pid, forward);
        false
    }
}

/// Integer scroll units ready for a platform API.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct QuantizedScroll {
    x: i32,
    y: i32,
}

use std::time::{Duration, Instant};

const IDLE_TIMEOUT: Duration = Duration::from_millis(250);

/// Carries fractional platform units across frames so rounding never changes
/// the cumulative distance.
#[derive(Default)]
struct ScrollQuantizer {
    residual_x: f64,
    residual_y: f64,
    last_dir_x: f64,
    last_dir_y: f64,
    last_update: Option<Instant>,
}

impl ScrollQuantizer {
    fn quantize(&mut self, delta: ScrollDelta, units_per_input: f64) -> QuantizedScroll {
        self.quantize_at(delta, units_per_input, Instant::now())
    }

    #[expect(
        clippy::float_cmp,
        reason = "signum produces exact -1.0 or 1.0 sentinels and 0.0 is the exact neutral state"
    )]
    fn quantize_at(
        &mut self,
        delta: ScrollDelta,
        units_per_input: f64,
        at: Instant,
    ) -> QuantizedScroll {
        if let Some(last_time) = self.last_update
            && at.saturating_duration_since(last_time) >= IDLE_TIMEOUT
        {
            self.residual_x = 0.0;
            self.residual_y = 0.0;
            self.last_dir_x = 0.0;
            self.last_dir_y = 0.0;
        }

        let input_x = delta.x();
        let input_y = delta.y();

        // Axis Switch: if input is strictly horizontal or vertical, clear the other axis remainder.
        if input_x != 0.0 && input_y == 0.0 {
            self.residual_y = 0.0;
            self.last_dir_y = 0.0;
        } else if input_y != 0.0 && input_x == 0.0 {
            self.residual_x = 0.0;
            self.last_dir_x = 0.0;
        }

        // Direction Reversal on X
        if input_x != 0.0 {
            let dir_x = input_x.signum();
            if self.last_dir_x != 0.0 && dir_x != self.last_dir_x {
                self.residual_x = 0.0;
            }
            self.last_dir_x = dir_x;
        }

        // Direction Reversal on Y
        if input_y != 0.0 {
            let dir_y = input_y.signum();
            if self.last_dir_y != 0.0 && dir_y != self.last_dir_y {
                self.residual_y = 0.0;
            }
            self.last_dir_y = dir_y;
        }

        self.last_update = Some(at);

        QuantizedScroll {
            x: quantize_axis(&mut self.residual_x, input_x, units_per_input),
            y: quantize_axis(&mut self.residual_y, input_y, units_per_input),
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the rounded value is clamped to the i32 range before conversion"
)]
fn quantize_axis(residual: &mut f64, input: f64, units_per_input: f64) -> i32 {
    let exact = input.mul_add(units_per_input, *residual);
    let rounded = exact
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX));
    let output = rounded as i32;
    *residual = exact - f64::from(output);
    output
}

/// Synthesise a typed scroll distance at the current focus.
///
/// Fractional wheel ticks are retained until the platform can represent them,
/// so a sequence of high-resolution frames preserves its cumulative distance.
/// Non-finite input is rejected at this I/O boundary.
pub fn post_scroll(delta: ScrollDelta) {
    if !delta.is_finite() || (delta.x() == 0.0 && delta.y() == 0.0) {
        return;
    }
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            macos::post_scroll(delta);
        } else if #[cfg(target_os = "linux")] {
            linux::post_scroll(delta);
        } else if #[cfg(target_os = "windows")] {
            windows::post_scroll(delta);
        } else {
            let _ = delta;
        }
    }
}

/// Lifecycle phase of one synthetic smooth-scroll frame.
///
/// macOS forwards this state to the scroll-wheel event so applications see a
/// balanced continuous gesture. Linux and Windows have no equivalent field;
/// there the phase is retained by the runtime contract but only the frame's
/// distance is injected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmoothScrollPhase {
    /// First output frame of a new animation.
    Began,
    /// An intermediate output frame, including frames after retargeting.
    Changed,
    /// Final frame, carrying any correction needed to reach the exact target.
    Ended,
    /// The capture source ended before the animation reached its target.
    Cancelled,
}

/// Synthesise one frame of a finite smooth-scroll animation.
///
/// On macOS wheel ticks become continuous pixel events at ten points per tick,
/// matching the line/point relationship carried in native continuous events.
/// Other platforms preserve fractional wheel ticks through their native
/// high-resolution output. Non-finite distance is rejected at this I/O
/// boundary; zero-distance terminal frames remain meaningful on macOS.
pub fn post_smooth_scroll(delta: ScrollDelta, phase: SmoothScrollPhase) {
    if !delta.is_finite() {
        return;
    }
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            macos::post_smooth_scroll(delta, phase);
        } else {
            let _ = phase;
            post_scroll(delta);
        }
    }
}

/// Return the `/dev/input/eventN` node for the action-injector uinput device,
/// initialising it if needed.
///
/// Intended for debugging and manual smoke-testing (e.g. attaching `evtest`
/// before firing [`execute`]). Returns `None` on non-Linux platforms or
/// when the device could not be created (e.g. `/dev/uinput` not writable).
#[cfg(target_os = "linux")]
#[must_use]
pub fn action_device_path() -> Option<std::path::PathBuf> {
    linux::device_node()
}

/// Stamped into the `EVENT_SOURCE_USER_DATA` field of every mouse event
/// [`execute`] synthesizes on macOS, so OpenLogi's own `CGEventTap` can
/// recognize and skip its own injections. Without it, a gesture/button action
/// that posts a mouse button (e.g. a remapped `MiddleClick`) would re-enter the
/// hook — and for a gesture button, be misread as a fresh hold, looping. The
/// value is arbitrary but distinctive ("OLGI"); real events carry `0` here.
pub const SYNTHETIC_EVENT_USER_DATA: i64 = 0x4F4C_4749;

/// Translate a platform-neutral USB HID keyboard usage to a Win32 virtual key.
// Not `expect`: the lint fires in the `--lib` build and not in the `--test`
// one, so an expectation is always unfulfilled for one of them.
#[cfg_attr(
    not(target_os = "windows"),
    expect(clippy::allow_attributes, reason = "see above"),
    allow(dead_code, reason = "called only by the Windows backend")
)]
fn hid_usage_to_windows(usage: u8) -> Option<u16> {
    match usage {
        0x04..=0x1d => Some(u16::from(b'A' + usage - 0x04)),
        0x1e..=0x26 => Some(u16::from(b'1' + usage - 0x1e)),
        0x27 => Some(u16::from(b'0')),
        0x3a..=0x45 => Some(0x70 + u16::from(usage - 0x3a)),
        0x68..=0x6f => Some(0x7c + u16::from(usage - 0x68)),
        0x28 => Some(0x0d),
        0x29 => Some(0x1b),
        0x2a => Some(0x08),
        0x2b => Some(0x09),
        0x2c => Some(0x20),
        0x2d => Some(0xbd),
        0x2e => Some(0xbb),
        0x2f => Some(0xdb),
        0x30 => Some(0xdd),
        0x31 => Some(0xdc),
        0x33 => Some(0xba),
        0x34 => Some(0xde),
        0x35 => Some(0xc0),
        0x36 => Some(0xbc),
        0x37 => Some(0xbe),
        0x38 => Some(0xbf),
        0x4a => Some(0x24),
        0x4b => Some(0x21),
        0x4c => Some(0x2e),
        0x4d => Some(0x23),
        0x4e => Some(0x22),
        0x4f => Some(0x27),
        0x50 => Some(0x25),
        0x51 => Some(0x28),
        0x52 => Some(0x26),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use openlogi_core::scroll::ScrollDelta;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    use openlogi_core::binding::KeyCombo;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    use super::{HeldKey, HeldOutput, HoldTransition};
    use super::{QuantizedScroll, ScrollQuantizer};

    /// Synthetic high-resolution input: eight eighth-ticks must total exactly
    /// one Windows/Linux wheel detent (120 raw units). This is deterministic
    /// model data, not a hardware capture.
    #[test]
    fn fractional_frames_preserve_cumulative_wheel_distance() {
        let mut quantizer = ScrollQuantizer::default();
        let total = (0..8)
            .map(|_| {
                quantizer
                    .quantize(ScrollDelta::wheel_ticks(0.0, 0.125), 120.0)
                    .y
            })
            .sum::<i32>();
        assert_eq!(total, 120);
    }

    #[test]
    fn opposing_fractional_input_cancels_without_rounding_drift() {
        let mut quantizer = ScrollQuantizer::default();
        let forward = quantizer.quantize(ScrollDelta::wheel_ticks(0.25, 0.0), 120.0);
        let backward = quantizer.quantize(ScrollDelta::wheel_ticks(-0.25, 0.0), 120.0);
        assert_eq!(forward, QuantizedScroll { x: 30, y: 0 });
        assert_eq!(backward, QuantizedScroll { x: -30, y: 0 });
    }

    #[test]
    fn idle_timeout_clears_fractional_residuals() {
        use std::time::{Duration, Instant};
        let mut quantizer = ScrollQuantizer::default();
        let now = Instant::now();
        // Send fractional step (0.005 * 120 = 0.6 => rounds to 1, residual = -0.4)
        let _ = quantizer.quantize_at(ScrollDelta::wheel_ticks(0.0, 0.005), 120.0, now);
        assert_ne!(quantizer.residual_y, 0.0);

        // After > 250ms idle, residual should be cleared
        let later = now + Duration::from_millis(300);
        let _ = quantizer.quantize_at(ScrollDelta::wheel_ticks(0.0, 0.005), 120.0, later);
        // On new clean start: 0.005 * 120 = 0.6 => rounds to 1, residual = 0.6 - 1.0 = -0.4
        assert_eq!(quantizer.residual_y, -0.4);
    }

    #[test]
    fn axis_switch_clears_opposite_axis_residual() {
        use std::time::Instant;
        let mut quantizer = ScrollQuantizer::default();
        let now = Instant::now();
        // Send fractional vertical step
        let _ = quantizer.quantize_at(ScrollDelta::wheel_ticks(0.0, 0.005), 120.0, now);
        assert_ne!(quantizer.residual_y, 0.0);

        // Switch to horizontal axis
        let _ = quantizer.quantize_at(ScrollDelta::wheel_ticks(0.005, 0.0), 120.0, now);
        assert_eq!(quantizer.residual_y, 0.0);
        assert_ne!(quantizer.residual_x, 0.0);
    }

    #[test]
    fn direction_reversal_clears_fractional_residual() {
        use std::time::Instant;
        let mut quantizer = ScrollQuantizer::default();
        let now = Instant::now();
        // Send fractional vertical step upwards (+y)
        let _ = quantizer.quantize_at(ScrollDelta::wheel_ticks(0.0, 0.005), 120.0, now);
        assert_ne!(quantizer.residual_y, 0.0);

        // Reverse direction (-y)
        let _ = quantizer.quantize_at(ScrollDelta::wheel_ticks(0.0, -0.005), 120.0, now);
        // -0.005 * 120 = -0.6 (since residual was cleared on reversal, exact = -0.6 => rounds to -1, residual = 0.4)
        assert_eq!(quantizer.residual_y, 0.4);
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn combo(label: &str) -> KeyCombo {
        label.parse().expect("test shortcut must be valid")
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn shared_control_stays_down_until_its_last_chord_ends() {
        let control_a = combo("Ctrl+A");
        let control_b = combo("Ctrl+B");
        let mut output = HeldOutput::default();

        assert_eq!(
            output.transition(None, Some(&control_a)),
            HoldTransition {
                up: vec![],
                down: vec![HeldKey::Control, HeldKey::Key(control_a.key())],
            }
        );
        assert_eq!(
            output.transition(None, Some(&control_b)),
            HoldTransition {
                up: vec![],
                down: vec![HeldKey::Key(control_b.key())],
            }
        );
        assert_eq!(
            output.transition(Some(&control_a), None),
            HoldTransition {
                up: vec![HeldKey::Key(control_a.key())],
                down: vec![],
            }
        );
        assert_eq!(
            output.transition(Some(&control_b), None),
            HoldTransition {
                up: vec![HeldKey::Control, HeldKey::Key(control_b.key())],
                down: vec![],
            }
        );
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    fn command_and_control_share_one_physical_output() {
        let command_a = combo("Cmd+A");
        let control_b = combo("Ctrl+B");
        let mut output = HeldOutput::default();

        output.transition(None, Some(&command_a));
        assert_eq!(
            output.transition(None, Some(&control_b)),
            HoldTransition {
                up: vec![],
                down: vec![HeldKey::Key(control_b.key())],
            }
        );
        assert_eq!(
            output.transition(Some(&command_a), None),
            HoldTransition {
                up: vec![HeldKey::Key(command_a.key())],
                down: vec![],
            }
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn command_and_control_are_distinct_physical_outputs() {
        let command_a = combo("Cmd+A");
        let control_b = combo("Ctrl+B");
        let mut output = HeldOutput::default();

        output.transition(None, Some(&command_a));
        assert_eq!(
            output.transition(None, Some(&control_b)),
            HoldTransition {
                up: vec![],
                down: vec![HeldKey::Control, HeldKey::Key(control_b.key())],
            }
        );
        assert_eq!(
            output.transition(Some(&command_a), None),
            HoldTransition {
                up: vec![HeldKey::Command, HeldKey::Key(command_a.key())],
                down: vec![],
            }
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn shared_command_stays_down_until_its_last_chord_ends() {
        let command_a = combo("Cmd+A");
        let command_b = combo("Cmd+B");
        let mut output = HeldOutput::default();

        output.transition(None, Some(&command_a));
        assert_eq!(
            output.transition(None, Some(&command_b)),
            HoldTransition {
                up: vec![],
                down: vec![HeldKey::Key(command_b.key())],
            }
        );
        assert_eq!(
            output.transition(Some(&command_a), None),
            HoldTransition {
                up: vec![HeldKey::Key(command_a.key())],
                down: vec![],
            }
        );
        assert_eq!(
            output.transition(Some(&command_b), None),
            HoldTransition {
                up: vec![HeldKey::Command, HeldKey::Key(command_b.key())],
                down: vec![],
            }
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn replacement_preserves_shared_physical_outputs() {
        let old = combo("Ctrl+A");
        let new = combo("Ctrl+B");
        let mut output = HeldOutput::default();

        output.transition(None, Some(&old));
        assert_eq!(
            output.transition(Some(&old), Some(&new)),
            HoldTransition {
                up: vec![HeldKey::Key(old.key())],
                down: vec![HeldKey::Key(new.key())],
            }
        );
    }

    #[test]
    fn hid_usages_map_across_windows_key_categories() {
        use super::hid_usage_to_windows;

        assert_eq!(hid_usage_to_windows(0x04), Some(0x41)); // A
        assert_eq!(hid_usage_to_windows(0x1e), Some(0x31)); // 1
        assert_eq!(hid_usage_to_windows(0x3a), Some(0x70)); // F1
        assert_eq!(hid_usage_to_windows(0x6f), Some(0x83)); // F20
        assert_eq!(hid_usage_to_windows(0x50), Some(0x25)); // Left
        assert_eq!(hid_usage_to_windows(0x2c), Some(0x20)); // Space
        assert_eq!(hid_usage_to_windows(0x33), Some(0xba)); // Semicolon
        assert_eq!(hid_usage_to_windows(0xff), None);
    }

    #[test]
    fn saved_shortcut_application_target_matches_open_application() {
        use openlogi_core::binding::{Action, ActionTarget, ApplicationTarget};

        let app_target =
            ApplicationTarget::new("/System/Applications/Calculator.app", "Calculator")
                .expect("valid application target");

        let direct = Action::OpenApplication(app_target.clone());
        let saved = Action::SavedShortcut {
            id: "calc".into(),
            name: "Calculator".into(),
            target: ActionTarget::Application(app_target.clone()),
            icon: None,
        };

        assert_eq!(direct.application_target(), Some(&app_target));
        assert_eq!(saved.application_target(), Some(&app_target));
    }
}
