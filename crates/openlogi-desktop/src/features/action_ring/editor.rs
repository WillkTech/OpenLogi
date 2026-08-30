//! Categorized action, shortcut, path, and icon editor for one ring slot.

use gpui::{
    Entity, InteractiveElement, IntoElement, ParentElement, Role, ScrollHandle,
    StatefulInteractiveElement as _, Styled, div, prelude::FluentBuilder as _, px, rgb, svg,
};
use gpui_component::{
    Icon, IconName, Selectable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::InputState,
    scroll::ScrollableElement as _,
    v_flex,
};
use openlogi_core::binding::{
    Action, ActionRingEntry, ActionRingIcon, ActionRingSlot, ApplicationTarget, Category, KeyCombo,
    RingAction,
};

use super::ActionRingPanel;
use super::action_icons::action_icon_path;
use crate::features::mouse::picker::{EditFn, editor_section};
use crate::state::{AppState, DeviceRecord, StateEvent};
use crate::ui::action::localized_action_label;
use crate::ui::action_composer::toggle_modifier_in_text;
use crate::ui::components::{MenuRow, control_input};
use crate::ui::theme::{self, Palette, Typography as _};

#[expect(
    clippy::too_many_arguments,
    reason = "renders full action ring library panel"
)]
pub(super) fn action_library(
    slot: ActionRingSlot,
    current: Option<&ActionRingEntry>,
    application_input: &Entity<InputState>,
    shortcut_input: &Entity<InputState>,
    library_scroll: &ScrollHandle,
    view: &Entity<ActionRingPanel>,
    pal: Palette,
    cx: &gpui::App,
) -> impl IntoElement {
    let view_edit = view.clone();
    let on_edit: EditFn = std::rc::Rc::new(move |entry, window, cx| {
        view_edit.update(cx, |panel, cx| {
            panel.open_shortcut_builder_with(
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
    let current_action = current.map(ActionRingEntry::action).cloned();
    let current_label = current_action
        .as_ref()
        .map_or_else(|| tr!("Empty slot"), localized_action_label);

    v_flex()
        .flex_1()
        .min_w(px(280.0))
        .max_w(px(320.0))
        .h(px(420.0))
        .overflow_hidden()
        .rounded(pal.card_radius)
        .border_1()
        .border_color(pal.border)
        .bg(pal.panel)
        .child(
            v_flex()
                .flex_none()
                .gap_1()
                .border_b_1()
                .border_color(pal.border)
                .p_3()
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(div().text_subheading().child(tr!("Actions Ring")))
                        .child(
                            Button::new("ring-clear-slot")
                                .compact()
                                .label(tr!("Clear slot"))
                                .on_click(move |_, _, cx| commit_slot(slot, None, cx)),
                        ),
                )
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child(current_label),
                ),
        )
        .child(action_rows_scroller(
            v_flex()
                .p_1p5()
                .when_some(current_action.as_ref(), |library, action| {
                    library.child(icon_editor(
                        slot,
                        action,
                        current.and_then(ActionRingEntry::custom_icon),
                        pal,
                    ))
                })
                .child(composer_section(
                    slot,
                    shortcut_input,
                    application_input,
                    pal,
                ))
                .children(action_sections(
                    slot,
                    current_action.as_ref(),
                    Some(&on_edit),
                    pal,
                    cx,
                )),
            library_scroll,
        ))
}

fn action_rows_scroller(content: impl IntoElement, scroll: &ScrollHandle) -> impl IntoElement {
    div()
        .id("ring-action-library")
        .flex_1()
        .min_h_0()
        .track_scroll(scroll)
        .overflow_y_scroll()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
        .vertical_scrollbar(scroll)
        .child(content)
}

fn icon_editor(
    slot: ActionRingSlot,
    action: &Action,
    current: Option<ActionRingIcon>,
    pal: Palette,
) -> impl IntoElement {
    let default_path = action_icon_path(action);
    let default = icon_button(
        "ring-default-icon",
        default_path,
        tr!("Use action icon"),
        current.is_none(),
        pal,
    )
    .on_click(move |_, _, cx| commit_icon(slot, None, cx));

    v_flex()
        .gap_1()
        .child(editor_section(tr!("Icon"), pal))
        .child(
            h_flex().flex_wrap().gap_1().child(default).children(
                ActionRingIcon::ALL
                    .into_iter()
                    .enumerate()
                    .map(move |(index, icon)| {
                        icon_button(
                            ("ring-custom-icon", index),
                            icon.asset_path(),
                            rust_i18n::t!(icon.label()),
                            current == Some(icon),
                            pal,
                        )
                        .on_click(move |_, _, cx| commit_icon(slot, Some(icon), cx))
                    }),
            ),
        )
}

fn icon_button(
    id: impl Into<gpui::ElementId>,
    path: &'static str,
    label: impl Into<gpui::SharedString>,
    selected: bool,
    pal: Palette,
) -> Button {
    Button::new(id)
        .size(px(32.0))
        .rounded(px(16.0))
        .selected(selected)
        .icon(Icon::empty().path(path).text_color(pal.text_muted))
        .tooltip(label)
}

fn composer_section(
    slot: ActionRingSlot,
    shortcut_input: &Entity<InputState>,
    path_input: &Entity<InputState>,
    pal: Palette,
) -> impl IntoElement {
    let shortcut_sub = shortcut_input.clone();
    let path_sub = path_input.clone();
    v_flex()
        .gap_1p5()
        .child(editor_section(tr!("Custom Action Target"), pal))
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child(tr!("Modifiers")),
                )
                .child(
                    h_flex().flex_wrap().gap_1().children(
                        ["Win", "Cmd", "Ctrl", "Alt", "Shift", "Fn"]
                            .into_iter()
                            .map(|mod_name| {
                                let input = shortcut_input.clone();
                                Button::new(format!("ring-mod-{mod_name}"))
                                    .compact()
                                    .label(mod_name)
                                    .on_click(move |_, window, cx| {
                                        let text = input.read(cx).value();
                                        let new_text = toggle_modifier_in_text(&text, mod_name);
                                        input.update(cx, |state, cx| {
                                            state.set_value(new_text, window, cx);
                                        });
                                    })
                            }),
                    ),
                ),
        )
        .child(
            h_flex()
                .gap_1p5()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(control_input(shortcut_input).cleanable(true)),
                )
                .child(
                    Button::new("ring-add-shortcut")
                        .compact()
                        .label(tr!("Add Shortcut"))
                        .on_click(move |_, _, cx| {
                            let shortcut = shortcut_sub.read(cx).value().to_string();
                            if let Ok(combo) = shortcut.parse::<KeyCombo>() {
                                commit_action(slot, Action::CustomShortcut(combo), cx);
                            }
                        }),
                ),
        )
        .child(
            h_flex()
                .gap_1p5()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(control_input(path_input).cleanable(true)),
                )
                .child(
                    Button::new("ring-add-path")
                        .compact()
                        .label(tr!("Add Path"))
                        .on_click(move |_, _, cx| {
                            let path = path_sub.read(cx).value().to_string();
                            if let Ok(target) = ApplicationTarget::new(path, "") {
                                commit_action(slot, Action::OpenApplication(target), cx);
                            }
                        }),
                ),
        )
}

