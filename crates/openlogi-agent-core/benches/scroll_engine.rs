#![expect(unsafe_code, reason = "mach_absolute_time FFI for latency measurement")]

use std::time::{Duration, Instant};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use openlogi_agent_core::runtime::scroll::{AxisAccelerationParams, ScrollAccelerationEngine};
use openlogi_core::config::{AppSettings, CurvePreset};
use openlogi_core::math::{ScrollLut, ShapeLut, gain_at_rate_preset};

fn bench_gain_at_rate(c: &mut Criterion) {
    c.bench_function("gain_at_rate_smooth", |b| {
        let mut rate = 5.0f32;
        b.iter(|| {
            rate += 0.1;
            if rate > 50.0 {
                rate = 5.0;
            }
            black_box(gain_at_rate_preset(
                black_box(rate),
                black_box(CurvePreset::Smooth),
                black_box(2.5),
            ))
        });
    });

    c.bench_function("gain_at_rate_sigmoid", |b| {
        let mut rate = 5.0f32;
        b.iter(|| {
            rate += 0.1;
            if rate > 50.0 {
                rate = 5.0;
            }
            black_box(gain_at_rate_preset(
                black_box(rate),
                black_box(CurvePreset::Sigmoid),
                black_box(2.5),
            ))
        });
    });
}

fn bench_compute_gains(c: &mut Criterion) {
    c.bench_function("compute_gains_none_passthrough", |b| {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        let v_params = AxisAccelerationParams {
            preset: CurvePreset::None,
            multiplier: 1.0,
        };
        let h_params = AxisAccelerationParams {
            preset: CurvePreset::None,
            multiplier: 1.0,
        };
        let mut step = 0u64;

        b.iter(|| {
            step += 1;
            let at = start + Duration::from_micros(step * 10);
            black_box(engine.compute_gains(
                black_box(0.5),
                black_box(1.0),
                black_box(at),
                black_box(v_params),
                black_box(h_params),
            ))
        });
    });

    c.bench_function("compute_gains_smooth", |b| {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        let v_params = AxisAccelerationParams {
            preset: CurvePreset::Smooth,
            multiplier: 2.5,
        };
        let h_params = AxisAccelerationParams {
            preset: CurvePreset::Linear,
            multiplier: 2.0,
        };
        let mut step = 0u64;

        b.iter(|| {
            step += 1;
            let at = start + Duration::from_micros(step * 10);
            black_box(engine.compute_gains(
                black_box(0.5),
                black_box(1.0),
                black_box(at),
                black_box(v_params),
                black_box(h_params),
            ))
        });
    });

    c.bench_function("compute_gains_fixed_lut", |b| {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        let v_lut = ScrollLut::generate(CurvePreset::Smooth, 2.5);
        let h_lut = ScrollLut::generate(CurvePreset::Linear, 2.0);
        let mut step = 0u64;

        b.iter(|| {
            step += 1;
            let at = start + Duration::from_micros(step * 10);
            black_box(engine.compute_gains_fixed(
                black_box(0.5),
                black_box(1.0),
                black_box(at),
                black_box(CurvePreset::Smooth),
                black_box(&v_lut),
                black_box(CurvePreset::Linear),
                black_box(&h_lut),
            ))
        });
    });
}

