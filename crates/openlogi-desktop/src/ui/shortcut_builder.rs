//! Interactive Shortcut Builder modal/drawer component for creating and editing
//! custom named shortcuts with ActionComposer.

use std::rc::Rc;

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div,
};
use gpui_component::{
    Disableable as _, IconName,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use openlogi_core::binding::{ActionRingIcon, ActionTarget};

use crate::ui::action_composer::ActionComposer;
use crate::ui::section::section_label;
use crate::ui::theme::{self, Typography as _};

pub type SaveShortcutFn =
    Rc<dyn Fn(String, String, ActionTarget, Option<ActionRingIcon>, &mut Window, &mut App)>;
pub type CancelShortcutFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Represents the state of the interactive shortcut builder modal wrapping ActionComposer.
pub struct ShortcutBuilderModal {
    focus_handle: FocusHandle,
    pub id: Option<String>,
    pub name_input: Entity<InputState>,
    pub composer: Entity<ActionComposer>,
    pub on_save: Option<SaveShortcutFn>,
    pub on_cancel: Option<CancelShortcutFn>,
}

impl ShortcutBuilderModal {
    pub fn new(
        existing_id: Option<String>,
        name: &str,
        target: Option<&ActionTarget>,
        icon: Option<ActionRingIcon>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name_str = name.to_string();
        let name_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            if !name_str.is_empty() {
                state.set_value(name_str, window, cx);
            }
            state
        });

        let composer = cx.new(|cx| ActionComposer::new(target, icon, window, cx));

        Self {
            focus_handle: cx.focus_handle(),
            id: existing_id,
            name_input,
            composer,
            on_save: None,
            on_cancel: None,
        }
    }
}

impl Focusable for ShortcutBuilderModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutBuilderModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = theme::palette(cx);
        let name_text = self.name_input.read(cx).value().to_string();
        let composer_read = self.composer.read(cx);
        let target_opt = composer_read.build_target(cx);
        let selected_icon = composer_read.selected_icon;
        let is_valid = target_opt.is_some() && !name_text.trim().is_empty();

        let id_val = self.id.clone().unwrap_or_else(|| {
            format!(
                "custom_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            )
        });
        let on_save = self.on_save.clone();
        let on_cancel = self.on_cancel.clone();

        v_flex()
            .gap_3()
            .p_4()
            .bg(pal.panel)
            .border_1()
            .border_color(pal.border)
            .rounded(pal.card_radius)
            .shadow_md()
            .track_focus(&self.focus_handle)
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div().text_subheading().child(if self.id.is_some() {
                            "Edit Custom Action"
                        } else {
                            "New Custom Action"
                        }),
                    )
                    .child(
                        Button::new("cancel-builder")
                            .icon(IconName::Close)
                            .ghost()
                            .on_click({
                                let on_cancel = on_cancel.clone();
                                move |_, window, cx| {
                                    if let Some(on_cancel) = &on_cancel {
                                        on_cancel(window, cx);
                                    }
                                }
                            }),
                    ),
            )
            // Shortcut Name Input
            .child(
                v_flex()
                    .gap_1()
                    .child(section_label("Name", pal))
                    .child(Input::new(&self.name_input)),
            )
            // Shared ActionComposer View
            .child(self.composer.clone())
            // Action Buttons
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("builder-cancel")
                            .label("Cancel")
                            .on_click({
                                let on_cancel = on_cancel.clone();
                                move |_, window, cx| {
                                    if let Some(on_cancel) = &on_cancel {
                                        on_cancel(window, cx);
                                    }
                                }
                            }),
                    )
                    .child(
                        Button::new("builder-save")
                            .primary()
                            .label("Save Action")
                            .disabled(!is_valid)
                            .on_click({
                                move |_, window, cx| {
                                    if let (Some(on_save), Some(target)) = (&on_save, target_opt.clone()) {
                                        on_save(id_val.clone(), name_text.clone(), target, selected_icon, window, cx);
                                    }
                                }
                            }),
                    ),
            )
    }
}
