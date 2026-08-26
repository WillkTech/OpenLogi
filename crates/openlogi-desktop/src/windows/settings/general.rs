//! General settings page.

mod curve;

use curve::AccelerationCurveGraph;
use gpui::{InteractiveElement as _, SharedString, StatefulInteractiveElement as _};
use gpui_component::{
    Selectable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
};

use super::{
    App, AppState, Entity, FluentBuilder, IconName, IndexPath, ParentElement, SelectItem,
    SelectState, SettingField, SettingGroup, SettingItem, SettingPage, Slider, SliderState,
    StateEvent, Styled, ThumbwheelSensitivity, VerticalScrollSensitivity, div, h_flex, px, theme,
    v_flex,
};
use crate::ui::components::{Toggle, control_select, hint_icon, labeled_row};
use crate::ui::theme::Typography as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AccelerationPreset {
    /// Acceleration off — maps onto `*_acceleration_enabled = false`, so no
    /// config/wire type is needed.
    None,
    Precise,
    Balanced,
    Fast,
    Custom,
}

impl AccelerationPreset {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Precise => "Precise",
            Self::Balanced => "Balanced",
            Self::Fast => "Fast",
            Self::Custom => "Custom",
        }
    }

    /// The `(strength, multiplier)` pair a preset writes; `None` carries none
    /// (it only flips the axis' enabled flag) and `Custom` keeps whatever the
    /// sliders hold.
    pub(super) fn values(self, is_vertical: bool) -> Option<(f64, f64)> {
        match self {
            Self::None | Self::Custom => None,
            Self::Precise => Some((0.5, 1.5)),
            Self::Balanced => {
                if is_vertical {
                    Some((1.0, 2.5))
                } else {
                    Some((1.0, 2.0))
                }
            }
            Self::Fast => Some((1.5, 3.0)),
        }
    }

    /// Which preset a stored configuration corresponds to. A disabled axis is
    /// `None` regardless of its stored pair, so re-enabling via any preset
    /// hydrates that preset's values instead of reviving stale ones.
    pub(super) fn from_values(
        enabled: bool,
        factor: f64,
        max_gain: f64,
        is_vertical: bool,
    ) -> Self {
        if !enabled {
            return Self::None;
        }
        if is_vertical {
            if (factor - 0.5).abs() < 0.01 && (max_gain - 1.5).abs() < 0.01 {
                Self::Precise
            } else if (factor - 1.0).abs() < 0.01 && (max_gain - 2.5).abs() < 0.01 {
                Self::Balanced
            } else if (factor - 1.5).abs() < 0.01 && (max_gain - 3.0).abs() < 0.01 {
                Self::Fast
            } else {
                Self::Custom
            }
        } else if (factor - 0.5).abs() < 0.01 && (max_gain - 1.5).abs() < 0.01 {
            Self::Precise
        } else if (factor - 1.0).abs() < 0.01 && (max_gain - 2.0).abs() < 0.01 {
            Self::Balanced
        } else if (factor - 1.5).abs() < 0.01 && (max_gain - 3.0).abs() < 0.01 {
            Self::Fast
        } else {
            Self::Custom
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PresetOption {
    pub(super) preset: AccelerationPreset,
}

impl SelectItem for PresetOption {
    type Value = AccelerationPreset;

    fn title(&self) -> gpui::SharedString {
        gpui::SharedString::from(rust_i18n::t!(self.preset.label()).into_owned())
    }

    fn value(&self) -> &Self::Value {
        &self.preset
    }
}

pub(super) fn preset_options() -> Vec<PresetOption> {
    vec![
        PresetOption {
            preset: AccelerationPreset::None,
        },
        PresetOption {
            preset: AccelerationPreset::Precise,
        },
        PresetOption {
            preset: AccelerationPreset::Balanced,
        },
        PresetOption {
            preset: AccelerationPreset::Fast,
        },
        PresetOption {
            preset: AccelerationPreset::Custom,
        },
    ]
}

pub(super) fn selected_preset_index(preset: AccelerationPreset) -> IndexPath {
    let row = match preset {
        AccelerationPreset::None => 0,
        AccelerationPreset::Precise => 1,
        AccelerationPreset::Balanced => 2,
        AccelerationPreset::Fast => 3,
        AccelerationPreset::Custom => 4,
    };
    IndexPath::default().row(row)
}

// One row per control, each needing its own handle — a bundle struct would
// only move the count, not remove it.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one Entity handle per General-page control; the page builder is linear by nature"
)]
pub(super) fn general_page(
    vertical_scroll_sensitivity_slider: &Entity<SliderState>,
    thumbwheel_sensitivity_slider: &Entity<SliderState>,
    v_accel_slider: &Entity<SliderState>,
    v_max_gain_slider: &Entity<SliderState>,
    h_accel_slider: &Entity<SliderState>,
    h_max_gain_slider: &Entity<SliderState>,
    v_preset_select: &Entity<SelectState<Vec<PresetOption>>>,
    h_preset_select: &Entity<SelectState<Vec<PresetOption>>>,
) -> SettingPage {
    // The `render` closures are `'static`, so each field keeps its own owned
    // handle; the by-reference parameters keep the caller's intent explicit.
    let vertical_scroll_sensitivity_slider = vertical_scroll_sensitivity_slider.clone();
    let thumbwheel_sensitivity_slider = thumbwheel_sensitivity_slider.clone();
    let group = SettingGroup::new()
        .item(
            SettingItem::new(
                tr!("Smooth scrolling"),
                SettingField::render(move |_, _, cx| {
                    let enabled =
                        AppState::try_read(cx).is_some_and(|s| s.app_settings().smooth_scroll);
                    h_flex()
                        .gap_2()
                        .child(
                            Toggle::new("general-smooth-scroll")
                                .selected(enabled)
                                .on_change(|enabled, _, cx| {
                                    AppState::update(cx, move |state, cx| {
                                        state.set_smooth_scroll(*enabled);
                                        cx.emit(StateEvent::SettingsChanged);
                                    });
                                }),
                        )
                        .child(hint_icon(
                            "smooth-scroll-hint",
                            tr!(
                                "Animate traditional mouse-wheel input while leaving trackpad scrolling unchanged."
                            ),
                            theme::palette(cx).text_muted,
                        ))
                }),
            ),
        )
        .item(
            SettingItem::new(
                tr!("Vertical Scroll Sensitivity"),
                SettingField::render(move |_, _, cx| {
                    vertical_scroll_sensitivity_field(&vertical_scroll_sensitivity_slider, cx)
                }),
            ),
        )
        .item(
            SettingItem::render({
                let v_accel_slider = v_accel_slider.clone();
                let v_max_gain_slider = v_max_gain_slider.clone();
                let v_preset_select = v_preset_select.clone();
                move |_, _, cx| {
                    acceleration_controls_block(
                        tr!("Vertical Scroll Acceleration"),
                        true,
                        &v_accel_slider,
                        &v_max_gain_slider,
                        &v_preset_select,
                        cx,
                    )
                }
            }),
        )
        .item(
            SettingItem::render({
                let h_accel_slider = h_accel_slider.clone();
                let h_max_gain_slider = h_max_gain_slider.clone();
                let h_preset_select = h_preset_select.clone();
                move |_, _, cx| {
                    acceleration_controls_block(
                        tr!("Horizontal Scroll Acceleration"),
                        false,
                        &h_accel_slider,
                        &h_max_gain_slider,
                        &h_preset_select,
                        cx,
                    )
                }
            }),
        )
        .item(
            SettingItem::new(
                tr!("Thumb Wheel Sensitivity"),
                SettingField::render(move |_, _, cx| {
                    thumbwheel_sensitivity_field(&thumbwheel_sensitivity_slider, cx)
                }),
            ),
        )
        .item(
            SettingItem::new(
                tr!("Launch at login"),
                SettingField::switch(
                    |cx| AppState::try_read(cx).is_some_and(|s| s.app_settings().launch_at_login),
                    |enabled, cx| {
                        AppState::update(cx, move |state, cx| {
                            state.set_launch_at_login(enabled);
                            cx.emit(StateEvent::SettingsChanged);
                        });
                    },
                ),
            ),
        );

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let group = group.item(SettingItem::new(
        if cfg!(target_os = "macos") {
            tr!("Show in menu bar")
        } else {
            tr!("Show in the notification area")
        },
        SettingField::switch(
            |cx| AppState::try_read(cx).is_some_and(|s| s.app_settings().show_in_menu_bar),
            |enabled, cx| {
                AppState::update(cx, move |state, cx| {
                    state.set_show_in_menu_bar(enabled);
                    cx.emit(StateEvent::SettingsChanged);
                });
            },
        ),
    ));

    SettingPage::new(tr!("General"))
        .icon(IconName::Settings)
        .resettable(false)
        .group(group)
}


