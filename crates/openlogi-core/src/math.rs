//! Mathematical utilities and formulas for OpenLogi.

use az::SaturatingAs;
use serde::{Deserialize, Serialize};

/// Fixed-point scale factor (1000 = 1.000x).
pub const FIXED_POINT_SCALE: i32 = 1_000;

/// Unity gain in fixed point (1000 = 1.000x).
pub const UNITY_GAIN_FIXED: i32 = 1_000;

/// Wheel rate threshold (in ticks/second) below which scrolling receives no acceleration (gain = 1.0).
pub const REFERENCE_RATE: f32 = 4.0;

/// Wheel rate (in ticks/second) at which acceleration reaches the full Multiplier ceiling.
pub const MAX_RATE: f32 = 40.0;

/// Scroll acceleration / smoothing curve preset.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurvePreset {
    /// 1:1 direct passthrough without dynamic curve processing.
    #[default]
    None,
    /// Low-latency smooth curve with gentle response.
    #[serde(alias = "custom")]
    Smooth,
    /// Linear velocity scaling.
    Linear,
    /// Exponential progressive acceleration.
    Exponential,
    /// Logarithmic compressed acceleration.
    Logarithmic,
    /// Quadratic parabolic acceleration.
    Quadratic,
    /// Cubic steep acceleration.
    Cubic,
    /// Quartic very steep acceleration.
    Quartic,
    /// Square-root rapid early acceleration with diminishing growth.
    #[serde(alias = "sqrt")]
    SquareRoot,
    /// S-shaped sigmoid acceleration profile.
    #[serde(alias = "s_curve", alias = "scurve")]
    Sigmoid,
    /// Gentle start followed by stronger acceleration.
    EaseIn,
    /// Quick initial response gradually flattening.
    EaseOut,
    /// Slow start and finish with peak responsiveness in the middle.
    EaseInOut,
}

impl CurvePreset {
    /// Every preset option for UI selectors.
    pub const ALL: [Self; 13] = [
        Self::None,
        Self::Smooth,
        Self::Linear,
        Self::Exponential,
        Self::Logarithmic,
        Self::Quadratic,
        Self::Cubic,
        Self::Quartic,
        Self::SquareRoot,
        Self::Sigmoid,
        Self::EaseIn,
        Self::EaseOut,
        Self::EaseInOut,
    ];

    /// User-facing label key for translation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Smooth => "Smooth",
            Self::Linear => "Linear",
            Self::Exponential => "Exponential",
            Self::Logarithmic => "Logarithmic",
            Self::Quadratic => "Quadratic",
            Self::Cubic => "Cubic",
            Self::Quartic => "Quartic",
            Self::SquareRoot => "Square Root",
            Self::Sigmoid => "Sigmoid",
            Self::EaseIn => "Ease-In",
            Self::EaseOut => "Ease-Out",
            Self::EaseInOut => "Ease-In-Out",
        }
    }

    /// Whether dynamic acceleration / smoothing is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Calculate normalized curve response f(x) for normalized velocity x in [0.0, 1.0].
    ///
    /// Guarantees:
    /// - f(0.0) == 0.0
    /// - f(1.0) == 1.0 (for enabled presets)
    /// - Monotonically non-decreasing on [0.0, 1.0]
    /// - f(x) in [0.0, 1.0]
    #[must_use]
    pub fn evaluate_normalized(self, x: f32) -> f32 {
        if !x.is_finite() || x <= 0.0 {
            return 0.0;
        }
        let x = x.min(1.0);
        match self {
            Self::None => 0.0,
            Self::Linear => x,
            Self::Smooth => (2.0 * x) / (x + 1.0),
            Self::Quadratic | Self::EaseIn => x * x,
            Self::Cubic => x * x * x,
            Self::Quartic => x * x * x * x,
            Self::SquareRoot => x.sqrt(),
            Self::Exponential => {
                const K: f32 = 3.0;
                let den = K.exp() - 1.0;
                let num = (K * x).exp() - 1.0;
                (num / den).clamp(0.0, 1.0)
            }
            Self::Logarithmic => {
                let num = (1.0 + 9.0 * x).ln();
                let den = 10.0f32.ln();
                (num / den).clamp(0.0, 1.0)
            }
            Self::Sigmoid => {
                let s = |t: f32| 1.0 / (1.0 + (-6.0 * (t - 0.5)).exp());
                let s0 = s(0.0);
                let s1 = s(1.0);
                ((s(x) - s0) / (s1 - s0)).clamp(0.0, 1.0)
            }
            Self::EaseOut => x * (2.0 - x),
            Self::EaseInOut => x * x * (3.0 - 2.0 * x),
        }
    }
}

