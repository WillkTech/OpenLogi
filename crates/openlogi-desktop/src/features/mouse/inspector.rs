//! Fixed binding inspector for the Buttons workspace.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, Role,
    StatefulInteractiveElement as _, Styled, div, prelude::FluentBuilder as _, px, rgb, svg,
};
use gpui_base::Button as BaseButton;
use gpui_base::Disableable as _;
use gpui_component::{
    Icon, IconName, Selectable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::InputState,
    scroll::ScrollableElement as _,
    v_flex,
};
use openlogi_core::binding::{Action, ButtonId, GestureDirection, default_binding};

use super::hotspots::MouseControlId;
use super::picker::{EditFn, GESTURE_BUTTON_ICON, PickFn, action_icon_path, editor_section};
use super::thumbwheel::ThumbwheelPreset;
use super::view::MouseModelView;
use crate::state::AppState;
use crate::ui::action::localized_action_label;
use crate::ui::action_library::{ActionLibrary, NewShortcutFn, OpenPowerUserFn};
use crate::ui::components::{MenuRow, control_button};
use crate::ui::theme::{self, ACCENT_BLUE, Palette, Typography as _};

pub(super) const INSPECTOR_W: f32 = 328.;

#[derive(Clone, Copy)]
pub(super) struct BindingInspectorData<'a> {
    pub selected: Option<MouseControlId>,
    pub gesture_direction: Option<GestureDirection>,
    pub action_picker_open: bool,
    pub bindings: &'a BTreeMap<ButtonId, Action>,
    pub gesture_maps: &'a BTreeMap<ButtonId, BTreeMap<GestureDirection, Action>>,
    pub editing_app: Option<&'a str>,
    pub overridden: Option<&'a BTreeMap<ButtonId, Action>>,
}

#[derive(Clone, Copy)]
struct ActionPickerContext<'a> {
    open: bool,
    search: &'a Entity<InputState>,
    power_user: &'a crate::ui::power_user::PowerUserEditorState,
    view: &'a Entity<MouseModelView>,
}

