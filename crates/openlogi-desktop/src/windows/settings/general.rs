//! General settings page.

mod curve;

use curve::AccelerationCurveGraph;
use gpui_component::Disableable as _;
use gpui_component::button::{Button, ButtonVariants as _};

use super::{
    App, AppState, Entity, FluentBuilder, IconName, IndexPath, ParentElement, SelectItem,
    SelectState, SettingField, SettingGroup, SettingItem, SettingPage, Slider, SliderState,
    StateEvent, Styled, ThumbwheelSensitivity, VerticalScrollSensitivity, div, h_flex, px, theme,
    v_flex,
};
use crate::ui::components::control_select;
use crate::ui::theme::Typography as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AccelerationPreset {
    Precise,
    Balanced,
    Fast,
    Custom,
}

impl AccelerationPreset {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Precise => "Precise",
            Self::Balanced => "Balanced",
            Self::Fast => "Fast",
            Self::Custom => "Custom",
        }
    }

    pub(super) fn values(self, is_vertical: bool) -> Option<(f64, f64)> {
        match self {
            Self::Precise => Some((0.5, 1.5)),
            Self::Balanced => {
                if is_vertical {
                    Some((1.0, 2.5))
                } else {
                    Some((1.0, 2.0))
                }
            }
            Self::Fast => Some((1.5, 3.0)),
            Self::Custom => None,
        }
    }

    pub(super) fn from_values(factor: f64, max_gain: f64, is_vertical: bool) -> Self {
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
        AccelerationPreset::Precise => 0,
        AccelerationPreset::Balanced => 1,
        AccelerationPreset::Fast => 2,
        AccelerationPreset::Custom => 3,
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
                SettingField::switch(
                    |cx| AppState::try_read(cx).is_some_and(|s| s.app_settings().smooth_scroll),
                    |enabled, cx| {
                        AppState::update(cx, move |state, cx| {
                            state.set_smooth_scroll(enabled);
                            cx.emit(StateEvent::SettingsChanged);
                        });
                    },
                ),
            )
            .description(tr!(
                "Animate traditional mouse-wheel input while leaving trackpad scrolling unchanged."
            )),
        )
        .item(
            SettingItem::new(
                tr!("Vertical Scroll Sensitivity"),
                SettingField::render(move |_, _, cx| {
                    vertical_scroll_sensitivity_field(&vertical_scroll_sensitivity_slider, cx)
                }),
            )
            .description(tr!(
                "Controls baseline scrolling distance per wheel notch. (Distinct from Acceleration, which responds to wheel turn speed)."
            )),
        )
        .item(
            SettingItem::new(
                tr!("Vertical Scroll Acceleration"),
                SettingField::switch(
                    |cx| {
                        AppState::try_read(cx)
                            .is_some_and(|s| s.app_settings().vertical_acceleration_enabled)
                    },
                    |enabled, cx| {
                        AppState::update(cx, move |state, cx| {
                            state.set_vertical_acceleration_enabled(enabled);
                            cx.emit(StateEvent::SettingsChanged);
                        });
                    },
                ),
            )
            .description(tr!(
                "Progressively increases vertical scroll distance during fast wheel movement."
            )),
        )
        .item(
            SettingItem::new(
                tr!("Vertical Acceleration Controls"),
                SettingField::render({
                    let v_accel_slider = v_accel_slider.clone();
                    let v_max_gain_slider = v_max_gain_slider.clone();
                    let v_preset_select = v_preset_select.clone();
                    move |_, _, cx| {
                        acceleration_controls_block(
                            true,
                            &v_accel_slider,
                            &v_max_gain_slider,
                            &v_preset_select,
                            cx,
                        )
                    }
                }),
            )
            .description(tr!(
                "Strength controls how aggressively speed increases. Maximum Speed limits the top multiplier."
            )),
        )
        .item(
            SettingItem::new(
                tr!("Horizontal Scroll Acceleration"),
                SettingField::switch(
                    |cx| {
                        AppState::try_read(cx)
                            .is_some_and(|s| s.app_settings().horizontal_acceleration_enabled)
                    },
                    |enabled, cx| {
                        AppState::update(cx, move |state, cx| {
                            state.set_horizontal_acceleration_enabled(enabled);
                            cx.emit(StateEvent::SettingsChanged);
                        });
                    },
                ),
            )
            .description(tr!(
                "Progressively increases horizontal scroll distance during fast wheel movement."
            )),
        )
        .item(
            SettingItem::new(
                tr!("Horizontal Acceleration Controls"),
                SettingField::render({
                    let h_accel_slider = h_accel_slider.clone();
                    let h_max_gain_slider = h_max_gain_slider.clone();
                    let h_preset_select = h_preset_select.clone();
                    move |_, _, cx| {
                        acceleration_controls_block(
                            false,
                            &h_accel_slider,
                            &h_max_gain_slider,
                            &h_preset_select,
                            cx,
                        )
                    }
                }),
            )
            .description(tr!(
                "Strength controls how aggressively speed increases. Maximum Speed limits the top multiplier."
            )),
        )
        .item(
            SettingItem::new(
                tr!("Thumb Wheel Sensitivity"),
                SettingField::render(move |_, _, cx| {
                    thumbwheel_sensitivity_field(&thumbwheel_sensitivity_slider, cx)
                }),
            )
            .description(tr!(
                "Scales the thumb wheel's horizontal scroll speed and how readily custom wheel actions trigger."
            )),
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
            )
            .description(if cfg!(target_os = "macos") {
                tr!("Automatically start OpenLogi when you log in to macOS.")
            } else {
                tr!("Automatically start OpenLogi when you log in.")
            }),
        );

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let group = group.item(
        SettingItem::new(
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
        )
        .description(if cfg!(target_os = "macos") {
            tr!("Keep OpenLogi's icon in the menu bar. When off, it stays in the Dock instead.")
        } else {
            tr!(
                "Keep OpenLogi's icon in the taskbar notification area. Takes effect the next time the background agent starts."
            )
        }),
    );

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
    is_vertical: bool,
    accel_slider: &Entity<SliderState>,
    max_gain_slider: &Entity<SliderState>,
    preset_select: &Entity<SelectState<Vec<PresetOption>>>,
    cx: &mut App,
) -> gpui::Div {
    let pal = theme::palette(cx);
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

    let factor_percent = percent_of(factor);
    let graph = AccelerationCurveGraph::new(enabled, factor, max_gain);

    v_flex().gap_3().p_1().child(
        h_flex()
            .items_center()
            .gap_4()
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(120.))
                                    .text_caption()
                                    .text_color(if enabled {
                                        pal.text_primary
                                    } else {
                                        pal.text_muted
                                    })
                                    .child(tr!("Preset")),
                            )
                            .child(
                                div().w(px(160.)).h_6().child(
                                    control_select(preset_select)
                                        .w(px(160.))
                                        .menu_width(px(160.))
                                        .disabled(!enabled),
                                ),
                            ),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(120.))
                                    .text_caption()
                                    .text_color(if enabled {
                                        pal.text_primary
                                    } else {
                                        pal.text_muted
                                    })
                                    .child(tr!("Strength")),
                            )
                            .child(
                                div()
                                    .w(px(160.))
                                    .child(Slider::new(accel_slider).disabled(!enabled)),
                            )
                            .child(
                                div()
                                    .w(px(50.))
                                    .text_caption()
                                    .text_color(pal.text_muted)
                                    .child(format!("{factor_percent}%")),
                            ),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(120.))
                                    .text_caption()
                                    .text_color(if enabled {
                                        pal.text_primary
                                    } else {
                                        pal.text_muted
                                    })
                                    .child(tr!("Maximum Speed")),
                            )
                            .child(
                                div()
                                    .w(px(160.))
                                    .child(Slider::new(max_gain_slider).disabled(!enabled)),
                            )
                            .child(
                                div()
                                    .w(px(50.))
                                    .text_caption()
                                    .text_color(pal.text_muted)
                                    .child(format!("{max_gain:.1}×")),
                            ),
                    )
                    .child(
                        h_flex().justify_end().child(
                            Button::new(if is_vertical {
                                "reset_v_accel"
                            } else {
                                "reset_h_accel"
                            })
                            .ghost()
                            .label(tr!("Reset to default"))
                            .disabled(!enabled)
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
                    ),
            )
            .child(graph),
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
    let value = VerticalScrollSensitivity::from_rounded(slider.read(cx).value().start());
    sensitivity_field(
        slider,
        value.to_string(),
        value == VerticalScrollSensitivity::DEFAULT,
        cx,
    )
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

/// Render the strength multiplier (0.2-2.0, snapped onto the 5% grid) as a
/// whole percent.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "factor is snapped into 0.2..=2.0 before rounding, so the cast is exact"
)]
fn percent_of(factor: f64) -> u32 {
    (factor * 100.0).round() as u32
}
