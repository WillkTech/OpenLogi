//! Mathematical utilities and formulas for OpenLogi.

use az::SaturatingAs;

/// Wheel rate threshold (in ticks/second) below which scrolling receives no acceleration (gain = 1.0).
pub const REFERENCE_RATE: f32 = 4.0;

/// Scaling denominator for the smooth monotonic gain curve.
pub const CURVE_SCALE: f32 = 8.0;

/// Calculate the scroll gain multiplier for a given velocity `rate` (in ticks/sec).
#[must_use]
#[inline]
pub fn gain_at_rate(rate: f32, enabled: bool, factor: f32, max_gain: f32) -> f32 {
    if !enabled || max_gain <= 1.0 || !rate.is_finite() || !factor.is_finite() || factor <= 0.0 {
        return 1.0;
    }

    let excess_speed = (rate - REFERENCE_RATE).max(0.0);
    if excess_speed == 0.0 {
        return 1.0;
    }

    let curve = excess_speed / (excess_speed + CURVE_SCALE);
    let gain = 1.0 + (max_gain - 1.0) * curve * factor;

    gain.max(1.0).min(max_gain.max(1.0))
}

/// Quantizes a floating point input into a bounded integer type `T`.
///
/// NaN values default to `nan_fallback`. The input is clamped to `[min, max]`,
/// rounded to the nearest integer, and converted to `T` with saturation.
#[must_use]
#[inline]
pub fn quantize<T>(value: f32, min: f32, max: f32, nan_fallback: f32) -> T
where
    f32: az::SaturatingCast<T>,
{
    let value = if value.is_nan() { nan_fallback } else { value };
    value.clamp(min, max).round().saturating_as::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the disabled path returns the literal 1.0 exactly"
    )]
    fn gain_at_rate_disabled_returns_unity() {
        assert_eq!(gain_at_rate(10.0, false, 1.0, 2.5), 1.0);
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the below-reference path returns the literal 1.0 exactly"
    )]
    fn gain_at_rate_below_reference_returns_unity() {
        assert_eq!(gain_at_rate(0.0, true, 1.0, 2.5), 1.0);
        assert_eq!(gain_at_rate(1.0, true, 1.0, 2.5), 1.0);
        assert_eq!(gain_at_rate(3.0, true, 1.0, 2.5), 1.0);
    }

    #[test]
    fn gain_at_rate_above_reference_increases_up_to_max() {
        let g1 = gain_at_rate(10.0, true, 1.0, 2.5);
        let g2 = gain_at_rate(40.0, true, 1.0, 2.5);
        assert!(g1 > 1.0);
        assert!(g2 > g1);
        assert!(g2 <= 2.5);
    }

    #[test]
    fn gain_at_rate_f32_f64_numerical_precision_comparison() {
        // Reference double-precision calculation
        fn gain_at_rate_f64(rate: f64, enabled: bool, factor: f64, max_gain: f64) -> f64 {
            if !enabled
                || max_gain <= 1.0
                || !rate.is_finite()
                || !factor.is_finite()
                || factor <= 0.0
            {
                return 1.0;
            }
            let excess_speed = (rate - 4.0).max(0.0);
            if excess_speed == 0.0 {
                return 1.0;
            }
            let curve = excess_speed / (excess_speed + 8.0);
            let gain = 1.0 + (max_gain - 1.0) * curve * factor;
            gain.max(1.0).min(max_gain.max(1.0))
        }

        // Test representative input spectrum
        let rates = [0.0, 1.0, 3.9, 4.0, 4.1, 10.0, 25.0, 50.0, 100.0, 500.0];
        let factors = [0.2, 0.5, 1.0, 1.5, 2.0, 4.0];
        let max_gains = [1.5, 2.0, 2.5, 3.0, 5.0];

        let mut max_diff: f64 = 0.0;

        for &rate in &rates {
            for &factor in &factors {
                for &max_gain in &max_gains {
                    let g32 = f64::from(gain_at_rate(rate, true, factor, max_gain));
                    let g64 = gain_at_rate_f64(
                        f64::from(rate),
                        true,
                        f64::from(factor),
                        f64::from(max_gain),
                    );
                    let diff = (g32 - g64).abs();
                    if diff > max_diff {
                        max_diff = diff;
                    }
                    assert!(
                        diff < 1e-6,
                        "f32 divergence exceeds threshold: diff={diff}, f32={g32}, f64={g64} at rate={rate}, factor={factor}, max_gain={max_gain}"
                    );
                }
            }
        }
    }

    #[test]
    fn quantize_handles_nan_and_clamping() {
        assert_eq!(quantize::<u8>(f32::NAN, 1.0, 100.0, 1.0), 1);
        assert_eq!(quantize::<u8>(-10.0, 1.0, 100.0, 1.0), 1);
        assert_eq!(quantize::<u8>(150.0, 1.0, 100.0, 1.0), 100);
        assert_eq!(quantize::<u8>(49.6, 1.0, 100.0, 1.0), 50);
    }
}