pub(super) fn binding_inspector(
    data: BindingInspectorData<'_>,
    action_search: &Entity<InputState>,
    power_user: &crate::ui::power_user::PowerUserEditorState,
    view: &Entity<MouseModelView>,
    cx: &Context<MouseModelView>,
) -> gpui::Div {
    let pal = theme::palette(cx);
    let picker = ActionPickerContext {
        open: data.action_picker_open,
        search: action_search,
        power_user,
        view,
    };
    let body = match data.selected {
        None => empty_inspector(
            data.editing_app,
            data.overridden.map_or(0, BTreeMap::len),
            pal,
        ),
        Some(MouseControlId::ThumbwheelRotation) => thumbwheel_inspector(
            data.bindings,
            data.editing_app,
            data.overridden,
            picker,
            pal,
        ),
        Some(MouseControlId::Button(button)) => button_inspector(button, &data, picker, pal, cx),
    };

    v_flex()
        .w(px(INSPECTOR_W))
        .h_full()
        .min_h_0()
        .flex_shrink_0()
        .border_l_1()
        .border_color(pal.border)
        .bg(pal.panel)
        .child(
            div()
                .id("button-inspector-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .p_4()
                .child(body),
        )
}

fn empty_inspector(app: Option<&str>, override_count: usize, pal: Palette) -> gpui::Div {
    let summary = match (app, override_count) {
        (Some(app), 0) => tr!(
            "No overrides yet. Select a button to customize for %{app}.",
            app => app.to_string()
        ),
        (Some(app), 1) => tr!(
            "%{app} overrides 1 button. Others inherit Default.",
            app => app.to_string()
        ),
        (Some(app), count) => tr!(
            "%{app} overrides %{count} buttons. Others inherit Default.",
            app => app.to_string(),
            count => count.to_string()
        ),
        (None, _) => tr!("Select a button on the device to change what it does."),
    };
    v_flex()
        .gap_3()
        .child(inspector_heading(tr!("Button inspector"), None, pal))
        .child(div().text_body().text_color(pal.text_muted).child(summary))
}

#[expect(
    clippy::too_many_lines,
    reason = "assembles header, gestures, picker, and reset button"
)]
fn button_inspector(
    button: ButtonId,
    data: &BindingInspectorData<'_>,
    picker: ActionPickerContext<'_>,
    pal: Palette,
    cx: &Context<MouseModelView>,
) -> gpui::Div {
    if picker.power_user.is_open() {
        let view_close = picker.view.clone();
        let view_commit = picker.view.clone();
        let view_promote = picker.view.clone();
        let on_promote: crate::ui::power_user::PromoteActionFn =
            Arc::new(move |target, window, cx| {
                view_promote.update(cx, |view, cx| {
                    view.power_user.close();
                    view.close_action_picker();
                    view.open_shortcut_builder_with(None, "", Some(&target), None, window, cx);
                    cx.notify();
                });
            });
        return picker.power_user.render_card_with_promote(
            &tr!(button.label()),
            move |action, _window, cx| {
                AppState::update_bindings(cx, |state| state.commit_binding(button, action));
                view_commit.update(cx, |view, cx| {
                    view.power_user.close();
                    view.close_action_picker();
                    cx.notify();
                });
            },
            move |_window, cx| {
                view_close.update(cx, |view, cx| {
                    view.power_user.close();
                    cx.notify();
                });
            },
            Some(on_promote),
            pal,
            cx,
        );
    }

    let gesture_map = data.gesture_maps.get(&button);
    let overridden = data
        .overridden
        .is_some_and(|overrides| overrides.contains_key(&button));
    if data.editing_app.is_none()
        && let Some(gesture_map) = gesture_map
    {
        return gesture_inspector(button, gesture_map, data.gesture_direction, picker, pal, cx);
    }
    if let Some(app) = data.editing_app
        && !overridden
        && gesture_map.is_some()
    {
        return inherited_gesture_inspector(button, app, picker, pal, cx);
    }

    let action = data
        .bindings
        .get(&button)
        .cloned()
        .unwrap_or_else(|| default_binding(button));
    let status = match (
        data.editing_app,
        overridden,
        action == default_binding(button),
    ) {
        (Some(app), true, _) => tr!("Overridden in %{app}", app => app.to_string()),
        (Some(_), false, _) => tr!("Inherited from Default"),
        (None, _, true) => tr!("Device default"),
        (None, _, false) => tr!("Customized"),
    };

    let can_reset = if data.editing_app.is_some() {
        overridden
    } else {
        action != default_binding(button)
    };

    let (badge_text, is_accent) = match (
        data.editing_app,
        overridden,
        action == default_binding(button),
    ) {
        (Some(_), true, _) => (tr!("App Override"), true),
        (Some(_), false, _) => (tr!("Inherited"), false),
        (None, _, false) => (tr!("Customized"), true),
        (None, _, true) => (tr!("Default"), false),
    };

    let view_reset = picker.view.clone();
    let header = h_flex()
        .items_center()
        .justify_between()
        .child(
            v_flex()
                .gap_1()
                .child(div().text_heading().child(tr!(button.label())))
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child(status),
                ),
        )
        .child(
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .text_caption()
                        .font_weight(FontWeight::MEDIUM)
                        .when(is_accent, |badge| {
                            badge.bg(theme::accent_tint()).text_color(rgb(ACCENT_BLUE))
                        })
                        .when(!is_accent, |badge| {
                            badge.bg(pal.control).text_color(pal.text_muted)
                        })
                        .child(badge_text),
                )
                .child(
                    Button::new("inspector-reset-to-default")
                        .ghost()
                        .compact()
                        .disabled(!can_reset)
                        .label(tr!("Reset to default"))
                        .on_click(move |_event, _window, cx| {
                            AppState::update_bindings(cx, |state| {
                                state.reset_binding(button);
                            });
                            view_reset.update(cx, |view, cx| {
                                view.close_action_picker();
                                cx.notify();
                            });
                        }),
                ),
        );

    let observer = picker.view.clone();
    let on_pick: PickFn = Rc::new(move |action, _window, cx| {
        AppState::update_bindings(cx, |state| state.commit_binding(button, action));
        observer.update(cx, |view, cx| {
            view.close_action_picker();
            cx.notify();
        });
    });

    v_flex()
        .gap_3()
        .child(header)
        .child(current_action_card(&action, picker, pal))
        .when(overridden, |panel| {
            let observer = picker.view.clone();
            panel.child(
                control_button("inspector-use-default")
                    .w_full()
                    .icon(IconName::Undo)
                    .label(tr!("Use the default profile"))
                    .on_click(move |_, _, cx| {
                        AppState::update_bindings(cx, |state| {
                            state.clear_app_binding(button);
                        });
                        observer.update(cx, |view, cx| {
                            view.close_action_picker();
                            cx.notify();
                        });
                    }),
            )
        })
        .when(
            data.editing_app.is_none()
                && (button.is_hidpp_gesture_source() || button.is_os_hook_button()),
            |panel| {
                let observer = picker.view.clone();
                panel.child(
                    control_button("inspector-use-gestures")
                        .w_full()
                        .icon(Icon::empty().path(GESTURE_BUTTON_ICON))
                        .label(tr!("Use gestures"))
                        .on_click(move |_, _, cx| {
                            AppState::update_bindings(cx, |state| {
                                state.commit_gesture_mode(button, true);
                            });
                            observer.update(cx, |view, cx| {
                                view.set_gesture_selected_dir(Some(GestureDirection::Click));
                                cx.notify();
                            });
                        }),
                )
            },
        )
        .when(picker.open, |panel| {
            panel.child(action_library(
                "inspector-action",
                Some(&action),
                picker,
                &on_pick,
                pal,
                cx,
            ))
        })
}