/// Calculate the scroll gain multiplier for a given velocity `rate` (in ticks/sec), `preset`, and `multiplier`.
#[must_use]
pub fn gain_at_rate_preset(rate: f32, preset: CurvePreset, multiplier: f32) -> f32 {
    if !preset.is_enabled()
        || multiplier <= 1.0
        || !rate.is_finite()
        || !multiplier.is_finite()
        || rate <= REFERENCE_RATE
    {
        return 1.0;
    }

    let norm_rate = ((rate - REFERENCE_RATE) / (MAX_RATE - REFERENCE_RATE)).clamp(0.0, 1.0);
    let curve_val = preset.evaluate_normalized(norm_rate);
    let gain = 1.0 + (multiplier - 1.0) * curve_val;
    gain.clamp(1.0, multiplier.max(1.0))
}

/// Calculate the scroll gain multiplier for a given velocity `rate` (in ticks/sec) and enabled state.
#[must_use]
#[inline]
pub fn gain_at_rate(rate: f32, enabled: bool, multiplier: f32) -> f32 {
    let preset = if enabled {
        CurvePreset::Smooth
    } else {
        CurvePreset::None
    };
    gain_at_rate_preset(rate, preset, multiplier)
}

/// Stage 1 normalized shape lookup table (0 = 0.000, 1000 = 1.000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeLut {
    table: [i32; 256],
}

impl ShapeLut {
    /// Generate a normalized shape table for a preset.
    #[must_use]
    pub fn generate(preset: CurvePreset) -> Self {
        let mut table = [0; 256];
        if !preset.is_enabled() {
            return Self { table };
        }
        for (rate_bin, slot) in table.iter_mut().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "rate_bin is 0..255")]
            let rate = rate_bin as f32;
            if rate <= REFERENCE_RATE {
                *slot = 0;
            } else {
                let norm_rate =
                    ((rate - REFERENCE_RATE) / (MAX_RATE - REFERENCE_RATE)).clamp(0.0, 1.0);
                let norm_gain = preset.evaluate_normalized(norm_rate);
                *slot = quantize_fixed_point(norm_gain);
            }
        }
        Self { table }
    }

    /// Lookup the normalized response in fixed point (0..=1000).
    #[must_use]
    #[inline]
    pub const fn lookup(&self, rate_bin: usize) -> i32 {
        if rate_bin >= 256 {
            self.table[255]
        } else {
            self.table[rate_bin]
        }
    }
}

/// 256-slot static fixed-point lookup table for scroll gain (1000 = 1.000x).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollLut {
    table: [i32; 256],
}

impl ScrollLut {
    /// Lookup the fixed-point gain multiplier for a velocity bin (in ticks/sec, clamped 0..=255).
    #[must_use]
    #[inline]
    pub const fn lookup(&self, rate_bin: usize) -> i32 {
        if rate_bin >= 256 {
            self.table[255]
        } else {
            self.table[rate_bin]
        }
    }

    /// Lookup gain as a floating point value.
    #[must_use]
    #[inline]
    pub fn lookup_f32(&self, rate_bin: usize) -> f32 {
        fixed_to_f32(self.lookup(rate_bin))
    }

    /// Generate the active LUT for a given preset and multiplier.
    #[must_use]
    pub fn generate(preset: CurvePreset, multiplier: f32) -> Self {
        let mut table = [UNITY_GAIN_FIXED; 256];
        if !preset.is_enabled() || multiplier <= 1.0 {
            return Self { table };
        }
        for (rate_bin, slot) in table.iter_mut().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "rate_bin is 0..255")]
            let rate = rate_bin as f32;
            let gain = gain_at_rate_preset(rate, preset, multiplier);
            *slot = quantize_fixed_point(gain);
        }
        Self { table }
    }
}

impl Default for ScrollLut {
    fn default() -> Self {
        Self {
            table: [UNITY_GAIN_FIXED; 256],
        }
    }
}

/// Convert floating point value into fixed-point integer (1000 = 1.000).
#[must_use]
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "fixed-point conversion precision with scale factor 1000 is exact in f32"
)]
pub fn quantize_fixed_point(value: f32) -> i32 {
    let scaled = value * (FIXED_POINT_SCALE as f32);
    if scaled.is_nan() {
        UNITY_GAIN_FIXED
    } else {
        scaled.round() as i32
    }
}

