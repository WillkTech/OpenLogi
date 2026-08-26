//! Scroll acceleration curve visualization widget.

use gpui::{
    App, Bounds, IntoElement, ParentElement, PathBuilder, RenderOnce, Styled, Window, canvas,
    point, px,
};

use crate::ui::theme::{self, Palette};

/// Lightweight curve preview component for mouse-wheel acceleration.
///
/// X axis: Wheel speed (Slow → Fast, 0 to 40 ticks/sec).
/// Y axis: Effective scroll gain multiplier (1.0× to the multiplier, with 3.0×
/// as the floor so low-multiplier curves keep a readable scale).
#[derive(IntoElement)]
pub(crate) struct AccelerationCurveGraph {
    enabled: bool,
    acceleration_factor: f64,
    max_gain: f64,
}

impl AccelerationCurveGraph {
    pub(crate) fn new(enabled: bool, acceleration_factor: f64, max_gain: f64) -> Self {
        Self {
            enabled,
            acceleration_factor,
            max_gain,
        }
    }
}

const SAMPLE_COUNT: usize = 30;

fn compute_curve_samples(
    enabled: bool,
    factor: f64,
    max_gain: f64,
) -> [(f32, f32); SAMPLE_COUNT + 1] {
    let min_y_val = 1.0f64;
    let max_y_val = max_gain.max(3.0);
    let range = max_y_val - min_y_val;

    let mut samples = [(0.0f32, 0.0f32); SAMPLE_COUNT + 1];

    for (i, sample) in samples.iter_mut().enumerate() {
        let t = f64::from(i) / f64::from(SAMPLE_COUNT);
        let rate = t * 40.0;
        let gain = openlogi_core::math::gain_at_rate(rate, enabled, factor, max_gain);
        let norm_x = norm_f32(t);
        let norm_y = norm_f32((gain - min_y_val) / range);
        *sample = (norm_x, norm_y);
    }

    samples
}

impl RenderOnce for AccelerationCurveGraph {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pal = theme::palette(cx);
        let enabled = self.enabled;
        let factor = self.acceleration_factor;
        let max_gain = self.max_gain;
        let samples = compute_curve_samples(enabled, factor, max_gain);

        gpui::div()
            .flex_none()
            .w(px(220.))
            .h(px(90.))
            .rounded_md()
            .bg(pal.panel)
            .border_1()
            .border_color(pal.border)
            .p_2()
            .child(
                canvas(
                    move |_, _, _| (enabled, max_gain, samples, pal),
                    move |bounds, (enabled, max_gain, samples, pal), window, _| {
                        paint_graph(bounds, enabled, max_gain, samples, pal, window);
                    },
                )
                .size_full(),
            )
    }
}

fn paint_graph(
    bounds: Bounds<gpui::Pixels>,
    enabled: bool,
    max_gain: f64,
    samples: [(f32, f32); SAMPLE_COUNT + 1],
    pal: Palette,
    window: &mut Window,
) {
    let origin_x = f32::from(bounds.origin.x);
    let origin_y = f32::from(bounds.origin.y);
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    if width <= 10. || height <= 10. {
        return;
    }

    // Padding inside the graph canvas
    let pad_left = 28.;
    let pad_bottom = 16.;
    let pad_top = 8.;
    let pad_right = 8.;

    let graph_left = origin_x + pad_left;
    let graph_bottom = origin_y + height - pad_bottom;
    let graph_right = origin_x + width - pad_right;
    let graph_top = origin_y + pad_top;

    let plot_w = (graph_right - graph_left).max(1.);
    let plot_h = (graph_bottom - graph_top).max(1.);

    // Dynamic vertical scale: 1.0× at bottom, the multiplier at top (never
    // below 3.0×, so small-multiplier curves don't fill the whole plot).
    let min_y_val = 1.0f64;
    let max_y_val = max_gain.max(3.0);

    let line_color = if enabled {
        theme::accent()
    } else {
        pal.text_muted.opacity(0.4)
    };

    let grid_color = pal.border.opacity(0.5);

    // 1. Draw 1.0× Baseline (bottom grid line)
    let y_1x = graph_bottom;
    let mut baseline_path = PathBuilder::stroke(px(1.));
    baseline_path.move_to(point(px(graph_left), px(y_1x)));
    baseline_path.line_to(point(px(graph_right), px(y_1x)));
    if let Ok(p) = baseline_path.build() {
        window.paint_path(p, grid_color);
    }

    // 2. Draw Max Gain reference line (faded top threshold line)
    let norm_max = norm_f32((max_gain - min_y_val) / (max_y_val - min_y_val));
    let y_max = graph_bottom - norm_max * plot_h;

    let mut max_line_path = PathBuilder::stroke(px(1.));
    max_line_path.move_to(point(px(graph_left), px(y_max)));
    max_line_path.line_to(point(px(graph_right), px(y_max)));
    if let Ok(p) = max_line_path.build() {
        window.paint_path(p, pal.text_muted.opacity(0.3));
    }

    // 3. Draw the Acceleration Curve
    let mut curve_path = PathBuilder::stroke(px(2.));

    for (i, &(norm_x, norm_y)) in samples.iter().enumerate() {
        let pt_x = graph_left + norm_x * plot_w;
        let pt_y = graph_bottom - norm_y * plot_h;

        let pt = point(px(pt_x), px(pt_y));
        if i == 0 {
            curve_path.move_to(pt);
        } else {
            curve_path.line_to(pt);
        }
    }

    if let Ok(p) = curve_path.build() {
        window.paint_path(p, line_color);
    }
}

/// Map a normalized 0..=1 curve coordinate onto the f32 geometry space. The
/// inputs are bounded by construction, so the narrowing conversion loses no
/// meaningful precision.
#[expect(
    clippy::cast_possible_truncation,
    reason = "normalized coordinates are bounded to 0.0..=1.0"
)]
fn norm_f32(value: f64) -> f32 {
    value as f32
}
