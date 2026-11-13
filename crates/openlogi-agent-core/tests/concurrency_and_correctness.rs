//! Concurrency, boundary conditions, and correctness validation for OpenLogi 0.8.4.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    use openlogi_agent_core::runtime::scroll::{
        AxisAccelerationParams, ScrollAccelerationEngine, ScrollPreferences,
    };
    use openlogi_core::config::{
        AppSettings, CurvePreset, HorizontalScrollBaseline, VerticalScrollSensitivity,
    };
    use openlogi_core::math::{CurvePreset as CorePreset, ScrollLut, UNITY_GAIN_FIXED};

    #[test]
    fn test_all_13_presets_bit_for_bit_exact() {
        let multipliers = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 10.0];
        for preset in CorePreset::ALL {
            for &m in &multipliers {
                let fast = ScrollLut::generate(preset, m);
                let direct = ScrollLut::generate_direct(preset, m);
                for bin in 0..256 {
                    let fast_val = fast.lookup(bin);
                    let direct_val = direct.lookup(bin);
                    assert_eq!(
                        fast_val, direct_val,
                        "Bit-for-bit mismatch for {preset:?} at bin {bin}, multiplier {m}: fast={fast_val}, direct={direct_val}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_boundary_and_unusual_settings() {
        let unusual_multipliers = [
            0.0,
            -1.0,
            -5.0,
            1.0,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            0.5,
            0.999,
            1.0001,
            100.0,
        ];

        for preset in CorePreset::ALL {
            for &m in &unusual_multipliers {
                let lut = ScrollLut::generate(preset, m);
                for bin in 0..256 {
                    let val = lut.lookup(bin);
                    assert!(
                        val >= UNITY_GAIN_FIXED,
                        "Gain must never fall below unity for {preset:?} with multiplier {m}, got {val}"
                    );
                    assert!(
                        val.is_positive(),
                        "Gain must always be positive for {preset:?} with multiplier {m}, got {val}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_simultaneous_preference_changes_and_continuous_scrolling() {
        let initial_settings = AppSettings {
            vertical_curve_preset: CurvePreset::Smooth,
            horizontal_curve_preset: CurvePreset::Linear,
            vertical_multiplier: 2.5,
            horizontal_multiplier: 2.0,
            vertical_scroll_baseline: VerticalScrollSensitivity::DEFAULT,
            ..AppSettings::default()
        };
        let preferences = Arc::new(ScrollPreferences::from_app_settings(&initial_settings));
        let stop = Arc::new(AtomicBool::new(false));

        // Thread 1: Continuous scroll reader simulating high-frequency scrolling (50,000 ticks)
        let prefs_reader = Arc::clone(&preferences);
        let stop_reader = Arc::clone(&stop);
        let reader_handle = thread::spawn(move || {
            let mut engine = ScrollAccelerationEngine::default();
            let base_instant = Instant::now();
            let mut ticks = 0u64;

            while !stop_reader.load(Ordering::Relaxed) && ticks < 50_000 {
                ticks += 1;
                let prefs = prefs_reader.load();
                let at = base_instant + Duration::from_micros(ticks * 20);
                let (gx, gy) = engine.compute_gains_fixed(
                    0.0,
                    1.0,
                    at,
                    prefs.vertical_preset,
                    &prefs.vertical_lut,
                    prefs.horizontal_preset,
                    &prefs.horizontal_lut,
                );
                assert!(gx >= UNITY_GAIN_FIXED);
                assert!(gy >= UNITY_GAIN_FIXED);
            }
            ticks
        });

        // Thread 2: Rapid concurrent preference updates simulating user dragging settings slider
        let prefs_writer = Arc::clone(&preferences);
        let stop_writer = Arc::clone(&stop);
        let writer_handle = thread::spawn(move || {
            let presets = [
                CurvePreset::Smooth,
                CurvePreset::Sigmoid,
                CurvePreset::Linear,
                CurvePreset::Exponential,
                CurvePreset::None,
            ];
            let mut updates = 0u64;
            for i in 0..500 {
                if stop_writer.load(Ordering::Relaxed) {
                    break;
                }
                #[expect(clippy::cast_precision_loss, reason = "i < 500")]
                let mult = 1.0 + ((i % 10) as f32) * 0.4;
                let preset = presets[i % presets.len()];
                let new_settings = AppSettings {
                    vertical_curve_preset: preset,
                    horizontal_curve_preset: preset,
                    vertical_multiplier: mult,
                    horizontal_multiplier: mult,
                    vertical_scroll_baseline: VerticalScrollSensitivity::DEFAULT,
                    horizontal_scroll_baseline: HorizontalScrollBaseline::DEFAULT,
                    ..AppSettings::default()
                };
                prefs_writer.publish_app_settings(&new_settings);
                updates += 1;
                thread::yield_now();
            }
            updates
        });

        let updates_done = writer_handle.join().expect("writer must not panic");
        stop.store(true, Ordering::Relaxed);
        let ticks_done = reader_handle.join().expect("reader must not panic");

        println!("\nConcurrency Stress Test Succeeded:");
        println!(
            "Processed {ticks_done} scroll events concurrently with {updates_done} preference snapshot swaps."
        );
        assert!(ticks_done > 1000);
        assert!(updates_done > 50);
    }

    #[test]
    fn test_axes_independence_under_rapid_reversals() {
        let mut engine = ScrollAccelerationEngine::default();
        let start = Instant::now();
        let v_params = AxisAccelerationParams {
            preset: CurvePreset::Smooth,
            multiplier: 3.0,
        };
        let h_params = AxisAccelerationParams {
            preset: CurvePreset::Linear,
            multiplier: 2.5,
        };

        // Scroll vertical positive for 10 ticks -> vertical gain builds up
        for i in 0..10 {
            let t = start + Duration::from_millis(i * 20);
            let (_gx, gy) = engine.compute_gains(0.0, 1.0, t, v_params, h_params);
            if i > 2 {
                assert!(gy > 1.0);
            }
        }

        // Now reverse vertical direction -> vertical resets to 1.0, horizontal starts fresh
        let t_rev = start + Duration::from_millis(10 * 20);
        let (gx_fresh, gy_reset) = engine.compute_gains(1.0, -1.0, t_rev, v_params, h_params);
        assert_eq!(gy_reset, 1.0, "reversed axis must reset to 1.0");
        assert_eq!(gx_fresh, 1.0, "orthogonal axis first tick must be 1.0");
    }
}