/// Convert fixed-point integer (1000 = 1.000) into floating point value.
#[must_use]
#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "fixed-point conversion precision with scale factor 1000 is exact in f32"
)]
pub fn fixed_to_f32(value: i32) -> f32 {
    (value as f32) / (FIXED_POINT_SCALE as f32)
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
        assert_eq!(gain_at_rate(10.0, false, 2.5), 1.0);
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "the below-reference path returns the literal 1.0 exactly"
    )]
    fn gain_at_rate_below_reference_returns_unity() {
        assert_eq!(gain_at_rate(0.0, true, 2.5), 1.0);
        assert_eq!(gain_at_rate(1.0, true, 2.5), 1.0);
        assert_eq!(gain_at_rate(4.0, true, 2.5), 1.0);
    }

    #[test]
    fn gain_at_rate_above_reference_increases_up_to_max() {
        let g1 = gain_at_rate(10.0, true, 2.5);
        let g2 = gain_at_rate(40.0, true, 2.5);
        assert!(g1 > 1.0);
        assert!(g2 > g1);
        assert!((g2 - 2.5).abs() < 1e-5);
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        clippy::cast_precision_loss,
        reason = "preset None returns literal 0.0 and step is a bounded loop counter"
    )]
    fn test_all_preset_boundary_conditions_and_monotonicity() {
        for preset in CurvePreset::ALL {
            if preset == CurvePreset::None {
                assert_eq!(preset.evaluate_normalized(0.0), 0.0);
                assert_eq!(preset.evaluate_normalized(1.0), 0.0);
                continue;
            }

            let f0 = preset.evaluate_normalized(0.0);
            let f1 = preset.evaluate_normalized(1.0);
            assert!(
                (f0 - 0.0).abs() < 1e-6,
                "preset {preset:?} f(0) must be 0.0, got {f0}"
            );
            assert!(
                (f1 - 1.0).abs() < 1e-6,
                "preset {preset:?} f(1) must be 1.0, got {f1}"
            );

            // Monotonicity check
            let mut last_val = 0.0;
            for step in 0..=100 {
                let x = step as f32 / 100.0;
                let val = preset.evaluate_normalized(x);
                assert!(
                    val.is_finite(),
                    "preset {preset:?} produced non-finite at {x}"
                );
                assert!(
                    val >= last_val - 1e-6,
                    "preset {preset:?} violated monotonicity at {x}: {val} < {last_val}"
                );
                assert!(
                    (0.0..=1.0001).contains(&val),
                    "preset {preset:?} value out of range at {x}: {val}"
                );
                last_val = val;
            }
        }
    }

    #[test]
    fn test_multiplier_invariance_across_presets() {
        let multipliers = [1.0, 1.5, 2.0, 2.5, 3.0, 5.0];
        let rates = [4.0, 8.0, 15.0, 22.0, 30.0, 40.0, 60.0];

        for preset in CurvePreset::ALL {
            if preset == CurvePreset::None {
                continue;
            }

            for &rate in &rates {
                for &m1 in &multipliers {
                    for &m2 in &multipliers {
                        if m1 <= 1.0 || m2 <= 1.0 {
                            continue;
                        }
                        let g1 = gain_at_rate_preset(rate, preset, m1);
                        let g2 = gain_at_rate_preset(rate, preset, m2);

                        let norm1 = (g1 - 1.0) / (m1 - 1.0);
                        let norm2 = (g2 - 1.0) / (m2 - 1.0);

                        assert!(
                            (norm1 - norm2).abs() < 1e-5,
                            "Multiplier invariance failed for {preset:?} at rate={rate}: m1={m1} (norm={norm1}), m2={m2} (norm={norm2})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_curve_presets_and_lut() {
        for preset in CurvePreset::ALL {
            let lut = ScrollLut::generate(preset, 2.5);
            if preset == CurvePreset::None {
                assert_eq!(lut.lookup(10), UNITY_GAIN_FIXED);
                assert_eq!(lut.lookup(50), UNITY_GAIN_FIXED);
            } else {
                assert_eq!(lut.lookup(0), UNITY_GAIN_FIXED);
                assert_eq!(lut.lookup(4), UNITY_GAIN_FIXED); // REFERENCE_RATE = 4.0
                let g10 = lut.lookup(10);
                let g40 = lut.lookup(40);
                assert!(g10 >= UNITY_GAIN_FIXED);
                assert!(g40 >= g10);
                assert_eq!(g40, 2500); // 2.5 multiplier * 1000
                assert_eq!(lut.lookup(100), 2500); // Clamped saturation at peak
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

    #[test]
    fn fixed_point_conversions_work() {
        assert_eq!(quantize_fixed_point(1.0), 1000);
        assert_eq!(quantize_fixed_point(2.5), 2500);
        assert_eq!(quantize_fixed_point(f32::NAN), 1000);
        assert!((fixed_to_f32(2500) - 2.5).abs() < 1e-6);
    }
}
