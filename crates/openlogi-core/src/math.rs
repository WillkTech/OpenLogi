//! Mathematical utilities and formulas for OpenLogi.

use az::SaturatingAs;

/// Wheel rate threshold (in ticks/second) below which scrolling receives no acceleration (gain = 1.0).
pub const REFERENCE_RATE: f64 = 4.0;

/// Scaling denominator for the smooth monotonic gain curve.
pub const CURVE_SCALE: f64 = 8.0;

/// Calculate the scroll gain multiplier for a given velocity `rate` (in ticks/sec).
#[must_use]
#[inline]
pub fn gain_at_rate(rate: f64, enabled: bool, factor: f64, max_gain: f64) -> f64 {
    if !enabled
        || max_gain <= 1.0
        || !rate.is_finite()
        || !factor.is_finite()
        || factor <= 0.0
    {
        return 1.0;
    }

    let excess_speed = (rate - REFERENCE_RATE).max(0.0);
    if excess_speed == 0.0 {
        return 1.0;
    }

    let curve = excess_speed / (excess_speed + CURVE_SCALE);
    let gain = 1.0 + (max_gain - 1.0) * curve * factor;

    gain.clamp(1.0, max_gain)
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
    fn gain_at_rate_disabled_returns_unity() {
        assert_eq!(gain_at_rate(10.0, false, 1.0, 2.5), 1.0);
    }

    #[test]
    fn gain_at_rate_below_reference_returns_unity() {
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
    fn quantize_handles_nan_and_clamping() {
        assert_eq!(quantize::<u8>(f32::NAN, 1.0, 100.0, 1.0), 1);
        assert_eq!(quantize::<u8>(-10.0, 1.0, 100.0, 1.0), 1);
        assert_eq!(quantize::<u8>(150.0, 1.0, 100.0, 1.0), 100);
        assert_eq!(quantize::<u8>(49.6, 1.0, 100.0, 1.0), 50);
    }
}
