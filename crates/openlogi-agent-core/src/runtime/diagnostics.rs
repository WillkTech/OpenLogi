//! Lightweight high-resolution diagnostic metrics for the input pipeline.
#![expect(
    unsafe_code,
    reason = "mach_absolute_time FFI for sub-microsecond monotonic latency metrics on macOS"
)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[cfg(target_os = "macos")]
#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    fn mach_absolute_time() -> u64;
}

/// Global flag controlling whether detailed pipeline latency probes are active.
/// When false, probe sites execute a single branch on an atomic boolean (<1ns).
static DIAGNOSTICS_ENABLED: AtomicBool = AtomicBool::new(false);

/// Current diagnostic metrics aggregated across the input pipeline.
#[derive(Default)]
pub struct PipelineMetrics {
    pub events_processed: AtomicU64,
    pub total_callback_duration_ns: AtomicU64,
    pub max_callback_duration_ns: AtomicU64,
    pub total_dispatch_duration_ns: AtomicU64,
    pub max_dispatch_duration_ns: AtomicU64,
    pub dropped_events: AtomicU64,
}

static METRICS: PipelineMetrics = PipelineMetrics {
    events_processed: AtomicU64::new(0),
    total_callback_duration_ns: AtomicU64::new(0),
    max_callback_duration_ns: AtomicU64::new(0),
    total_dispatch_duration_ns: AtomicU64::new(0),
    max_dispatch_duration_ns: AtomicU64::new(0),
    dropped_events: AtomicU64::new(0),
};

/// High-resolution monotonic timestamp (nanoseconds on Apple Silicon).
#[inline]
#[must_use]
pub fn current_time_ns() -> u64 {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: mach_absolute_time is a standard Darwin libSystem timer function.
        // On Apple Silicon, 1 tick = 1 nanosecond (timebase 1/1).
        unsafe { mach_absolute_time() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64)
    }
}

/// Enable or disable diagnostic mode.
pub fn set_diagnostics_enabled(enabled: bool) {
    DIAGNOSTICS_ENABLED.store(enabled, Ordering::Release);
}

/// Whether diagnostic latency sampling is active.
#[inline]
#[must_use]
pub fn diagnostics_enabled() -> bool {
    DIAGNOSTICS_ENABLED.load(Ordering::Relaxed)
}

/// Record callback execution duration.
pub fn record_callback_duration(duration_ns: u64) {
    if !diagnostics_enabled() {
        return;
    }
    METRICS.events_processed.fetch_add(1, Ordering::Relaxed);
    METRICS
        .total_callback_duration_ns
        .fetch_add(duration_ns, Ordering::Relaxed);
    let mut current_max = METRICS.max_callback_duration_ns.load(Ordering::Relaxed);
    while duration_ns > current_max {
        match METRICS.max_callback_duration_ns.compare_exchange_weak(
            current_max,
            duration_ns,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => current_max = actual,
        }
    }
}

/// Record dispatch / injection execution duration.
pub fn record_dispatch_duration(duration_ns: u64) {
    if !diagnostics_enabled() {
        return;
    }
    METRICS
        .total_dispatch_duration_ns
        .fetch_add(duration_ns, Ordering::Relaxed);
    let mut current_max = METRICS.max_dispatch_duration_ns.load(Ordering::Relaxed);
    while duration_ns > current_max {
        match METRICS.max_dispatch_duration_ns.compare_exchange_weak(
            current_max,
            duration_ns,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => current_max = actual,
        }
    }
}

/// Record a dropped event.
pub fn record_dropped_event() {
    if !diagnostics_enabled() {
        return;
    }
    METRICS.dropped_events.fetch_add(1, Ordering::Relaxed);
}

/// Snapshot of diagnostic metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticSnapshot {
    pub events_processed: u64,
    pub avg_callback_duration_ns: u64,
    pub max_callback_duration_ns: u64,
    pub avg_dispatch_duration_ns: u64,
    pub max_dispatch_duration_ns: u64,
    pub dropped_events: u64,
}

/// Read current diagnostic metrics snapshot.
#[must_use]
pub fn snapshot() -> DiagnosticSnapshot {
    let events = METRICS.events_processed.load(Ordering::Relaxed);
    let total_cb = METRICS.total_callback_duration_ns.load(Ordering::Relaxed);
    let total_disp = METRICS.total_dispatch_duration_ns.load(Ordering::Relaxed);
    DiagnosticSnapshot {
        events_processed: events,
        avg_callback_duration_ns: total_cb.checked_div(events).unwrap_or(0),
        max_callback_duration_ns: METRICS.max_callback_duration_ns.load(Ordering::Relaxed),
        avg_dispatch_duration_ns: total_disp.checked_div(events).unwrap_or(0),
        max_dispatch_duration_ns: METRICS.max_dispatch_duration_ns.load(Ordering::Relaxed),
        dropped_events: METRICS.dropped_events.load(Ordering::Relaxed),
    }
}

/// Reset diagnostic counters.
pub fn reset() {
    METRICS.events_processed.store(0, Ordering::Relaxed);
    METRICS
        .total_callback_duration_ns
        .store(0, Ordering::Relaxed);
    METRICS.max_callback_duration_ns.store(0, Ordering::Relaxed);
    METRICS
        .total_dispatch_duration_ns
        .store(0, Ordering::Relaxed);
    METRICS.max_dispatch_duration_ns.store(0, Ordering::Relaxed);
    METRICS.dropped_events.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_recording_and_snapshot() {
        set_diagnostics_enabled(true);
        reset();

        record_callback_duration(500);
        record_callback_duration(1500);
        record_dispatch_duration(300);
        record_dropped_event();

        let s = snapshot();
        assert_eq!(s.events_processed, 2);
        assert_eq!(s.avg_callback_duration_ns, 1000);
        assert_eq!(s.max_callback_duration_ns, 1500);
        assert_eq!(s.dropped_events, 1);

        reset();
        set_diagnostics_enabled(false);
    }
}