#[expect(
    clippy::too_many_lines,
    reason = "iterates all action catalog sections"
)]
fn action_sections(
    slot: ActionRingSlot,
    current: Option<&Action>,
    on_edit: Option<&EditFn>,
    pal: Palette,
    cx: &gpui::App,
) -> impl Iterator<Item = impl IntoElement> {
    let mut index = 0usize;
    let on_edit = on_edit.cloned();
    ring_catalog(Some(cx))
        .into_iter()
        .map(move |(category, actions)| {
            let on_edit = on_edit.clone();
            v_flex()
                .child(editor_section(rust_i18n::t!(category.label()), pal))
                .children(actions.into_iter().map(|action| {
                    let selected = current == Some(&action);
                    let action_to_commit = action.clone();
                    let label = tr!(action.label());
                    let icon_path = action_icon_path(&action);
                    let is_custom = matches!(&action, Action::SavedShortcut { .. });
                    let custom_entry = if let Action::SavedShortcut {
                        id,
                        name,
                        target,
                        icon,
                    } = &action
                    {
                        Some(openlogi_core::config::CustomShortcutEntry {
                            id: id.clone(),
                            name: name.clone(),
                            target: target.clone(),
                            icon: *icon,
                        })
                    } else {
                        None
                    };
                    let row_index = index;
                    index += 1;
                    let on_edit = on_edit.clone();
                    MenuRow::new(("ring-action", row_index))
                        .role(Role::MenuItem)
                        .aria_label(label.clone())
                        .selected(selected)
                        .child(
                            h_flex()
                                .flex_1()
                                .items_center()
                                .justify_between()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            svg()
                                                .path(icon_path)
                                                .size_4()
                                                .flex_none()
                                                .text_color(pal.text_muted),
                                        )
                                        .child(label),
                                )
                                .when_some(custom_entry, |parent, entry| {
                                    let id = entry.id.clone();
                                    let edit_entry = entry.clone();
                                    parent.child(
                                        h_flex()
                                            .items_center()
                                            .gap_1()
                                            .when_some(on_edit, |controls, on_edit| {
                                                controls.child(
                                                    Button::new(("ring-edit-shortcut", row_index))
                                                        .icon(
                                                            Icon::empty()
                                                                .path("action-icons/pencil.svg")
                                                                .text_color(pal.text_muted),
                                                        )
                                                        .compact()
                                                        .ghost()
                                                        .tooltip("Edit shortcut")
                                                        .on_click(move |_event, window, cx| {
                                                            (on_edit)(
                                                                edit_entry.clone(),
                                                                window,
                                                                cx,
                                                            );
                                                        }),
                                                )
                                            })
                                            .child(
                                                Button::new(("ring-delete-shortcut", row_index))
                                                    .icon(IconName::Close)
                                                    .compact()
                                                    .ghost()
                                                    .tooltip("Delete shortcut")
                                                    .on_click(move |_event, _, cx| {
                                                        let id = id.clone();
                                                        AppState::update(cx, |state, _cx| {
                                                            state.delete_custom_shortcut(&id);
                                                        });
                                                    }),
                                            ),
                                    )
                                }),
                        )
                        .when(selected && !is_custom, |row| {
                            row.child(
                                Icon::new(IconName::Check)
                                    .size_3()
                                    .text_color(rgb(theme::ACCENT_BLUE)),
                            )
                        })
                        .on_click(move |_, _, cx| {
                            commit_action(slot, action_to_commit.clone(), cx);
                        })
                }))
        })
}

