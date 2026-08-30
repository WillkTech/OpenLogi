//! General settings page.

mod curve;

use curve::AccelerationCurveGraph;
use gpui::SharedString;
use gpui_component::button::{Button, ButtonVariants as _};
use openlogi_core::config::{CurvePreset, HorizontalScrollBaseline, VerticalScrollBaseline};

use super::{
    App, AppState, Entity, FluentBuilder, IconName, IndexPath, ParentElement, SelectItem,
    SelectState, SettingField, SettingGroup, SettingItem, SettingPage, Slider, SliderState,
    StateEvent, Styled, div, h_flex, px, theme, v_flex,
};
use crate::ui::components::{control_select, hint_icon, labeled_row};
use crate::ui::theme::Typography as _;

#[derive(Clone, Copy)]
pub(super) struct PresetOption {
    pub(super) preset: CurvePreset,
}

impl SelectItem for PresetOption {
    type Value = CurvePreset;

    fn title(&self) -> gpui::SharedString {
        gpui::SharedString::from(rust_i18n::t!(self.preset.label()).into_owned())
    }

    fn value(&self) -> &Self::Value {
        &self.preset
    }
}

pub(super) fn preset_options() -> Vec<PresetOption> {
    CurvePreset::ALL
        .into_iter()
        .map(|preset| PresetOption { preset })
        .collect()
}

pub(super) fn selected_preset_index(preset: CurvePreset) -> IndexPath {
    let row = CurvePreset::ALL
        .iter()
        .position(|&p| p == preset)
        .unwrap_or(0);
    IndexPath::default().row(row)
}

// One row per control, each needing its own handle — a bundle struct would
// only move the count, not remove it.
pub(super) fn general_page(
    vertical_scroll_sensitivity_slider: &Entity<SliderState>,
    thumbwheel_sensitivity_slider: &Entity<SliderState>,
    v_multiplier_slider: &Entity<SliderState>,
    h_multiplier_slider: &Entity<SliderState>,
    v_preset_select: &Entity<SelectState<Vec<PresetOption>>>,
    h_preset_select: &Entity<SelectState<Vec<PresetOption>>>,
) -> SettingPage {
    // The `render` closures are `'static`, so each field keeps its own owned
    // handle; the by-reference parameters keep the caller's intent explicit.
    let vertical_scroll_sensitivity_slider = vertical_scroll_sensitivity_slider.clone();
    let thumbwheel_sensitivity_slider = thumbwheel_sensitivity_slider.clone();
    let group = SettingGroup::new()
        .item(SettingItem::new(
            tr!("Vertical Scroll Baseline"),
            SettingField::render(move |_, _, cx| {
                vertical_scroll_sensitivity_field(&vertical_scroll_sensitivity_slider, cx)
            }),
        ))
        .item(SettingItem::render({
            let v_multiplier_slider = v_multiplier_slider.clone();
            let v_preset_select = v_preset_select.clone();
            move |_, _, cx| {
                acceleration_controls_block(
                    tr!("Vertical Scroll Acceleration"),
                    true,
                    &v_multiplier_slider,
                    &v_preset_select,
                    cx,
                )
            }
        }))
        .item(SettingItem::new(
            tr!("Horizontal Scroll Baseline"),
            SettingField::render(move |_, _, cx| {
                thumbwheel_sensitivity_field(&thumbwheel_sensitivity_slider, cx)
            }),
        ))
        .item(SettingItem::render({
            let h_multiplier_slider = h_multiplier_slider.clone();
            let h_preset_select = h_preset_select.clone();
            move |_, _, cx| {
                acceleration_controls_block(
                    tr!("Horizontal Scroll Acceleration"),
                    false,
                    &h_multiplier_slider,
                    &h_preset_select,
                    cx,
                )
            }
        }))
        .item(SettingItem::new(
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
        ));

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

fn acceleration_controls_block(
    heading: SharedString,
    is_vertical: bool,
    multiplier_slider: &Entity<SliderState>,
    preset_select: &Entity<SelectState<Vec<PresetOption>>>,
    cx: &mut App,
) -> gpui::Div {
    let (preset, multiplier) = AppState::try_read(cx).map_or((CurvePreset::Smooth, 2.5), |s| {
        let app = s.app_settings();
        if is_vertical {
            (
                app.effective_vertical_preset(),
                app.effective_vertical_multiplier(),
            )
        } else {
            (
                app.effective_horizontal_preset(),
                app.effective_horizontal_multiplier(),
            )
        }
    });

    let pal = theme::palette(cx);
    let graph = AccelerationCurveGraph::new(preset, multiplier);
    let enabled = preset.is_enabled();

    h_flex()
        .w_full()
        .items_start()
        .gap_4()
        .p_1()
        .pr_4()
        // First column: heading + curve below it (visible when preset is enabled)
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
                .child(labeled_row(
                    tr!("Preset"),
                    Some((
                        "preset-hint",
                        tr!("Scroll acceleration curve preset or smoothing algorithm."),
                        pal.text_muted,
                    )),
                    |row| {
                        row.child(
                            div().w(px(160.)).h_6().child(
                                control_select(preset_select)
                                    .w(px(160.))
                                    .menu_width(px(160.)),
                            ),
                        )
                    },
                ))
                .when(enabled, |col| {
                    col.child(labeled_row(
                        tr!("Multiplier"),
                        Some((
                            "multiplier-hint",
                            tr!("Maximum acceleration gain factor applied at peak scrolling speed."),
                            pal.text_muted,
                        )),
                        |row| {
                            row.child(div().w(px(160.)).child(Slider::new(multiplier_slider)))
                                .child(
                                    div()
                                        .w(px(50.))
                                        .text_caption()
                                        .text_color(pal.text_muted)
                                        .child(format!("{multiplier:.1}×")),
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
    let value = HorizontalScrollBaseline::from_rounded(slider.read(cx).value().start());
    sensitivity_field(
        slider,
        value.to_string(),
        value == HorizontalScrollBaseline::DEFAULT,
        cx,
    )
}

fn vertical_scroll_sensitivity_field(slider: &Entity<SliderState>, cx: &mut App) -> gpui::Div {
    let pal = theme::palette(cx);
    let value = VerticalScrollBaseline::from_rounded(slider.read(cx).value().start());
    let field = sensitivity_field(
        slider,
        value.to_string(),
        value == VerticalScrollBaseline::DEFAULT,
        cx,
    );
    field.child(hint_icon(
        "sensitivity-hint",
        tr!("Controls baseline scrolling distance per vertical wheel notch."),
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
