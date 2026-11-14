//! Shared ActionComposer GPUI component for configuring shortcuts and application launch targets
//! with bi-directional modifier token synchronization, live preview, and icon selection.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div, prelude::FluentBuilder as _, px, rgb,
};
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

use crate::ui::power_user::{WorkflowStepDraft, workflow_steps_list_and_adder};
use crate::ui::section::section_label;
use crate::ui::theme::{self, ACCENT_BLUE, Typography as _};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComposerTargetMode {
    #[default]
    Shortcut,
    Application,
    TypeText,
    RunAppleScript,
    RunShellCommand,
    Workflow,
}

impl ComposerTargetMode {
    #[must_use]
    pub fn supported_on_current_platform() -> &'static [Self] {
        #[cfg(target_os = "macos")]
        {
            &[
                Self::Shortcut,
                Self::Application,
                Self::TypeText,
                Self::RunAppleScript,
                Self::RunShellCommand,
                Self::Workflow,
            ]
        }
        #[cfg(not(target_os = "macos"))]
        {
            &[
                Self::Shortcut,
                Self::Application,
                Self::RunShellCommand,
                Self::Workflow,
            ]
        }
    }

    pub fn label(self) -> gpui::SharedString {
        match self {
            Self::Shortcut => tr!("Shortcut"),
            Self::Application => tr!("Application / Folder"),
            Self::TypeText => tr!("Type Text"),
            Self::RunAppleScript => tr!("AppleScript"),
            Self::RunShellCommand => tr!("Shell"),
            Self::Workflow => tr!("Workflow"),
        }
    }
}

pub fn parse_modifiers_from_text(text: &str) -> (bool, bool, bool, bool, bool, bool) {
    let lower = text.to_lowercase();
    let tokens: Vec<&str> = lower.split('+').map(str::trim).collect();

    let win = tokens
        .iter()
        .any(|&t| t == "win" || t == "super" || t == "meta");
    let cmd = tokens.iter().any(|&t| t == "cmd" || t == "command");
    let ctrl = tokens.iter().any(|&t| t == "ctrl" || t == "control");
    let alt = tokens.iter().any(|&t| t == "alt" || t == "option");
    let shift = tokens.contains(&"shift");
    let fn_key = tokens.iter().any(|&t| t == "fn" || t == "globe");

    (win, cmd, ctrl, alt, shift, fn_key)
}

pub fn toggle_modifier_in_text(text: &str, modifier: &str) -> String {
    let lower_mod = modifier.to_lowercase();
    let mut tokens: Vec<&str> = text
        .split('+')
        .map(str::trim)
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

#[expect(
    clippy::struct_excessive_bools,
    reason = "tracks active states of 6 modifier keys plus legend"
)]
pub struct ActionComposer {
    pub mode: ComposerTargetMode,
    pub key_input: Entity<InputState>,
    pub path_input: Entity<InputState>,
    pub text_input: Entity<InputState>,
    pub workflow_steps: Rc<RefCell<Vec<WorkflowStepDraft>>>,
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
    #[expect(
        clippy::too_many_lines,
        reason = "initializes interactive inputs, subscriptions, and drafts across all target kinds"
    )]
    pub fn new(
        target: Option<&ActionTarget>,
        icon: Option<ActionRingIcon>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (mode, combo_opt, app_opt, text_opt, wf_steps) = match target {
            Some(ActionTarget::Shortcut(combo)) => (
                ComposerTargetMode::Shortcut,
                Some(combo.clone()),
                None,
                None,
                Vec::new(),
            ),
            Some(ActionTarget::Application(app)) => (
                ComposerTargetMode::Application,
                None,
                Some(app.clone()),
                None,
                Vec::new(),
            ),
            Some(ActionTarget::Text(text)) => (
                ComposerTargetMode::TypeText,
                None,
                None,
                Some(text.clone()),
                Vec::new(),
            ),
            Some(ActionTarget::AppleScript(script)) => (
                ComposerTargetMode::RunAppleScript,
                None,
                None,
                Some(script.clone()),
                Vec::new(),
            ),
            Some(ActionTarget::Shell(cmd)) => (
                ComposerTargetMode::RunShellCommand,
                None,
                None,
                Some(cmd.clone()),
                Vec::new(),
            ),
            Some(ActionTarget::Workflow(steps)) => (
                ComposerTargetMode::Workflow,
                None,
                None,
                None,
                steps.clone(),
            ),
            None => (ComposerTargetMode::Shortcut, None, None, None, Vec::new()),
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

        let text_seed = text_opt.unwrap_or_default();
        let text_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Text to type or script…");
            if !text_seed.is_empty() {
                state.set_value(text_seed, window, cx);
            }
            state
        });

        let workflow_steps = Rc::new(RefCell::new(
            wf_steps
                .into_iter()
                .map(|step| WorkflowStepDraft::new(step, window, cx))
                .collect(),
        ));

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
            text_input,
            workflow_steps,
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
            ComposerTargetMode::TypeText => {
                let text = self.text_input.read(cx).value().to_string();
                Some(ActionTarget::Text(text))
            }
            ComposerTargetMode::RunAppleScript => {
                let script = self.text_input.read(cx).value().to_string();
                Some(ActionTarget::AppleScript(script))
            }
            ComposerTargetMode::RunShellCommand => {
                let cmd = self.text_input.read(cx).value().to_string();
                Some(ActionTarget::Shell(cmd))
            }
            ComposerTargetMode::Workflow => {
                let steps = self
                    .workflow_steps
                    .borrow()
                    .iter()
                    .map(|draft| draft.build_step(cx))
                    .collect();
                Some(ActionTarget::Workflow(steps))
            }
        }
    }
}