fn bench_lut_generation(c: &mut Criterion) {
    // Stage 1: Cold shape LUT initialization (calculates mathematical curve shapes)
    c.bench_function("initialize_smooth_shape_lut", |b| {
        b.iter(|| black_box(ShapeLut::generate(black_box(CurvePreset::Smooth))));
    });

    c.bench_function("initialize_sigmoid_shape_lut", |b| {
        b.iter(|| black_box(ShapeLut::generate(black_box(CurvePreset::Sigmoid))));
    });

    // Stage 2: Direct un-cached transcendental calculation (0.8.3 baseline)
    c.bench_function("generate_smooth_lut_direct_baseline", |b| {
        b.iter(|| {
            black_box(ScrollLut::generate_direct(
                black_box(CurvePreset::Smooth),
                black_box(2.5),
            ))
        });
    });

    c.bench_function("generate_sigmoid_lut_direct_baseline", |b| {
        b.iter(|| {
            black_box(ScrollLut::generate_direct(
                black_box(CurvePreset::Sigmoid),
                black_box(2.5),
            ))
        });
    });

    // Stage 3: Optimized shape-scaled LUT construction (0.8.4 vector-scaled)
    c.bench_function("generate_smooth_lut_optimized", |b| {
        b.iter(|| {
            black_box(ScrollLut::generate(
                black_box(CurvePreset::Smooth),
                black_box(2.5),
            ))
        });
    });

    c.bench_function("generate_sigmoid_lut_optimized", |b| {
        b.iter(|| {
            black_box(ScrollLut::generate(
                black_box(CurvePreset::Sigmoid),
                black_box(2.5),
            ))
        });
    });

    c.bench_function("generate_exponential_lut_optimized", |b| {
        b.iter(|| {
            black_box(ScrollLut::generate(
                black_box(CurvePreset::Exponential),
                black_box(2.5),
            ))
        });
    });

    // Direct O(1) bin lookup and sub-bin interpolation
    let lut = ScrollLut::generate(CurvePreset::Smooth, 2.5);
    c.bench_function("lut_lookup_direct_bin", |b| {
        let mut bin = 10usize;
        b.iter(|| {
            bin = (bin + 1) & 0xFF;
            black_box(lut.lookup(black_box(bin)))
        });
    });

    c.bench_function("lut_lookup_interpolated", |b| {
        let mut rate = 10.25f32;
        b.iter(|| {
            rate += 0.125;
            if rate > 250.0 {
                rate = 10.25;
            }
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "rate clamped to 0..254, bin within 0..255"
            )]
            let bin = rate as usize;
            #[expect(clippy::cast_precision_loss, reason = "bin is <= 255, exact in f32")]
            let frac = rate - (bin as f32);
            let g0 = lut.lookup(bin);
            let g1 = lut.lookup(bin + 1);
            #[expect(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                reason = "gain interpolation within fixed-point bounds"
            )]
            let interpolated = g0 + (((g1 - g0) as f32) * frac) as i32;
            black_box(interpolated)
        });
    });
}

#[cfg(target_os = "macos")]
#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    fn mach_absolute_time() -> u64;
}

fn bench_isolated_acceleration_cycle(c: &mut Criterion) {
    c.bench_function("isolated_acceleration_and_scaling_pipeline_ns", |b| {
        let mut engine = ScrollAccelerationEngine::default();
        let v_lut = ScrollLut::generate(CurvePreset::Smooth, 2.5);
        let h_lut = ScrollLut::generate(CurvePreset::None, 1.0);
        let mut step = 0u64;

        b.iter(|| {
            step += 1;
            #[cfg(target_os = "macos")]
            // SAFETY: mach_absolute_time is a standard Darwin libSystem timer function.
            let _t_start = unsafe { mach_absolute_time() };
            let at = Instant::now();

            let (h_gain, v_gain) = engine.compute_gains_fixed(
                black_box(0.0),
                black_box(1.0),
                black_box(at),
                black_box(CurvePreset::Smooth),
                black_box(&v_lut),
                black_box(CurvePreset::None),
                black_box(&h_lut),
            );
            let gain_y = openlogi_core::math::fixed_to_f32(v_gain);
            let scaled_y = 1.0 * gain_y;

            #[cfg(target_os = "macos")]
            // SAFETY: mach_absolute_time is a standard Darwin libSystem timer function.
            let _t_end = unsafe { mach_absolute_time() };
            black_box((h_gain, scaled_y))
        });
    });
}

#[expect(
    clippy::unwrap_used,
    reason = "static valid sensitivity value in benchmark fixture"
)]
fn bench_preferences_load(c: &mut Criterion) {
    let prefs = std::sync::Arc::new(
        openlogi_agent_core::runtime::scroll::ScrollPreferences::new(
            true,
            openlogi_core::config::VerticalScrollSensitivity::try_new(5).unwrap(),
        ),
    );
    c.bench_function("scroll_preferences_atomic_load", |b| {
        b.iter(|| {
            let loaded = prefs.load();
            black_box(loaded.vertical_baseline)
        });
    });

    let settings = AppSettings {
        vertical_curve_preset: CurvePreset::Smooth,
        horizontal_curve_preset: CurvePreset::Linear,
        vertical_multiplier: 2.5,
        horizontal_multiplier: 2.0,
        ..AppSettings::default()
    };
    c.bench_function("lut_regeneration_after_preference_change", |b| {
        b.iter(|| {
            prefs.publish_app_settings(black_box(&settings));
        });
    });
}

criterion_group!(
    benches,
    bench_gain_at_rate,
    bench_compute_gains,
    bench_lut_generation,
    bench_isolated_acceleration_cycle,
    bench_preferences_load,
);
criterion_main!(benches);
