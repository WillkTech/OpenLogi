use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use openlogi_agent_core::runtime::scroll::{AxisAccelerationParams, ScrollAccelerationEngine};
use openlogi_core::math::gain_at_rate;

fn bench_gain_at_rate(c: &mut Criterion) {
    c.bench_function("gain_at_rate_f32", |b| {
        let mut rate = 5.0f32;
        b.iter(|| {
            rate += 0.1;
            if rate > 50.0 {
                rate = 5.0;
            }
            gain_at_rate(rate, true, 1.0, 2.5)
        });
    });
}

fn bench_compute_gains(c: &mut Criterion) {
    c.bench_function("compute_gains_f32", |b| {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        let v_params = AxisAccelerationParams {
            enabled: true,
            factor: 1.0,
            max_gain: 2.5,
        };
        let h_params = AxisAccelerationParams {
            enabled: true,
            factor: 1.0,
            max_gain: 2.0,
        };
        let mut step = 0u64;

        b.iter(|| {
            step += 1;
            let at = start + Duration::from_micros(step * 10);
            engine.compute_gains(0.5, 1.0, at, v_params, h_params)
        });
    });
}

fn bench_quantize(c: &mut Criterion) {
    c.bench_function("quantize_u8", |b| {
        let mut val = 0.5f32;
        b.iter(|| {
            val += 0.7;
            if val > 150.0 {
                val = 0.5;
            }
            openlogi_core::math::quantize::<u8>(val, 1.0, 100.0, 1.0)
        });
    });
}

criterion_group!(
    benches,
    bench_gain_at_rate,
    bench_compute_gains,
    bench_quantize
);
criterion_main!(benches);
