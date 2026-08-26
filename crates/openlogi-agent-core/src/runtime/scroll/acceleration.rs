//! Mouse-wheel acceleration engine.
//!
//! Provides independent stateful vertical and horizontal mouse-wheel acceleration
//! based on normalized physical wheel velocity (ticks per second).

use std::time::{Duration, Instant};

use openlogi_core::math;

/// Time without wheel input after which acceleration state decays/resets.
const IDLE_TIMEOUT: Duration = Duration::from_millis(250);

/// Smoothing factor for exponential moving average of estimated wheel rate.
const ALPHA: f64 = 0.4;

/// Per-axis acceleration parameters.
#[derive(Debug, Clone, Copy)]
pub struct AxisAccelerationParams {
    /// Whether acceleration is enabled for the axis.
    pub enabled: bool,
    /// Acceleration strength factor.
    pub factor: f64,
    /// Upper bound on the gain multiplier.
    pub max_gain: f64,
}

/// Stateful acceleration tracker for a single scroll axis.
#[derive(Debug, Default, Clone)]
pub struct AxisAcceleration {
    last_event_time: Option<Instant>,
    estimated_rate: f64,
    last_direction: f64,
}

impl AxisAcceleration {
    /// Reset all internal state (e.g., after direction reversal or long idle).
    pub fn reset(&mut self) {
        self.last_event_time = None;
        self.estimated_rate = 0.0;
        self.last_direction = 0.0;
    }

    /// Calculate the acceleration gain factor for an incoming delta tick on this axis at `at`.
    #[expect(
        clippy::float_cmp,
        reason = "0.0 is the exact sentinel `signum` never produces for a valid delta"
    )]
    pub fn compute_gain(
        &mut self,
        delta_ticks: f64,
        at: Instant,
        enabled: bool,
        acceleration_factor: f64,
        max_gain: f64,
    ) -> f64 {
        if !enabled
            || max_gain <= 1.0
            || delta_ticks == 0.0
            || !delta_ticks.is_finite()
            || !acceleration_factor.is_finite()
            || acceleration_factor <= 0.0
        {
            self.reset();
            return 1.0;
        }

        let dir = delta_ticks.signum();

        // Direction reversal check
        if self.last_direction != 0.0 && dir != self.last_direction {
            self.reset();
            self.last_direction = dir;
            self.last_event_time = Some(at);
            return 1.0;
        }

        let abs_delta = delta_ticks.abs();

        if let Some(last_time) = self.last_event_time {
            let dt = at.saturating_duration_since(last_time);
            let dt_secs = dt.as_secs_f64();

            if dt >= IDLE_TIMEOUT {
                // Long idle: decay state back to zero
                self.reset();
                self.last_direction = dir;
                self.last_event_time = Some(at);
                return 1.0;
            }

            if dt_secs > 0.0 {
                let instantaneous_rate = abs_delta / dt_secs;
                if instantaneous_rate.is_finite() {
                    if self.estimated_rate == 0.0 {
                        self.estimated_rate = instantaneous_rate;
                    } else {
                        self.estimated_rate =
                            (1.0 - ALPHA) * self.estimated_rate + ALPHA * instantaneous_rate;
                    }
                }
            }
        } else {
            // First event: establish direction and timestamp. Rate starts at 0.0 so initial gain = 1.0.
            self.estimated_rate = 0.0;
        }

        self.last_direction = dir;
        self.last_event_time = Some(at);

        math::gain_at_rate(self.estimated_rate, enabled, acceleration_factor, max_gain)
    }
}

/// Independent two-axis mouse-wheel acceleration state machine.
#[derive(Debug, Default, Clone)]
pub struct ScrollAccelerationEngine {
    vertical: AxisAcceleration,
    horizontal: AxisAcceleration,
}

impl ScrollAccelerationEngine {
    /// Compute accelerated (x, y) scaling factors for a wheel impulse (x, y) at `at`.
    ///
    /// Returns `(gain_x, gain_y)`.
    pub fn compute_gains(
        &mut self,
        x_ticks: f64,
        y_ticks: f64,
        at: Instant,
        vertical: AxisAccelerationParams,
        horizontal: AxisAccelerationParams,
    ) -> (f64, f64) {
        let gain_x = self.horizontal.compute_gain(
            x_ticks,
            at,
            horizontal.enabled,
            horizontal.factor,
            horizontal.max_gain,
        );
        let gain_y = self.vertical.compute_gain(
            y_ticks,
            at,
            vertical.enabled,
            vertical.factor,
            vertical.max_gain,
        );
        (gain_x, gain_y)
    }

