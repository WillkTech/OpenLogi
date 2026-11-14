//! Categorized action, shortcut, path, and icon editor for one ring slot.

use gpui::{
    Entity, InteractiveElement, IntoElement, ParentElement, ScrollHandle,
    StatefulInteractiveElement as _, Styled, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Disableable as _, Icon, Selectable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    scroll::ScrollableElement as _,
    v_flex,
};
use openlogi_core::binding::{Action, ActionRingEntry, ActionRingIcon, ActionRingSlot, RingAction};

use super::ActionRingPanel;
use crate::features::mouse::picker::{EditFn, PickFn, editor_section};
use crate::state::{AppState, DeviceRecord, StateEvent};
use crate::ui::action::{action_icon_path, localized_action_label};
use crate::ui::action_library::ActionLibrary;
use crate::ui::theme::{Palette, Typography as _};

pub(super) fn action_library(
    slot: ActionRingSlot,
    current: Option<&ActionRingEntry>,
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
    let view_new = view.clone();
    let on_new = move |window: &mut gpui::Window, cx: &mut gpui::App| {
        view_new.update(cx, |panel, cx| {
            panel.open_shortcut_builder(window, cx);
            cx.notify();
        });
    };
    let on_pick: PickFn = std::rc::Rc::new(move |action, _window, cx| {
        commit_action(slot, action, cx);
    });
    let current_action = current.map(ActionRingEntry::action).cloned();
    let current_label = current_action
        .as_ref()
        .map_or_else(|| tr!("Empty slot"), localized_action_label);
    let catalog_rows = ActionLibrary::new("ring-action", &on_pick)
        .current(current_action.as_ref())
        .filter(|action| RingAction::new(action.clone()).is_ok())
        .on_edit(Some(&on_edit))
        .build_rows(pal, cx);

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
        .child(slot_header(slot, current, current_label, pal))
        .child(action_rows_scroller(
            v_flex()
                .p_1p5()
                .gap_1()
                .when_some(current_action.as_ref(), |library, action| {
                    library.child(icon_editor(
                        slot,
                        action,
                        current.and_then(ActionRingEntry::custom_icon),
                        pal,
                    ))
                })
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .px_1p5()
                        .child(editor_section(tr!("Actions"), pal))
                        .child(
                            Button::new("ring-new-shortcut")
                                .compact()
                                .ghost()
                                .label(tr!("+ New Shortcut"))
                                .on_click(move |_, window, cx| {
                                    (on_new)(window, cx);
                                }),
                        ),
                )
                .children(catalog_rows),
            library_scroll,
        ))
}

fn slot_header(
    slot: ActionRingSlot,
    current: Option<&ActionRingEntry>,
    current_label: gpui::SharedString,
    pal: Palette,
) -> impl IntoElement {
    v_flex()
        .flex_none()
        .gap_1()
        .border_b_1()
        .border_color(pal.border)
        .p_3()
        .child(div().text_subheading().child(tr!("Actions Ring")))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child(current_label),
                )
                .child(
                    Button::new("ring-clear-slot")
                        .compact()
                        .ghost()
                        .disabled(current.is_none())
                        .label(tr!("Clear slot"))
                        .on_click(move |_, _, cx| commit_slot(slot, None, cx)),
                ),
        )
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
        let actions = Action::catalog()
            .into_iter()
            .filter(|action| RingAction::new(action.clone()).is_ok())
            .collect::<Vec<_>>();
        assert!(actions.contains(&Action::MissionControl));
        assert!(!actions.contains(&Action::None));
        assert!(!actions.contains(&Action::ShowActionsRing));
    }
}
