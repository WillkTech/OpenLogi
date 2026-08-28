//! Shared ActionComposer GPUI component for configuring shortcuts and application launch targets
//! with bi-directional modifier token synchronization, live preview, and icon selection.

use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    Icon, Selectable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use openlogi_core::binding::{
    ActionRingIcon, ActionTarget, ApplicationTarget, KeyCombo, KeyComboParseError,
};

use crate::ui::section::section_label;
use crate::ui::theme::{self, ACCENT_BLUE, Typography as _};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComposerTargetMode {
    #[default]
    Shortcut,
    Application,
}

pub fn parse_modifiers_from_text(text: &str) -> (bool, bool, bool, bool, bool, bool) {
    let lower = text.to_lowercase();
    let tokens: Vec<&str> = lower.split('+').map(|s| s.trim()).collect();

    let win = tokens
        .iter()
        .any(|&t| t == "win" || t == "super" || t == "meta");
    let cmd = tokens.iter().any(|&t| t == "cmd" || t == "command");
    let ctrl = tokens.iter().any(|&t| t == "ctrl" || t == "control");
    let alt = tokens.iter().any(|&t| t == "alt" || t == "option");
    let shift = tokens.iter().any(|&t| t == "shift");
    let fn_key = tokens.iter().any(|&t| t == "fn" || t == "globe");

    (win, cmd, ctrl, alt, shift, fn_key)
}

pub fn toggle_modifier_in_text(text: &str, modifier: &str) -> String {
    let lower_mod = modifier.to_lowercase();
    let mut tokens: Vec<&str> = text
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut found_idx = None;
    for (i, t) in tokens.iter().enumerate() {
        let t_lower = t.to_lowercase();
        if t_lower == lower_mod
            || (lower_mod == "win" && (t_lower == "super" || t_lower == "meta"))
            || (lower_mod == "cmd" && t_lower == "command")
            || (lower_mod == "ctrl" && t_lower == "control")
            || (lower_mod == "alt" && t_lower == "option")
            || (lower_mod == "fn" && t_lower == "globe")
        {
            found_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = found_idx {
        tokens.remove(idx);
    } else {
        let order = ["win", "cmd", "ctrl", "alt", "shift", "fn"];
        let mod_order_pos = order.iter().position(|&m| m == lower_mod).unwrap_or(0);

        let mut insert_idx = tokens.len();
        for (i, t) in tokens.iter().enumerate() {
            let t_lower = t.to_lowercase();
            let pos = order.iter().position(|&m| m == t_lower).unwrap_or(99);
            if pos > mod_order_pos {
                insert_idx = i;
                break;
            }
        }
        tokens.insert(insert_idx, modifier);
    }

    tokens.join(" + ")
}

pub struct ActionComposer {
    pub mode: ComposerTargetMode,
    pub key_input: Entity<InputState>,
    pub path_input: Entity<InputState>,
    pub win_active: bool,
    pub cmd_active: bool,
    pub ctrl_active: bool,
    pub alt_active: bool,
    pub shift_active: bool,
    pub fn_active: bool,
    pub selected_icon: Option<ActionRingIcon>,
    pub legend_open: bool,
    _input_obs: Subscription,
}

impl ActionComposer {
    pub fn new(
        target: Option<&ActionTarget>,
        icon: Option<ActionRingIcon>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (mode, combo_opt, app_opt) = match target {
            Some(ActionTarget::Shortcut(combo)) => {
                (ComposerTargetMode::Shortcut, Some(combo), None)
            }
            Some(ActionTarget::Application(app)) => {
                (ComposerTargetMode::Application, None, Some(app))
            }
            None => (ComposerTargetMode::Shortcut, None, None),
        };

        let key_text = if let Some(combo) = combo_opt {
            combo.rendered_label()
        } else {
            String::new()
        };

        let path_text = if let Some(app) = app_opt {
            app.path().to_string()
        } else {
            String::new()
        };

        let (win, cmd, ctrl, alt, shift, fn_mod) = parse_modifiers_from_text(&key_text);

        let key_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            if !key_text.is_empty() {
                state.set_value(key_text, window, cx);
            }
            state
        });

        let path_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            if !path_text.is_empty() {
                state.set_value(path_text, window, cx);
            }
            state
        });

        let input_obs = cx.subscribe(&key_input, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                let text = this.key_input.read(cx).value();
                let (win, cmd, ctrl, alt, shift, fn_key) = parse_modifiers_from_text(&text);
                this.win_active = win;
                this.cmd_active = cmd;
                this.ctrl_active = ctrl;
                this.alt_active = alt;
                this.shift_active = shift;
                this.fn_active = fn_key;
                cx.notify();
            }
        });

        Self {
            mode,
            key_input,
            path_input,
            win_active: win,
            cmd_active: cmd,
            ctrl_active: ctrl,
            alt_active: alt,
            shift_active: shift,
            fn_active: fn_mod,
            selected_icon: icon,
            legend_open: false,
            _input_obs: input_obs,
        }
    }

    pub fn build_combo(&self, cx: &App) -> Result<KeyCombo, KeyComboParseError> {
        let value = self.key_input.read(cx).value();
        let key_str = value.trim();
        if key_str.is_empty() {
            return Err(KeyComboParseError::MissingKey);
        }
        key_str.parse::<KeyCombo>()
    }

    pub fn build_target(&self, cx: &App) -> Option<ActionTarget> {
        match self.mode {
            ComposerTargetMode::Shortcut => self.build_combo(cx).ok().map(ActionTarget::Shortcut),
            ComposerTargetMode::Application => {
                let path = self.path_input.read(cx).value().trim().to_string();
                if path.is_empty() {
                    None
                } else {
                    ApplicationTarget::new(path, "")
                        .ok()
                        .map(ActionTarget::Application)
                }
            }
        }
    }
}