fn ring_catalog(cx: Option<&gpui::App>) -> Vec<(Category, Vec<Action>)> {
    let mut sections: Vec<(Category, Vec<Action>)> = Vec::new();

    if let Some(state) = cx.and_then(AppState::try_read) {
        let custom_actions: Vec<Action> = state
            .custom_shortcuts()
            .iter()
            .map(|entry| Action::SavedShortcut {
                id: entry.id.clone(),
                name: entry.name.clone(),
                target: entry.target.clone(),
                icon: entry.icon,
            })
            .filter(|action| RingAction::new(action.clone()).is_ok())
            .collect();
        if !custom_actions.is_empty() {
            sections.push((Category::CustomShortcuts, custom_actions));
        }
    }

    for action in Action::catalog() {
        if RingAction::new(action.clone()).is_err() {
            continue;
        }
        let category = action.category();
        if let Some((_, actions)) = sections
            .iter_mut()
            .find(|(candidate, _)| *candidate == category)
        {
            actions.push(action);
        } else {
            sections.push((category, vec![action]));
        }
    }
    sections
}

fn commit_action(slot: ActionRingSlot, action: Action, cx: &mut gpui::App) {
    let Ok(action) = RingAction::new(action) else {
        return;
    };
    commit_slot(slot, Some(action), cx);
}

fn commit_slot(slot: ActionRingSlot, action: Option<RingAction>, cx: &mut gpui::App) {
    AppState::update(cx, |state, cx| {
        let key = state.current_record().map(DeviceRecord::device_key);
        state.commit_action_ring_slot(slot, action);
        if let Some(key) = key {
            cx.emit(StateEvent::BindingsChanged(key));
        }
    });
}

fn commit_icon(slot: ActionRingSlot, icon: Option<ActionRingIcon>, cx: &mut gpui::App) {
    AppState::update(cx, |state, cx| {
        let key = state.current_record().map(DeviceRecord::device_key);
        state.commit_action_ring_icon(slot, icon);
        if let Some(key) = key {
            cx.emit(StateEvent::BindingsChanged(key));
        }
    });
}

#[cfg(test)]
mod tests {
    use gpui::{
        Context, PlatformInput, Render, ScrollDelta, ScrollHandle, ScrollWheelEvent,
        TestAppContext, Window, point, size,
    };

    use super::*;

    struct NestedScrollView {
        page_scroll: ScrollHandle,
        sidebar_scroll: ScrollHandle,
    }

    impl Render for NestedScrollView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("page-scroll")
                .size_full()
                .track_scroll(&self.page_scroll)
                .overflow_y_scroll()
                .child(
                    v_flex()
                        .h(px(300.0))
                        .child(v_flex().h(px(100.0)).child(action_rows_scroller(
                            div().h(px(240.0)),
                            &self.sidebar_scroll,
                        )))
                        .child(div().h(px(200.0))),
                )
        }
    }

    #[gpui::test]
    fn sidebar_scroll_does_not_move_the_page(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let page_scroll = ScrollHandle::new();
        let sidebar_scroll = ScrollHandle::new();
        let window = cx.open_window(size(px(160.0), px(120.0)), {
            let page_scroll = page_scroll.clone();
            let sidebar_scroll = sidebar_scroll.clone();
            move |_, _| NestedScrollView {
                page_scroll,
                sidebar_scroll,
            }
        });
        cx.run_until_parked();

        window
            .update(cx, |_, window, cx| {
                window.dispatch_event(
                    PlatformInput::ScrollWheel(ScrollWheelEvent {
                        position: point(px(80.0), px(50.0)),
                        delta: ScrollDelta::Pixels(point(px(0.0), px(-20.0))),
                        ..Default::default()
                    }),
                    cx,
                );
            })
            .unwrap();

        assert_eq!(sidebar_scroll.offset().y, px(-20.0));
        assert_eq!(page_scroll.offset().y, px(0.0));
    }

    #[test]
    fn ring_catalog_is_categorized_and_excludes_invalid_actions() {
        let sections = ring_catalog(None);
        assert!(
            sections
                .iter()
                .any(|(category, _)| *category == Category::Navigation)
        );
        let actions = sections
            .into_iter()
            .flat_map(|(_, actions)| actions)
            .collect::<Vec<_>>();
        assert!(actions.contains(&Action::MissionControl));
        assert!(!actions.contains(&Action::None));
        assert!(!actions.contains(&Action::ShowActionsRing));
    }
}