fn inherited_gesture_inspector(
    button: ButtonId,
    app: &str,
    picker: ActionPickerContext<'_>,
    pal: Palette,
    cx: &Context<MouseModelView>,
) -> gpui::Div {
    let observer = picker.view.clone();
    let on_pick: PickFn = Rc::new(move |action, _window, cx| {
        AppState::update_bindings(cx, |state| state.commit_binding(button, action));
        observer.update(cx, |view, cx| {
            view.close_action_picker();
            cx.notify();
        });
    });
    let edit_default = picker.view.clone();
    v_flex()
        .gap_3()
        .child(inspector_heading(
            tr!(button.label()),
            Some(tr!("Inherited from Default")),
            pal,
        ))
        .child(gesture_summary_card(picker, pal))
        .child(div().text_caption().text_color(pal.text_muted).child(tr!(
            "Choosing an action replaces the inherited gestures in %{app}.",
            app => app.to_string()
        )))
        .child(
            Button::new("inspector-edit-default-gestures")
                .small()
                .w_full()
                .label(tr!("Edit Default gestures"))
                .on_click(move |_, _, cx| {
                    AppState::update_bindings(cx, |state| state.set_editing_app(None));
                    edit_default.update(cx, |view, cx| {
                        view.set_gesture_selected_dir(Some(GestureDirection::Click));
                        cx.notify();
                    });
                }),
        )
        .when(picker.open, |panel| {
            panel.child(action_library(
                "inspector-gesture-override",
                None,
                picker,
                &on_pick,
                pal,
                cx,
            ))
        })
}

