use criterion::{Criterion, criterion_group, criterion_main};
use openlogi_core::math::gain_at_rate;

fn bench_gain_at_rate(c: &mut Criterion) {
    c.bench_function("gain_at_rate", |b| {
        let mut rate = 5.0;
        b.iter(|| {
            rate += 0.1;
            if rate > 50.0 {
                rate = 5.0;
            }
            gain_at_rate(rate, true, 1.0, 2.5)
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

criterion_group!(benches, bench_gain_at_rate, bench_quantize);
criterion_main!(benches);
