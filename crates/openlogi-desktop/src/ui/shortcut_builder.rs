//! Interactive Shortcut Builder modal/drawer component for creating and editing
//! custom named shortcuts with modifier chips, key selector, live chord preview,
//! icon grid picker, and interactive legend.

use std::rc::Rc;

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px, rgb,
};
use gpui_component::{
    Disableable, Icon, IconName, Selectable as _,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use openlogi_core::binding::{ActionRingIcon, KeyCombo, KeyComboParseError};

use crate::ui::section::section_label;
use crate::ui::theme::{self, ACCENT_BLUE, Typography as _};

pub type SaveShortcutFn =
    Rc<dyn Fn(String, String, KeyCombo, Option<ActionRingIcon>, &mut Window, &mut App)>;
pub type CancelShortcutFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Represents the state of the interactive shortcut builder.
#[expect(
    clippy::struct_excessive_bools,
    reason = "Represents interactive modifier key toggles"
)]
pub struct ShortcutBuilderModal {
    focus_handle: FocusHandle,
    pub id: Option<String>,
    pub name_input: Entity<InputState>,
    pub key_input: Entity<InputState>,
    pub win_active: bool,
    pub cmd_active: bool,
    pub ctrl_active: bool,
    pub alt_active: bool,
    pub shift_active: bool,
    pub fn_active: bool,
    pub selected_icon: Option<ActionRingIcon>,
    pub legend_open: bool,
    pub on_save: Option<SaveShortcutFn>,
    pub on_cancel: Option<CancelShortcutFn>,
}

impl ShortcutBuilderModal {
    pub fn new(
        existing_id: Option<String>,
        name: &str,
        combo: Option<&KeyCombo>,
        icon: Option<ActionRingIcon>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (win, cmd, ctrl, alt, shift, fn_mod) = if let Some(combo) = combo {
            (
                combo.has_meta(),
                combo.has_command(),
                combo.has_control(),
                combo.has_option(),
                combo.has_shift(),
                combo.has_fn(),
            )
        } else {
            (false, false, false, false, false, false)
        };

        let key_text = if let Some(combo) = combo {
            combo.key().label()
        } else {
            String::new()
        };

        let name_str = name.to_string();
        let name_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            if !name_str.is_empty() {
                state.set_value(name_str, window, cx);
            }
            state
        });

        let key_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            if !key_text.is_empty() {
                state.set_value(key_text, window, cx);
            }
            state
        });

        Self {
            focus_handle: cx.focus_handle(),
            id: existing_id,
            name_input,
            key_input,
            win_active: win,
            cmd_active: cmd,
            ctrl_active: ctrl,
            alt_active: alt,
            shift_active: shift,
            fn_active: fn_mod,
            selected_icon: icon,
            legend_open: false,
            on_save: None,
            on_cancel: None,
        }
    }

    /// Construct a `KeyCombo` from the currently selected modifier chips and target key input.
    pub fn build_combo(&self, cx: &App) -> Result<KeyCombo, KeyComboParseError> {
        let value = self.key_input.read(cx).value();
        let key_str = value.trim();
        if key_str.is_empty() {
            return Err(KeyComboParseError::MissingKey);
        }

        let mut parts = Vec::new();
        if self.win_active {
            parts.push("Win");
        }
        if self.cmd_active {
            parts.push("Cmd");
        }
        if self.ctrl_active {
            parts.push("Ctrl");
        }
        if self.alt_active {
            parts.push("Alt");
        }
        if self.shift_active {
            parts.push("Shift");
        }
        if self.fn_active {
            parts.push("Fn");
        }
        parts.push(key_str);

        let chord = parts.join("+");
        chord.parse::<KeyCombo>()
    }
}