fn gesture_inspector(
    button: ButtonId,
    gesture_map: &BTreeMap<GestureDirection, Action>,
    selected_direction: Option<GestureDirection>,
    picker: ActionPickerContext<'_>,
    pal: Palette,
    cx: &Context<MouseModelView>,
) -> gpui::Div {
    let direction = selected_direction.unwrap_or(GestureDirection::Click);
    let current = gesture_action(gesture_map, button, direction);

    if picker.power_user.is_open() {
        let view_close = picker.view.clone();
        let view_commit = picker.view.clone();
        let view_promote = picker.view.clone();
        let target_label = format!("{} · {}", tr!(button.label()), direction.label());
        let on_promote: crate::ui::power_user::PromoteActionFn =
            Arc::new(move |target, window, cx| {
                view_promote.update(cx, |view, cx| {
                    view.power_user.close();
                    view.close_action_picker();
                    view.open_shortcut_builder_with(None, "", Some(&target), None, window, cx);
                    cx.notify();
                });
            });
        return picker.power_user.render_card_with_promote(
            &target_label,
            move |action, _window, cx| {
                AppState::update_bindings(cx, |state| {
                    state.commit_gesture_binding(button, direction, action);
                });
                view_commit.update(cx, |view, cx| {
                    view.power_user.close();
                    view.close_action_picker();
                    cx.notify();
                });
            },
            move |_window, cx| {
                view_close.update(cx, |view, cx| {
                    view.power_user.close();
                    cx.notify();
                });
            },
            Some(on_promote),
            pal,
            cx,
        );
    }

    let observer = picker.view.clone();
    let on_pick: PickFn = Rc::new(move |action, _window, cx| {
        AppState::update_bindings(cx, |state| {
            state.commit_gesture_binding(button, direction, action);
        });
        observer.update(cx, |view, cx| {
            view.close_action_picker();
            cx.notify();
        });
    });
    let turn_off = picker.view.clone();

    v_flex()
        .gap_3()
        .child(inspector_heading(
            tr!(button.label()),
            Some(tr!("5 directions")),
            pal,
        ))
        .child(gesture_directions(
            direction,
            gesture_map,
            button,
            picker.view,
            pal,
        ))
        .child(current_action_card(&current, picker, pal))
        .child(
            control_button("inspector-single-action")
                .w_full()
                .label(tr!("Use a single action"))
                .on_click(move |_, _, cx| {
                    AppState::update_bindings(cx, |state| {
                        state.commit_gesture_mode(button, false);
                    });
                    turn_off.update(cx, |view, cx| {
                        view.set_gesture_selected_dir(None);
                        cx.notify();
                    });
                }),
        )
        .when(picker.open, |panel| {
            panel.child(action_library(
                "inspector-gesture-action",
                Some(&current),
                picker,
                &on_pick,
                pal,
                cx,
            ))
        })
}

fn gesture_directions(
    active: GestureDirection,
    gesture_map: &BTreeMap<GestureDirection, Action>,
    button: ButtonId,
    view: &Entity<MouseModelView>,
    pal: Palette,
) -> impl IntoElement {
    v_flex()
        .gap_1()
        .child(editor_section(tr!("Direction"), pal))
        .children(
            GestureDirection::ALL
                .into_iter()
                .enumerate()
                .map(|(index, direction)| {
                    let selected = direction == active;
                    let action = gesture_action(gesture_map, button, direction);
                    let view = view.clone();
                    MenuRow::new(("inspector-direction", index))
                        .selected(selected)
                        .role(Role::Button)
                        .child(
                            v_flex()
                                .min_w_0()
                                .child(div().text_body().child(format!(
                                    "{}  {}",
                                    direction.glyph(),
                                    tr!(direction.label())
                                )))
                                .child(
                                    div()
                                        .truncate()
                                        .text_caption()
                                        .text_color(pal.text_muted)
                                        .child(localized_action_label(&action)),
                                ),
                        )
                        .when(selected, |row| {
                            row.child(
                                Icon::new(IconName::Check)
                                    .size_3()
                                    .text_color(rgb(ACCENT_BLUE)),
                            )
                        })
                        .on_click(move |_, _, cx| {
                            view.update(cx, |view, cx| {
                                view.set_gesture_selected_dir(Some(direction));
                                cx.notify();
                            });
                        })
                }),
        )
}

