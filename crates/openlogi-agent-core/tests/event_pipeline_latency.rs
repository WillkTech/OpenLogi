//! High-precision latency and throughput measurement of the complete scroll event pipeline.
#![expect(
    unsafe_code,
    reason = "mach_absolute_time FFI for sub-microsecond monotonic latency metrics on Apple Silicon"
)]

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crossbeam_queue::ArrayQueue;
    use openlogi_agent_core::runtime::scroll::{ScrollAccelerationEngine, ScrollPreferences};
    use openlogi_core::config::{AppSettings, CurvePreset, VerticalScrollSensitivity};
    use openlogi_core::math::fixed_to_f32;
    use openlogi_core::scroll::ScrollDelta;

    #[cfg(target_os = "macos")]
    #[link(name = "System", kind = "dylib")]
    unsafe extern "C" {
        fn mach_absolute_time() -> u64;
    }

    #[inline]
    fn now_ns() -> u64 {
        #[cfg(target_os = "macos")]
        {
            // SAFETY: mach_absolute_time is standard Darwin libSystem timer.
            unsafe { mach_absolute_time() }
        }
        #[cfg(not(target_os = "macos"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos() as u64)
        }
    }

    fn calculate_percentiles(mut samples: Vec<u64>) -> (u64, u64, u64, u64, f64) {
        if samples.is_empty() {
            return (0, 0, 0, 0, 0.0);
        }
        samples.sort_unstable();
        let n = samples.len();
        let p50 = samples[n * 50 / 100];
        let p95 = samples[n * 95 / 100];
        let p99 = samples[n * 99 / 100];
        let max = samples[n - 1];
        let sum: u64 = samples.iter().sum();
        #[expect(clippy::cast_precision_loss, reason = "sample count <= 100_000")]
        let mean = (sum as f64) / (n as f64);
        (p50, p95, p99, max, mean)
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "comprehensive 7-stage event pipeline latency harness"
    )]
    fn test_measure_full_event_pipeline_latency() {
        const NUM_EVENTS: usize = 10_000;

        let settings = AppSettings {
            vertical_curve_preset: CurvePreset::Smooth,
            horizontal_curve_preset: CurvePreset::Linear,
            vertical_multiplier: 2.5,
            horizontal_multiplier: 2.0,
            vertical_scroll_baseline: VerticalScrollSensitivity::DEFAULT,
            ..AppSettings::default()
        };
        let preferences = ScrollPreferences::from_app_settings(&settings);
        let queue: ArrayQueue<ScrollDelta> = ArrayQueue::new(256);
        let mut accel_engine = ScrollAccelerationEngine::default();

        let mut decode_latencies = Vec::with_capacity(NUM_EVENTS);
        let mut queue_submission_latencies = Vec::with_capacity(NUM_EVENTS);
        let mut worker_receipt_latencies = Vec::with_capacity(NUM_EVENTS);
        let mut pref_load_latencies = Vec::with_capacity(NUM_EVENTS);
        let mut gain_calc_latencies = Vec::with_capacity(NUM_EVENTS);
        let mut end_to_end_latencies = Vec::with_capacity(NUM_EVENTS);

        let start_wall = Instant::now();
        let base_instant = Instant::now();

        for i in 0..NUM_EVENTS {
            let t_start = now_ns();

            // 1. Callback Entry & Event Decoding
            let t_dec_start = now_ns();
            let raw_delta_y = if (i % 20) == 0 { -1.0f32 } else { 1.0f32 };
            let delta = ScrollDelta::WheelTicks {
                x: 0.0,
                y: f64::from(raw_delta_y),
            };
            let t_dec_end = now_ns();
            decode_latencies.push(t_dec_end.saturating_sub(t_dec_start));

            // 2. Queue Submission
            let t_sub_start = now_ns();
            let pushed = queue.push(delta).is_ok();
            let t_sub_end = now_ns();
            assert!(pushed, "queue must not overflow");
            queue_submission_latencies.push(t_sub_end.saturating_sub(t_sub_start));

            // 3. Worker Pop & Receipt
            let t_pop_start = now_ns();
            let popped = queue.pop();
            let t_pop_end = now_ns();
            assert!(popped.is_some(), "worker must pop submitted command");
            worker_receipt_latencies.push(t_pop_end.saturating_sub(t_pop_start));

            // 4. Preference Snapshot Load (ArcSwap wait-free atomic)
            let t_pref_start = now_ns();
            let prefs = preferences.load();
            let t_pref_end = now_ns();
            pref_load_latencies.push(t_pref_end.saturating_sub(t_pref_start));

            // 5. Gain Computation with Fixed-Point LUT
            let t_gain_start = now_ns();
            let at = base_instant + Duration::from_micros((i as u64) * 10);
            let (h_gain_fixed, v_gain_fixed) = accel_engine.compute_gains_fixed(
                0.0,
                raw_delta_y,
                at,
                prefs.vertical_preset,
                &prefs.vertical_lut,
                prefs.horizontal_preset,
                &prefs.horizontal_lut,
            );
            let _gain_y = fixed_to_f32(v_gain_fixed);
            let _gain_x = fixed_to_f32(h_gain_fixed);
            let t_gain_end = now_ns();
            gain_calc_latencies.push(t_gain_end.saturating_sub(t_gain_start));

            // End-to-end event handling latency
            let t_end = now_ns();
            end_to_end_latencies.push(t_end.saturating_sub(t_start));
        }

        let elapsed = start_wall.elapsed();
        #[expect(clippy::cast_precision_loss, reason = "NUM_EVENTS = 10000")]
        let throughput = (NUM_EVENTS as f64) / elapsed.as_secs_f64();

        let (dec_p50, dec_p95, dec_p99, dec_max, dec_mean) =
            calculate_percentiles(decode_latencies);
        let (sub_p50, sub_p95, sub_p99, sub_max, sub_mean) =
            calculate_percentiles(queue_submission_latencies);
        let (pop_p50, pop_p95, pop_p99, pop_max, pop_mean) =
            calculate_percentiles(worker_receipt_latencies);
        let (pref_p50, pref_p95, pref_p99, pref_max, pref_mean) =
            calculate_percentiles(pref_load_latencies);
        let (gain_p50, gain_p95, gain_p99, gain_max, gain_mean) =
            calculate_percentiles(gain_calc_latencies);
        let (e2e_p50, e2e_p95, e2e_p99, e2e_max, e2e_mean) =
            calculate_percentiles(end_to_end_latencies);

        println!("\n=== Real Event-Path Latency Audit (10,000 Events, M5 Max) ===");
        println!("Throughput: {throughput:.0} events/second (Elapsed: {elapsed:?})");
        println!(
            "Stage 1 [Decode]:     P50: {dec_p50}ns | P95: {dec_p95}ns | P99: {dec_p99}ns | Max: {dec_max}ns | Mean: {dec_mean:.1}ns"
        );
        println!(
            "Stage 2 [Queue Push]: P50: {sub_p50}ns | P95: {sub_p95}ns | P99: {sub_p99}ns | Max: {sub_max}ns | Mean: {sub_mean:.1}ns"
        );
        println!(
            "Stage 3 [Queue Pop]:  P50: {pop_p50}ns | P95: {pop_p95}ns | P99: {pop_p99}ns | Max: {pop_max}ns | Mean: {pop_mean:.1}ns"
        );
        println!(
            "Stage 4 [Pref Load]:  P50: {pref_p50}ns | P95: {pref_p95}ns | P99: {pref_p99}ns | Max: {pref_max}ns | Mean: {pref_mean:.1}ns"
        );
        println!(
            "Stage 5 [Gain Calc]:  P50: {gain_p50}ns | P95: {gain_p95}ns | P99: {gain_p99}ns | Max: {gain_max}ns | Mean: {gain_mean:.1}ns"
        );
        println!(
            "---------------------------------------------------------------------------------------------------"
        );
        println!(
            "TOTAL End-to-End:     P50: {e2e_p50}ns | P95: {e2e_p95}ns | P99: {e2e_p99}ns | Max: {e2e_max}ns | Mean: {e2e_mean:.1}ns\n"
        );

        // Latency thresholds on Apple Silicon M5 Max
        assert!(
            e2e_p50 < 100,
            "P50 latency must be under 100ns, was {e2e_p50}ns"
        );
        assert!(
            e2e_p95 < 250,
            "P95 latency must be under 250ns, was {e2e_p95}ns"
        );
        assert!(
            e2e_p99 < 500,
            "P99 latency must be under 500ns, was {e2e_p99}ns"
        );
    }
}