impl Render for ActionComposer {
    #[expect(
        clippy::too_many_lines,
        reason = "renders full interactive ActionComposer view"
    )]
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
                            .flex_wrap()
                            .gap_1p5()
                            .children(
                                ComposerTargetMode::supported_on_current_platform()
                                    .iter()
                                    .map(|&mode| {
                                        Button::new(format!("mode-{mode:?}"))
                                            .compact()
                                            .selected(self.mode == mode)
                                            .label(mode.label())
                                            .on_click(cx.listener(move |composer, _, _, cx| {
                                                composer.mode = mode;
                                                cx.notify();
                                            }))
                                    }),
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
            .when(
                matches!(
                    self.mode,
                    ComposerTargetMode::TypeText
                        | ComposerTargetMode::RunAppleScript
                        | ComposerTargetMode::RunShellCommand
                ),
                |this| {
                    let lbl = match self.mode {
                        ComposerTargetMode::TypeText => "Text to type",
                        ComposerTargetMode::RunAppleScript => "AppleScript",
                        ComposerTargetMode::RunShellCommand => "Shell Command",
                        _ => "",
                    };
                    this.child(
                        v_flex()
                            .gap_1()
                            .child(section_label(lbl, pal))
                            .child(Input::new(&self.text_input)),
                    )
                },
            )
            .when(self.mode == ComposerTargetMode::Workflow, |this| {
                this.child(
                    v_flex()
                        .gap_1()
                        .child(section_label("Workflow Steps", pal))
                        .child(workflow_steps_list_and_adder(
                            &self.workflow_steps,
                            pal,
                            "composer-workflow-steps",
                        )),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_target_mode_supported_on_current_platform() {
        let modes = ComposerTargetMode::supported_on_current_platform();
        #[cfg(target_os = "macos")]
        {
            assert_eq!(
                modes,
                &[
                    ComposerTargetMode::Shortcut,
                    ComposerTargetMode::Application,
                    ComposerTargetMode::TypeText,
                    ComposerTargetMode::RunAppleScript,
                    ComposerTargetMode::RunShellCommand,
                    ComposerTargetMode::Workflow,
                ]
            );
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(
                modes,
                &[
                    ComposerTargetMode::Shortcut,
                    ComposerTargetMode::Application,
                    ComposerTargetMode::RunShellCommand,
                    ComposerTargetMode::Workflow,
                ]
            );
        }
    }

    #[test]
    fn composer_target_mode_labels_are_non_empty() {
        for &mode in ComposerTargetMode::supported_on_current_platform() {
            assert!(!mode.label().is_empty());
        }
    }
}
