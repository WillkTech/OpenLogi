//! Interactive Shortcut Builder modal/drawer component for creating and editing
//! custom named shortcuts with modifier chips, key selector, live chord preview,
//! icon grid picker, and interactive legend.

use std::rc::Rc;

use gpui::{
    App, AppContext, Entity, IntoElement, ParentElement,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder, px, rgb,
};
use gpui_component::{
    Icon, IconName, Selectable as _,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use openlogi_core::binding::{ActionRingIcon, KeyCombo, KeyComboParseError};

use crate::ui::section::section_label;
use crate::ui::theme::{ACCENT_BLUE, Palette, Typography as _};

/// Represents the state of the interactive shortcut builder.
pub struct ShortcutBuilderState {
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
}

impl ShortcutBuilderState {
    pub fn new(
        existing_id: Option<String>,
        name: &str,
        combo: Option<&KeyCombo>,
        icon: Option<ActionRingIcon>,
        window: &mut Window,
        cx: &mut App,
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

        let mut name_state = InputState::new(window, cx);
        name_state.set_value(name, window, cx);
        let name_input = cx.new(|_| name_state);

        let mut key_state = InputState::new(window, cx);
        key_state.set_value(&key_text, window, cx);
        let key_input = cx.new(|_| key_state);

        Self {
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

/// Render the interactive Shortcut Builder component.
pub fn render_shortcut_builder(
    builder: &mut ShortcutBuilderState,
    pal: Palette,
    window: &mut Window,
    cx: &mut App,
    on_save: Rc<dyn Fn(String, String, KeyCombo, Option<ActionRingIcon>, &mut Window, &mut App)>,
    on_cancel: Rc<dyn Fn(&mut Window, &mut App)>,
) -> impl IntoElement {
    let current_combo_res = builder.build_combo(cx);
    let name_text = builder.name_input.read(cx).value().to_string();

    v_flex()
        .gap_3()
        .p_4()
        .bg(pal.panel)
        .border_1()
        .border_color(pal.border)
        .rounded(pal.card_radius)
        .shadow_md()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    div().text_subheading().child(if builder.id.is_some() {
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
                            move |_, window, cx| on_cancel(window, cx)
                        }),
                ),
        )
        // Shortcut Name Input
        .child(
            v_flex()
                .gap_1()
                .child(section_label("Shortcut Name", pal))
                .child(Input::new(&builder.name_input)),
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
                        .child(modifier_chip(
                            "mod-chip-win",
                            "Win / Super",
                            builder.win_active,
                            pal,
                        ))
                        .child(modifier_chip("mod-chip-cmd", "Cmd ⌘", builder.cmd_active, pal))
                        .child(modifier_chip("mod-chip-ctrl", "Ctrl", builder.ctrl_active, pal))
                        .child(modifier_chip("mod-chip-alt", "Alt / Option", builder.alt_active, pal))
                        .child(modifier_chip("mod-chip-shift", "Shift ⇧", builder.shift_active, pal))
                        .child(modifier_chip("mod-chip-fn", "Fn / Globe", builder.fn_active, pal)),
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
                        .child(div().flex_1().child(Input::new(&builder.key_input)))
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
                                .text_color(rgb(0xffffff))
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
                        .label(if builder.legend_open {
                            "Hide Syntax Legend ▲"
                        } else {
                            "Show Syntax Legend ▼"
                        }),
                )
                .when(builder.legend_open, |this| {
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
                            let selected = builder.selected_icon == Some(icon);
                            Button::new(("builder-icon", idx))
                                .size(px(32.0))
                                .rounded(px(16.0))
                                .selected(selected)
                                .icon(
                                    Icon::empty()
                                        .path(icon.asset_path())
                                        .text_color(if selected { gpui::Hsla::from(rgb(ACCENT_BLUE)) } else { pal.text_muted }),
                                )
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
                            move |_, window, cx| on_cancel(window, cx)
                        }),
                )
                .child(
                    Button::new("builder-save")
                        .primary()
                        .label("Save Shortcut")
                        .disabled(current_combo_res.is_err() || name_text.trim().is_empty())
                        .on_click({
                            let on_save = on_save.clone();
                            let combo = current_combo_res.ok();
                            let id = builder.id.clone().unwrap_or_else(|| {
                                format!("custom_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
                            });
                            let icon = builder.selected_icon;
                            move |_, window, cx| {
                                if let Some(combo) = combo.clone() {
                                    on_save(id.clone(), name_text.clone(), combo, icon, window, cx);
                                }
                            }
                        }),
                ),
        )
}

fn modifier_chip(
    id: impl Into<gpui::ElementId>,
    label: &'static str,
    active: bool,
    _pal: Palette,
) -> Button {
    Button::new(id)
        .compact()
        .selected(active)
        .label(label)
}