    /// Reset both vertical and horizontal acceleration states.
    pub fn reset(&mut self) {
        self.vertical.reset();
        self.horizontal.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V: AxisAccelerationParams = AxisAccelerationParams {
        enabled: true,
        factor: 1.0,
        max_gain: 2.5,
    };
    const H: AxisAccelerationParams = AxisAccelerationParams {
        enabled: false,
        factor: 1.0,
        max_gain: 2.0,
    };
    const H_ON: AxisAccelerationParams = AxisAccelerationParams {
        enabled: true,
        factor: 1.0,
        max_gain: 2.0,
    };

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the disabled path returns the literal 1.0 exactly"
    )]
    fn test_acceleration_disabled_returns_unity_gain() {
        let mut engine = ScrollAccelerationEngine::default();
        let now = Instant::now();
        let (gx, gy) = engine.compute_gains(
            1.0,
            1.0,
            now,
            AxisAccelerationParams {
                enabled: false,
                ..V
            },
            H,
        );
        assert_eq!(gx, 1.0);
        assert_eq!(gy, 1.0);
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the first-event path feeds rate 0.0, which returns the literal 1.0"
    )]
    fn test_single_slow_event_returns_unity_gain() {
        let mut engine = ScrollAccelerationEngine::default();
        let now = Instant::now();
        let (_gx, gy) = engine.compute_gains(0.0, 1.0, now, V, H);
        assert_eq!(gy, 1.0);
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "sub-reference rates clamp excess speed to exactly 0.0 and return literal 1.0"
    )]
    fn test_repeated_slow_events_maintain_predictable_gain() {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        // Slow scrolling: 1 tick every 500ms (2 ticks/sec <= REFERENCE_RATE 4.0)
        for i in 0..10 {
            let t = start + Duration::from_millis(i * 500);
            let (_gx, gy) = engine.compute_gains(0.0, 1.0, t, V, H);
            assert_eq!(gy, 1.0, "slow deliberate tick {i} must have gain 1.0");
        }
    }

    #[test]
    fn test_repeated_fast_events_increase_gain_bounded_by_max() {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        // Fast scrolling: 1 tick every 20ms (50 ticks/sec > REFERENCE_RATE 4.0)
        let mut last_gain = 1.0;
        for i in 0..20 {
            let t = start + Duration::from_millis(i * 20);
            let (_gx, gy) = engine.compute_gains(0.0, 1.0, t, V, H);
            assert!(gy <= 2.5, "gain must never exceed max_gain 2.5");
            if i > 1 {
                assert!(
                    gy >= last_gain,
                    "gain must progressively increase during fast scrolling"
                );
            }
            last_gain = gy;
        }
        assert!(last_gain > 1.5, "fast scrolling must build noticeable gain");
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the reversal path returns the literal 1.0 exactly"
    )]
    fn test_direction_reversal_resets_acceleration_immediately() {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        // Fast scroll positive
        for i in 0..10 {
            engine.compute_gains(0.0, 1.0, start + Duration::from_millis(i * 20), V, H);
        }

        // Direction reversal to negative
        let t_reversal = start + Duration::from_millis(10 * 20);
        let (_gx, gy_rev) = engine.compute_gains(0.0, -1.0, t_reversal, V, H);
        assert_eq!(
            gy_rev, 1.0,
            "reversal event must immediately receive gain 1.0"
        );
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the idle-reset path returns the literal 1.0 exactly"
    )]
    fn test_idle_reset_after_pause() {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        // Fast scroll positive
        for i in 0..10 {
            engine.compute_gains(0.0, 1.0, start + Duration::from_millis(i * 20), V, H);
        }

        // Idle pause of 300ms (> IDLE_TIMEOUT 250ms)
        let t_idle = start + Duration::from_millis(200 + 300);
        let (_gx, gy_idle) = engine.compute_gains(0.0, 1.0, t_idle, V, H);
        assert_eq!(
            gy_idle, 1.0,
            "event after >250ms idle must receive gain 1.0"
        );
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "reversal and first-tick paths return the literal 1.0 exactly"
    )]
    fn test_axes_are_strictly_independent() {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();

        // Build vertical acceleration
        for i in 0..10 {
            engine.compute_gains(0.0, 1.0, start + Duration::from_millis(i * 20), V, H_ON);
        }

        // Reverse vertical direction -> vertical resets, but horizontal remains un-reset
        let t_v_rev = start + Duration::from_millis(10 * 20);
        let (gx_h, gy_v) = engine.compute_gains(1.0, -1.0, t_v_rev, V, H_ON);
        assert_eq!(
            gy_v, 1.0,
            "vertical reversal must reset vertical acceleration"
        );
        assert_eq!(gx_h, 1.0, "first horizontal tick must start at gain 1.0");
    }

    #[test]
    fn test_high_resolution_equivalent_rate_input() {
        let mut engine1 = ScrollAccelerationEngine::default();
        let mut engine2 = ScrollAccelerationEngine::default();
        let start = Instant::now();

        // Trace 1: initial tick at t=0, then 1 tick at t=100ms (rate = 1.0 / 0.1s = 10 ticks/sec)
        engine1.compute_gains(0.0, 1.0, start, V, H);
        let t1 = start + Duration::from_millis(100);
        let (_gx1, gy1) = engine1.compute_gains(0.0, 1.0, t1, V, H);

        // Trace 2: initial tick at t=0, then 8 x 0.125 ticks over 100ms (~12.5ms apart)
        engine2.compute_gains(0.0, 0.125, start, V, H);
        let mut gy2 = 1.0;
        for i in 1..=8 {
            let t = start + Duration::from_millis(i * 12 + 1);
            let (_gx, g) = engine2.compute_gains(0.0, 0.125, t, V, H);
            gy2 = g;
        }

        // Both represent ~10 ticks/sec physical rate
        assert!(
            (gy1 - gy2).abs() < 0.2,
            "high-resolution 8x0.125 ticks and 1x1.0 tick must yield equivalent rate gain: gy1={gy1}, gy2={gy2}"
        );
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "non-finite input hits the guard's literal 1.0 early-return"
    )]
    fn test_zero_and_invalid_timing_handled_safely() {
        let mut engine = ScrollAccelerationEngine::default();
        let now = Instant::now();

        // Simultaneous / zero dt
        let (_gx1, gy1) = engine.compute_gains(0.0, 1.0, now, V, H);
        let (_gx2, gy2) = engine.compute_gains(0.0, 1.0, now, V, H);

        assert!(gy1.is_finite());
        assert!(gy2.is_finite());
        assert!(!gy1.is_nan());
        assert!(!gy2.is_nan());

        // NaN / Infinite inputs
        let (_gx_nan, gy_nan) = engine.compute_gains(0.0, f64::NAN, now, V, H);
        assert_eq!(gy_nan, 1.0);
    }
}