impl Render for ActionComposer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = theme::palette(cx);
        let current_combo_res = self.build_combo(cx);

        v_flex()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(section_label("Target Type", pal))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("mode-shortcut")
                                    .compact()
                                    .selected(self.mode == ComposerTargetMode::Shortcut)
                                    .label("Shortcut")
                                    .on_click(cx.listener(|composer, _, _, cx| {
                                        composer.mode = ComposerTargetMode::Shortcut;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mode-app")
                                    .compact()
                                    .selected(self.mode == ComposerTargetMode::Application)
                                    .label("Application / Folder")
                                    .on_click(cx.listener(|composer, _, _, cx| {
                                        composer.mode = ComposerTargetMode::Application;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .when(self.mode == ComposerTargetMode::Shortcut, |this| {
                this
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
                                            .on_click(cx.listener(|composer, _, window, cx| {
                                                let val = composer.key_input.read(cx).value();
                                                let new_val = toggle_modifier_in_text(&val, "Win");
                                                composer.key_input.update(cx, |input, cx| {
                                                    input.set_value(new_val, window, cx);
                                                });
                                            })),
                                    )
                                    .child(
                                        Button::new("mod-chip-cmd")
                                            .compact()
                                            .selected(self.cmd_active)
                                            .label("Cmd ⌘")
                                            .on_click(cx.listener(|composer, _, window, cx| {
                                                let val = composer.key_input.read(cx).value();
                                                let new_val = toggle_modifier_in_text(&val, "Cmd");
                                                composer.key_input.update(cx, |input, cx| {
                                                    input.set_value(new_val, window, cx);
                                                });
                                            })),
                                    )
                                    .child(
                                        Button::new("mod-chip-ctrl")
                                            .compact()
                                            .selected(self.ctrl_active)
                                            .label("Ctrl")
                                            .on_click(cx.listener(|composer, _, window, cx| {
                                                let val = composer.key_input.read(cx).value();
                                                let new_val = toggle_modifier_in_text(&val, "Ctrl");
                                                composer.key_input.update(cx, |input, cx| {
                                                    input.set_value(new_val, window, cx);
                                                });
                                            })),
                                    )
                                    .child(
                                        Button::new("mod-chip-alt")
                                            .compact()
                                            .selected(self.alt_active)
                                            .label("Alt / Option")
                                            .on_click(cx.listener(|composer, _, window, cx| {
                                                let val = composer.key_input.read(cx).value();
                                                let new_val = toggle_modifier_in_text(&val, "Alt");
                                                composer.key_input.update(cx, |input, cx| {
                                                    input.set_value(new_val, window, cx);
                                                });
                                            })),
                                    )
                                    .child(
                                        Button::new("mod-chip-shift")
                                            .compact()
                                            .selected(self.shift_active)
                                            .label("Shift ⇧")
                                            .on_click(cx.listener(|composer, _, window, cx| {
                                                let val = composer.key_input.read(cx).value();
                                                let new_val = toggle_modifier_in_text(&val, "Shift");
                                                composer.key_input.update(cx, |input, cx| {
                                                    input.set_value(new_val, window, cx);
                                                });
                                            })),
                                    )
                                    .child(
                                        Button::new("mod-chip-fn")
                                            .compact()
                                            .selected(self.fn_active)
                                            .label("Fn / Globe")
                                            .on_click(cx.listener(|composer, _, window, cx| {
                                                let val = composer.key_input.read(cx).value();
                                                let new_val = toggle_modifier_in_text(&val, "Fn");
                                                composer.key_input.update(cx, |input, cx| {
                                                    input.set_value(new_val, window, cx);
                                                });
                                            })),
                                    ),
                            ),
                    )
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
                                    .on_click(cx.listener(|composer, _, _, cx| {
                                        composer.legend_open = !composer.legend_open;
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
            })
            .when(self.mode == ComposerTargetMode::Application, |this| {
                this.child(
                    v_flex()
                        .gap_1()
                        .child(section_label("Application or Folder Path", pal))
                        .child(Input::new(&self.path_input)),
                )
            })
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
                                Button::new(("composer-icon", idx))
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
                                    .on_click(cx.listener(move |composer, _, _, cx| {
                                        composer.selected_icon = if composer.selected_icon == Some(icon) {
                                            None
                                        } else {
                                            Some(icon)
                                        };
                                        cx.notify();
                                    }))
                            })),
                    ),
            )
    }
}