fn thumbwheel_inspector(
    bindings: &BTreeMap<ButtonId, Action>,
    editing_app: Option<&str>,
    overridden: Option<&BTreeMap<ButtonId, Action>>,
    picker: ActionPickerContext<'_>,
    pal: Palette,
) -> gpui::Div {
    let backward = bindings
        .get(&ButtonId::ThumbwheelScrollDown)
        .cloned()
        .unwrap_or_else(|| default_binding(ButtonId::ThumbwheelScrollDown));
    let forward = bindings
        .get(&ButtonId::ThumbwheelScrollUp)
        .cloned()
        .unwrap_or_else(|| default_binding(ButtonId::ThumbwheelScrollUp));
    let current = ThumbwheelPreset::recognize(&backward, &forward);
    let is_overridden = overridden.is_some_and(|overrides| {
        overrides.contains_key(&ButtonId::ThumbwheelScrollDown)
            || overrides.contains_key(&ButtonId::ThumbwheelScrollUp)
    });
    let status = match (editing_app, is_overridden) {
        (Some(app), true) => tr!("Overridden in %{app}", app => app.to_string()),
        (Some(_), false) => tr!("Inherited from Default"),
        (None, _) => tr!("Default profile"),
    };
    let current_label = current.map_or_else(|| tr!("Custom"), |preset| tr!(preset.label()));
    let current_icon = current.map_or("action-icons/chevrons-right.svg", ThumbwheelPreset::icon);
    let observer = picker.view.clone();

    v_flex()
        .gap_3()
        .child(inspector_heading(tr!("Thumb Wheel"), Some(status), pal))
        .child(selection_card(
            "inspector-current-thumbwheel-preset",
            tr!("Preset"),
            current_icon,
            current_label,
            picker,
            pal,
        ))
        .when(picker.open, |panel| {
            panel.child(
                v_flex()
                    .gap_1()
                    .child(editor_section(tr!("Preset"), pal))
                    .children(ThumbwheelPreset::ALL.into_iter().enumerate().map(
                        |(index, preset)| {
                            let selected = current == Some(preset);
                            let observer = observer.clone();
                            MenuRow::new(("inspector-thumbwheel", index))
                                .selected(selected)
                                .role(Role::Button)
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            svg()
                                                .path(preset.icon())
                                                .size_4()
                                                .text_color(pal.text_muted),
                                        )
                                        .child(div().child(tr!(preset.label()))),
                                )
                                .when(selected, |row| {
                                    row.child(
                                        Icon::new(IconName::Check)
                                            .size_3()
                                            .text_color(rgb(ACCENT_BLUE)),
                                    )
                                })
                                .on_click(move |_, _, cx| {
                                    AppState::update_bindings(cx, |state| {
                                        state.commit_thumbwheel_preset(preset);
                                    });
                                    observer.update(cx, |view, cx| {
                                        view.close_action_picker();
                                        cx.notify();
                                    });
                                })
                        },
                    )),
            )
        })
        .when(is_overridden, |panel| {
            let observer = picker.view.clone();
            panel.child(
                Button::new("inspector-thumbwheel-use-default")
                    .small()
                    .w_full()
                    .icon(IconName::Undo)
                    .label(tr!("Use the default profile"))
                    .on_click(move |_, _, cx| {
                        AppState::update_bindings(cx, |state| {
                            state.clear_app_thumbwheel();
                        });
                        observer.update(cx, |view, cx| {
                            view.close_action_picker();
                            cx.notify();
                        });
                    }),
            )
        })
}

fn inspector_heading(
    title: gpui::SharedString,
    status: Option<gpui::SharedString>,
    pal: Palette,
) -> impl IntoElement {
    v_flex()
        .gap_1()
        .child(div().text_heading().child(title))
        .children(status.map(|status| {
            div()
                .text_caption()
                .text_color(pal.text_muted)
                .child(status)
        }))
}

fn current_action_card(
    action: &Action,
    picker: ActionPickerContext<'_>,
    pal: Palette,
) -> impl IntoElement {
    selection_card(
        "inspector-current-action",
        tr!("Current action"),
        action_icon_path(action),
        localized_action_label(action),
        picker,
        pal,
    )
}