#[expect(
    clippy::too_many_lines,
    reason = "one labelled row per control keeps the block flat and scannable"
)]
fn acceleration_controls_block(
    heading: SharedString,
    is_vertical: bool,
    accel_slider: &Entity<SliderState>,
    max_gain_slider: &Entity<SliderState>,
    preset_select: &Entity<SelectState<Vec<PresetOption>>>,
    cx: &mut App,
) -> gpui::Div {
    // Two-column layout: the first column carries the section heading with the
    // curve graph beneath it (the heading row otherwise wastes its full width),
    // the second holds the controls. The Preset select is always rendered — it
    // is the only way back from "None" — while the sliders, graph, and Reset
    // button collapse when acceleration is off.
    let (enabled, factor, max_gain) = AppState::try_read(cx).map_or((false, 1.0, 2.5), |s| {
        let app = s.app_settings();
        if is_vertical {
            (
                app.vertical_acceleration_enabled,
                app.vertical_acceleration,
                app.vertical_max_gain,
            )
        } else {
            (
                app.horizontal_acceleration_enabled,
                app.horizontal_acceleration,
                app.horizontal_max_gain,
            )
        }
    });

    let pal = theme::palette(cx);
    let factor_percent = percent_of(factor);
    let graph = AccelerationCurveGraph::new(enabled, factor, max_gain);

    h_flex()
        .w_full()
        .items_start()
        .gap_4()
        .p_1()
        .pr_4()
        // First column: heading + curve below it, bounded so it shrinks with
        // the window instead of clipping against its right border.
        .child(
            v_flex()
                .flex_1()
                .min_w(px(220.))
                .max_w(px(360.))
                .gap_2()
                .child(div().text_body().child(heading))
                .when(enabled, |col| col.child(graph)),
        )
        // Second column: the control rows.
        .child(
            v_flex()
                .gap_2()
                .child(labeled_row(tr!("Preset"), None, |row| {
                    row.child(
                        div().w(px(160.)).h_6().child(
                            control_select(preset_select)
                                .w(px(160.))
                                .menu_width(px(160.)),
                        ),
                    )
                }))
                .when(enabled, |col| {
                    col.child(labeled_row(
                        tr!("Acceleration"),
                        Some((
                            "accel-hint",
                            tr!("Acceleration controls how aggressively speed increases."),
                            pal.text_muted,
                        )),
                        |row| {
                            row.child(div().w(px(160.)).child(Slider::new(accel_slider)))
                                .child(
                                    div()
                                        .w(px(50.))
                                        .text_caption()
                                        .text_color(pal.text_muted)
                                        .child(format!("{factor_percent}%")),
                                )
                        },
                    ))
                    .child(labeled_row(
                        tr!("Multiplier"),
                        Some((
                            "multiplier-hint",
                            tr!("Acceleration multiplier limits the top speed factor."),
                            pal.text_muted,
                        )),
                        |row| {
                            row.child(div().w(px(160.)).child(Slider::new(max_gain_slider)))
                                .child(
                                    div()
                                        .w(px(50.))
                                        .text_caption()
                                        .text_color(pal.text_muted)
                                        .child(format!("{max_gain:.1}×")),
                                )
                        },
                    ))
                    .child(
                        h_flex().justify_end().pr_2().child(
                            Button::new(if is_vertical {
                                "reset_v_accel"
                            } else {
                                "reset_h_accel"
                            })
                            .ghost()
                            .label(tr!("Reset to default"))
                            .on_click(move |_, _window, cx| {
                                AppState::update(cx, |state, cx| {
                                    if is_vertical {
                                        state.reset_vertical_acceleration();
                                    } else {
                                        state.reset_horizontal_acceleration();
                                    }
                                    cx.emit(StateEvent::SettingsChanged);
                                });
                            }),
                        ),
                    )
                }),
        )
}

