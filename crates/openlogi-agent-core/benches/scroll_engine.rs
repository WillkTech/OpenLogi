use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use openlogi_agent_core::runtime::scroll::{AxisAccelerationParams, ScrollAccelerationEngine};
use openlogi_core::config::CurvePreset;
use openlogi_core::math::{ScrollLut, gain_at_rate_preset};

fn bench_gain_at_rate(c: &mut Criterion) {
    c.bench_function("gain_at_rate_smooth", |b| {
        let mut rate = 5.0f32;
        b.iter(|| {
            rate += 0.1;
            if rate > 50.0 {
                rate = 5.0;
            }
            gain_at_rate_preset(rate, CurvePreset::Smooth, 2.5)
        });
    });

    c.bench_function("gain_at_rate_sigmoid", |b| {
        let mut rate = 5.0f32;
        b.iter(|| {
            rate += 0.1;
            if rate > 50.0 {
                rate = 5.0;
            }
            gain_at_rate_preset(rate, CurvePreset::Sigmoid, 2.5)
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
            engine.compute_gains(0.5, 1.0, at, v_params, h_params)
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
            engine.compute_gains(0.5, 1.0, at, v_params, h_params)
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
            engine.compute_gains_fixed(
                0.5,
                1.0,
                at,
                CurvePreset::Smooth,
                &v_lut,
                CurvePreset::Linear,
                &h_lut,
            )
        });
    });
}

fn bench_lut_generation(c: &mut Criterion) {
    c.bench_function("lut_generation_smooth", |b| {
        b.iter(|| ScrollLut::generate(CurvePreset::Smooth, 2.5));
    });

    c.bench_function("lut_generation_sigmoid", |b| {
        b.iter(|| ScrollLut::generate(CurvePreset::Sigmoid, 2.5));
    });

    c.bench_function("lut_generation_exponential", |b| {
        b.iter(|| ScrollLut::generate(CurvePreset::Exponential, 2.5));
    });
}

criterion_group!(
    benches,
    bench_gain_at_rate,
    bench_compute_gains,
    bench_lut_generation
);
criterion_main!(benches);