fn gesture_summary_card(picker: ActionPickerContext<'_>, pal: Palette) -> impl IntoElement {
    selection_card(
        "inspector-current-gesture-summary",
        tr!("Current action"),
        GESTURE_BUTTON_ICON,
        tr!("5 directions"),
        picker,
        pal,
    )
}

fn selection_card(
    id: &'static str,
    caption: gpui::SharedString,
    icon: &'static str,
    value: gpui::SharedString,
    picker: ActionPickerContext<'_>,
    pal: Palette,
) -> impl IntoElement {
    let toggle = picker.view.clone();
    let search = picker.search.clone();
    let opening = !picker.open;
    let accessible_label = value.clone();
    BaseButton::new(id)
        .accessibility_label(accessible_label)
        .aria_expanded(picker.open)
        .flex()
        .flex_col()
        .gap_2()
        .rounded(pal.control_radius)
        .border_1()
        .border_color(pal.border)
        .bg(pal.control)
        .p_3()
        .cursor_pointer()
        .hover(move |card| card.bg(pal.control_hover))
        .focus_visible(move |card| card.bg(pal.control_hover).border_color(rgb(ACCENT_BLUE)))
        .child(
            div()
                .text_caption()
                .text_color(pal.text_muted)
                .child(caption),
        )
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    h_flex()
                        .min_w_0()
                        .items_center()
                        .gap_2()
                        .child(
                            svg()
                                .path(icon)
                                .size_4()
                                .flex_none()
                                .text_color(pal.text_muted),
                        )
                        .child(div().min_w_0().truncate().text_body().child(value)),
                )
                .child(
                    svg()
                        .path(if picker.open {
                            "action-icons/chevrons-up.svg"
                        } else {
                            "action-icons/chevrons-down.svg"
                        })
                        .size_3()
                        .flex_none()
                        .text_color(pal.text_muted),
                ),
        )
        .on_click(move |_, window, cx| {
            if opening {
                search.update(cx, |search, cx| search.set_value("", window, cx));
            }
            toggle.update(cx, |view, cx| {
                view.toggle_action_picker();
                cx.notify();
            });
        })
}

fn action_library(
    id_prefix: &'static str,
    current: Option<&Action>,
    picker: ActionPickerContext<'_>,
    on_pick: &PickFn,
    pal: Palette,
    cx: &Context<MouseModelView>,
) -> impl IntoElement {
    let query = picker.search.read(cx).value();
    let view_edit = picker.view.clone();
    let on_edit: EditFn = Rc::new(move |entry, window, cx| {
        view_edit.update(cx, |view, cx| {
            view.open_shortcut_builder_with(
                Some(entry.id),
                &entry.name,
                Some(&entry.target),
                entry.icon,
                window,
                cx,
            );
            cx.notify();
        });
    });
    let view_new = picker.view.clone();
    let on_new: NewShortcutFn = Rc::new(move |window, cx| {
        view_new.update(cx, |view, cx| {
            view.open_shortcut_builder(window, cx);
            cx.notify();
        });
    });

    let view_pw = picker.view.clone();
    let current_action = current.cloned();
    let on_open_power_user: OpenPowerUserFn = Rc::new(move |kind, window, cx| {
        view_pw.update(cx, |view, cx| {
            view.power_user
                .open(kind, current_action.as_ref(), window, cx);
            cx.notify();
        });
    });

    ActionLibrary::new(id_prefix, on_pick)
        .current(current)
        .query(&query)
        .on_edit(Some(&on_edit))
        .on_new_shortcut(Some(on_new))
        .on_open_power_user(Some(on_open_power_user))
        .search_state(Some(picker.search))
        .render(pal, cx)
}

fn gesture_action(
    gesture_map: &BTreeMap<GestureDirection, Action>,
    button: ButtonId,
    direction: GestureDirection,
) -> Action {
    gesture_map.get(&direction).cloned().unwrap_or_else(|| {
        if direction == GestureDirection::Click {
            default_binding(button)
        } else {
            Action::None
        }
    })
}