impl Focusable for ShortcutBuilderModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutBuilderModal {
    #[expect(
        clippy::too_many_lines,
        reason = "Modal component with multiple input sections and icon picker"
    )]
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = theme::palette(cx);
        let current_combo_res = self.build_combo(cx);
        let name_text = self.name_input.read(cx).value().to_string();
        let is_valid = current_combo_res.is_ok() && !name_text.trim().is_empty();

        let id_val = self.id.clone().unwrap_or_else(|| {
            format!(
                "custom_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            )
        });
        let selected_icon = self.selected_icon;
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
                            "Edit Custom Shortcut"
                        } else {
                            "New Custom Shortcut"
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
                    .child(section_label("Shortcut Name", pal))
                    .child(Input::new(&self.name_input)),
            )
            // Modifier Chips
            .child(
                v_flex()
                    .gap_1()
                    .child(section_label("Modifiers", pal))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1p5()
                            .child(
                                Button::new("mod-chip-win")
                                    .compact()
                                    .selected(self.win_active)
                                    .label("Win / Super")
                                    .on_click(cx.listener(|modal, _, _, cx| {
                                        modal.win_active = !modal.win_active;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mod-chip-cmd")
                                    .compact()
                                    .selected(self.cmd_active)
                                    .label("Cmd ⌘")
                                    .on_click(cx.listener(|modal, _, _, cx| {
                                        modal.cmd_active = !modal.cmd_active;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mod-chip-ctrl")
                                    .compact()
                                    .selected(self.ctrl_active)
                                    .label("Ctrl")
                                    .on_click(cx.listener(|modal, _, _, cx| {
                                        modal.ctrl_active = !modal.ctrl_active;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mod-chip-alt")
                                    .compact()
                                    .selected(self.alt_active)
                                    .label("Alt / Option")
                                    .on_click(cx.listener(|modal, _, _, cx| {
                                        modal.alt_active = !modal.alt_active;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mod-chip-shift")
                                    .compact()
                                    .selected(self.shift_active)
                                    .label("Shift ⇧")
                                    .on_click(cx.listener(|modal, _, _, cx| {
                                        modal.shift_active = !modal.shift_active;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mod-chip-fn")
                                    .compact()
                                    .selected(self.fn_active)
                                    .label("Fn / Globe")
                                    .on_click(cx.listener(|modal, _, _, cx| {
                                        modal.fn_active = !modal.fn_active;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Target Key Input
            .child(
                v_flex()
                    .gap_1()
                    .child(section_label("Target Key", pal))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().flex_1().child(Input::new(&self.key_input)))
                            .child(
                                div()
                                    .text_caption()
                                    .text_color(pal.text_muted)
                                    .child("e.g. V, F11, Enter, Space, Tab"),
                            ),
                    ),
            )
            // Live Preview Badge
            .child(
                v_flex()
                    .gap_1()
                    .child(section_label("Live Chord Preview", pal))
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .bg(rgb(ACCENT_BLUE))
                                    .text_color(rgb(0x00ff_ffff))
                                    .child(match &current_combo_res {
                                        Ok(combo) => combo.rendered_label(),
                                        Err(_) => "Enter a valid key".to_string(),
                                    }),
                            ),
                    ),
            )
            // Collapsible Syntax Legend
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        Button::new("toggle-legend")
                            .compact()
                            .ghost()
                            .label(if self.legend_open {
                                "Hide Syntax Legend ▲"
                            } else {
                                "Show Syntax Legend ▼"
                            })
                            .on_click(cx.listener(|modal, _, _, cx| {
                                modal.legend_open = !modal.legend_open;
                                cx.notify();
                            })),
                    )
                    .when(self.legend_open, |this| {
                        this.child(
                            v_flex()
                                .p_2()
                                .bg(pal.control)
                                .rounded(pal.card_radius)
                                .text_caption()
                                .text_color(pal.text_muted)
                                .child("Tokens: Win/Super, Cmd, Ctrl, Alt/Option, Shift, Fn/Globe")
                                .child("Special Keys: F1-F20, Enter, Esc, Tab, Space, Backspace, Delete, Up/Down/Left/Right"),
                        )
                    }),
            )
            // Icon Picker Grid
            .child(
                v_flex()
                    .gap_1()
                    .child(section_label("Select Icon", pal))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .children(ActionRingIcon::ALL.into_iter().enumerate().map(|(idx, icon)| {
                                let selected = self.selected_icon == Some(icon);
                                Button::new(("builder-icon", idx))
                                    .size(px(32.0))
                                    .rounded(px(16.0))
                                    .selected(selected)
                                    .icon(
                                        Icon::empty()
                                            .path(icon.asset_path())
                                            .text_color(if selected {
                                                gpui::Hsla::from(rgb(ACCENT_BLUE))
                                            } else {
                                                pal.text_muted
                                            }),
                                    )
                                    .on_click(cx.listener(move |modal, _, _, cx| {
                                        modal.selected_icon = if modal.selected_icon == Some(icon) {
                                            None
                                        } else {
                                            Some(icon)
                                        };
                                        cx.notify();
                                    }))
                            })),
                    ),
            )
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
                            .label("Save Shortcut")
                            .disabled(!is_valid)
                            .on_click({
                                let combo = current_combo_res.ok();
                                move |_, window, cx| {
                                    if let (Some(on_save), Some(combo)) = (&on_save, combo.clone()) {
                                        on_save(id_val.clone(), name_text.clone(), combo, selected_icon, window, cx);
                                    }
                                }
                            }),
                    ),
            )
    }
}