fn thumbwheel_sensitivity_field(slider: &Entity<SliderState>, cx: &mut App) -> gpui::Div {
    let value = ThumbwheelSensitivity::from_rounded(slider.read(cx).value().start());
    sensitivity_field(
        slider,
        value.to_string(),
        value == ThumbwheelSensitivity::DEFAULT,
        cx,
    )
}

fn vertical_scroll_sensitivity_field(slider: &Entity<SliderState>, cx: &mut App) -> gpui::Div {
    let pal = theme::palette(cx);
    let value = VerticalScrollSensitivity::from_rounded(slider.read(cx).value().start());
    let field = sensitivity_field(
        slider,
        value.to_string(),
        value == VerticalScrollSensitivity::DEFAULT,
        cx,
    );
    field.child(hint_icon(
        "sensitivity-hint",
        tr!("Controls baseline scrolling distance per wheel notch."),
        pal.text_muted,
    ))
}

fn sensitivity_field(
    slider: &Entity<SliderState>,
    value: String,
    is_default: bool,
    cx: &mut App,
) -> gpui::Div {
    let pal = theme::palette(cx);
    v_flex()
        .flex_shrink_0()
        .gap_1()
        .child(
            h_flex()
                .items_center()
                .gap_3()
                .child(div().w(px(180.)).child(Slider::new(slider)))
                .child(
                    div()
                        .w(px(72.))
                        .text_body()
                        .text_color(pal.text_muted)
                        .child(value),
                ),
        )
        .when(is_default, |this| {
            this.child(
                div()
                    .text_caption()
                    .text_color(pal.text_muted)
                    .whitespace_nowrap()
                    .child(format!("({})", rust_i18n::t!("Default"))),
            )
        })
}

/// Render the acceleration strength (0.2-4.0, snapped onto the 5% grid) as a
/// whole percent.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "factor is snapped into 0.2..=4.0 before rounding, so the cast is exact"
)]
fn percent_of(factor: f64) -> u32 {
    (factor * 100.0).round() as u32
}
