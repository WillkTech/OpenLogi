//! Shared controller and card components for Power User actions and Workflow editor.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    App, AppContext as _, Entity, FontWeight, ParentElement, Styled, Window, div,
    prelude::FluentBuilder as _, px, svg,
};
use gpui_component::{
    IconName,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    v_flex,
};
use openlogi_core::binding::{Action, ActionTarget, KeyCombo, WorkflowStep};

use crate::features::mouse::picker::{compact_panel, divider, editor_scroll_list, title};
use crate::ui::components::control_input;
use crate::ui::theme::{Palette, Typography as _};

type CommitActionFn = Arc<dyn Fn(Action, &mut Window, &mut App)>;
type CloseActionFn = Arc<dyn Fn(&mut Window, &mut App)>;
pub type PromoteActionFn = Arc<dyn Fn(ActionTarget, &mut Window, &mut App)>;

/// Which power-user editor is showing.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PowerUserKind {
    TypeText,
    RunAppleScript,
    RunShellCommand,
    Workflow,
}

impl PowerUserKind {
    #[must_use]
    pub fn supported_on_current_platform() -> &'static [Self] {
        #[cfg(target_os = "macos")]
        {
            &[
                Self::TypeText,
                Self::RunAppleScript,
                Self::RunShellCommand,
                Self::Workflow,
            ]
        }
        #[cfg(not(target_os = "macos"))]
        {
            &[Self::RunShellCommand, Self::Workflow]
        }
    }

    pub fn heading(self) -> gpui::SharedString {
        match self {
            Self::TypeText => tr!("Type Text"),
            Self::RunAppleScript => tr!("Run AppleScript"),
            Self::RunShellCommand => tr!("Run Shell Command"),
            Self::Workflow => tr!("Workflow"),
        }
    }
}

pub fn text_editor_placeholder(kind: PowerUserKind) -> &'static str {
    match kind {
        PowerUserKind::TypeText => "Text to type…",
        PowerUserKind::RunAppleScript => "display dialog \"Hello\"",
        PowerUserKind::RunShellCommand => "echo hello",
        PowerUserKind::Workflow => "",
    }
}

pub fn text_editor_seed(action: Option<&Action>, kind: PowerUserKind) -> String {
    match (action, kind) {
        (Some(Action::TypeText(text)), PowerUserKind::TypeText)
        | (Some(Action::RunAppleScript(text)), PowerUserKind::RunAppleScript)
        | (Some(Action::RunShellCommand(text)), PowerUserKind::RunShellCommand) => text.clone(),
        _ => String::new(),
    }
}

pub fn workflow_editor_seed(action: Option<&Action>) -> Vec<WorkflowStep> {
    match action {
        Some(Action::Workflow(steps)) => steps.clone(),
        _ => Vec::new(),
    }
}

/// Supported workflow step types for the step adder bar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorkflowStepType {
    TypeText,
    PressKey,
    Delay,
    RunAppleScript,
    RunShellCommand,
}

impl WorkflowStepType {
    #[must_use]
    pub fn supported_on_current_platform() -> &'static [Self] {
        #[cfg(target_os = "macos")]
        {
            &[
                Self::TypeText,
                Self::PressKey,
                Self::Delay,
                Self::RunAppleScript,
                Self::RunShellCommand,
            ]
        }
        #[cfg(not(target_os = "macos"))]
        {
            &[Self::PressKey, Self::Delay, Self::RunShellCommand]
        }
    }

    pub fn label(self) -> gpui::SharedString {
        match self {
            Self::TypeText => tr!("Type Text"),
            Self::PressKey => tr!("Press Key"),
            Self::Delay => tr!("Delay"),
            Self::RunAppleScript => tr!("AppleScript"),
            Self::RunShellCommand => tr!("Shell"),
        }
    }

    pub fn default_step(self) -> WorkflowStep {
        match self {
            Self::TypeText => WorkflowStep::TypeText(String::new()),
            Self::PressKey => {
                #[cfg(target_os = "macos")]
                let combo_str = "cmd+c";
                #[cfg(not(target_os = "macos"))]
                let combo_str = "ctrl+c";
                #[expect(
                    clippy::unwrap_used,
                    reason = "hardcoded key combo string parses unconditionally"
                )]
                let combo = combo_str.parse::<KeyCombo>().unwrap();
                WorkflowStep::PressKey(combo)
            }
            Self::Delay => WorkflowStep::Delay { millis: 100 },
            Self::RunAppleScript => WorkflowStep::RunAppleScript(String::new()),
            Self::RunShellCommand => WorkflowStep::RunShellCommand(String::new()),
        }
    }
}

fn step_seed_and_placeholder(step: &WorkflowStep) -> (String, gpui::SharedString) {
    match step {
        WorkflowStep::TypeText(text) => (text.clone(), tr!("Text to type…")),
        WorkflowStep::PressKey(combo) => (combo.rendered_label(), tr!("Key chord (e.g. Cmd+C)")),
        WorkflowStep::Delay { millis } => (millis.to_string(), "100".into()),
        WorkflowStep::RunAppleScript(script) => (script.clone(), "display dialog \"Hello\"".into()),
        WorkflowStep::RunShellCommand(cmd) => (cmd.clone(), "echo hello".into()),
    }
}

/// One drafted workflow step in the editor with an inline input state.
#[derive(Clone)]
pub struct WorkflowStepDraft {
    pub step: WorkflowStep,
    pub input_state: Entity<InputState>,
}

impl WorkflowStepDraft {
    pub fn new(step: WorkflowStep, window: &mut Window, cx: &mut App) -> Self {
        let (seed, placeholder) = step_seed_and_placeholder(&step);
        let input_state = cx.new(|cx| {
            let mut input = InputState::new(window, cx).placeholder(placeholder);
            if !seed.is_empty() {
                input.set_value(seed, window, cx);
            }
            input
        });
        Self { step, input_state }
    }

    pub fn build_step(&self, cx: &App) -> WorkflowStep {
        let binding = self.input_state.read(cx).value();
        let val = binding.trim();
        match &self.step {
            WorkflowStep::TypeText(_) => WorkflowStep::TypeText(val.to_string()),
            WorkflowStep::RunAppleScript(_) => WorkflowStep::RunAppleScript(val.to_string()),
            WorkflowStep::RunShellCommand(_) => WorkflowStep::RunShellCommand(val.to_string()),
            WorkflowStep::Delay { .. } => {
                let millis = val.parse::<u64>().unwrap_or(100);
                WorkflowStep::Delay { millis }
            }
            WorkflowStep::PressKey(default_combo) => {
                let combo = val
                    .parse::<KeyCombo>()
                    .unwrap_or_else(|_| default_combo.clone());
                WorkflowStep::PressKey(combo)
            }
        }
    }
}

/// Unified Power User editor draft state.
#[derive(Clone, Default)]
pub struct PowerUserEditorState {
    active_editor: Option<PowerUserKind>,
    text_state: Option<Entity<InputState>>,
    workflow_steps: Rc<RefCell<Vec<WorkflowStepDraft>>>,
}

impl PowerUserEditorState {
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.active_editor.is_some()
    }

    pub fn close(&mut self) {
        self.active_editor = None;
        self.text_state = None;
        self.workflow_steps.borrow_mut().clear();
    }

    pub fn open(
        &mut self,
        kind: PowerUserKind,
        current_action: Option<&Action>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.active_editor = Some(kind);
        if kind == PowerUserKind::Workflow {
            self.text_state = None;
            let initial_steps = workflow_editor_seed(current_action);
            let drafts = initial_steps
                .into_iter()
                .map(|step| WorkflowStepDraft::new(step, window, cx))
                .collect();
            *self.workflow_steps.borrow_mut() = drafts;
        } else {
            self.workflow_steps.borrow_mut().clear();
            let seed = text_editor_seed(current_action, kind);
            let placeholder = text_editor_placeholder(kind);
            let placeholder_text: gpui::SharedString = match kind {
                PowerUserKind::TypeText => tr!("Text to type…"),
                _ => placeholder.into(),
            };
            let state = cx.new(|cx| {
                let mut input = InputState::new(window, cx).placeholder(placeholder_text);
                if !seed.is_empty() {
                    input.set_value(seed, window, cx);
                }
                input
            });
            self.text_state = Some(state);
        }
    }

    #[expect(
        dead_code,
        reason = "convenience wrapper when promote callback is not needed"
    )]
    pub fn render_card(
        &self,
        target_label: &str,
        on_commit: impl Fn(Action, &mut Window, &mut App) + 'static,
        on_close: impl Fn(&mut Window, &mut App) + 'static,
        pal: Palette,
        cx: &App,
    ) -> gpui::Div {
        self.render_card_with_promote(target_label, on_commit, on_close, None, pal, cx)
    }

    pub fn render_card_with_promote(
        &self,
        target_label: &str,
        on_commit: impl Fn(Action, &mut Window, &mut App) + 'static,
        on_close: impl Fn(&mut Window, &mut App) + 'static,
        on_promote: Option<PromoteActionFn>,
        pal: Palette,
        cx: &App,
    ) -> gpui::Div {
        let on_commit = Arc::new(on_commit);
        let on_close = Arc::new(on_close);
        match self.active_editor {
            Some(PowerUserKind::Workflow) => workflow_editor_card(
                target_label,
                &self.workflow_steps,
                on_commit,
                on_close,
                on_promote,
                pal,
                cx,
            ),
            Some(kind) => {
                if let Some(state) = self.text_state.clone() {
                    text_editor_card(
                        target_label,
                        kind,
                        state,
                        on_commit,
                        on_close,
                        on_promote,
                        pal,
                    )
                } else {
                    compact_panel(pal)
                        .w(px(320.))
                        .child(title(tr!("Editor unavailable"), pal))
                }
            }
            None => div(),
        }
    }
}

#[cfg(test)]
impl PowerUserEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn active_kind(&self) -> Option<PowerUserKind> {
        self.active_editor
    }

    pub fn push_step(&mut self, step: WorkflowStep, window: &mut Window, cx: &mut App) {
        let draft = WorkflowStepDraft::new(step, window, cx);
        self.workflow_steps.borrow_mut().push(draft);
    }

    pub fn remove_step(&mut self, idx: usize) {
        let mut steps = self.workflow_steps.borrow_mut();
        if idx < steps.len() {
            steps.remove(idx);
        }
    }

    pub fn update_workflow_step(
        &mut self,
        idx: usize,
        step: WorkflowStep,
        window: &mut Window,
        cx: &mut App,
    ) {
        let mut steps = self.workflow_steps.borrow_mut();
        if let Some(draft) = steps.get_mut(idx) {
            let (seed, _) = step_seed_and_placeholder(&step);
            draft.step = step;
            draft.input_state.update(cx, |input, cx| {
                input.set_value(seed, window, cx);
            });
        }
    }

    pub fn workflow_steps_len(&self) -> usize {
        self.workflow_steps.borrow().len()
    }

    pub fn build_workflow_steps(&self, cx: &App) -> Vec<WorkflowStep> {
        self.workflow_steps
            .borrow()
            .iter()
            .map(|draft| draft.build_step(cx))
            .collect()
    }

    pub fn text_state(&self) -> Option<Entity<InputState>> {
        self.text_state.clone()
    }
}

fn text_editor_card(
    target_label: &str,
    kind: PowerUserKind,
    text_state: Entity<InputState>,
    on_commit: CommitActionFn,
    on_close: CloseActionFn,
    on_promote: Option<PromoteActionFn>,
    pal: Palette,
) -> gpui::Div {
    let heading = kind.heading();

    compact_panel(pal)
        .w(px(320.))
        .child(title(
            tr!("%{action} · %{key}", action => heading, key => target_label.to_string()),
            pal,
        ))
        .child(divider(pal))
        .child(
            v_flex()
                .p_2()
                .gap_2()
                .child(div().child(control_input(&text_state).cleanable(true)))
                .child(
                    h_flex()
                        .gap_2()
                        .justify_between()
                        .child(h_flex().when_some(on_promote, |row, promote| {
                            let text_state = text_state.clone();
                            row.child(
                                Button::new("editor-promote")
                                    .compact()
                                    .ghost()
                                    .label(tr!("Save to Custom Actions…"))
                                    .on_click(move |_e, window, cx| {
                                        let text = text_state.read(cx).value().to_string();
                                        let target = match kind {
                                            PowerUserKind::TypeText => ActionTarget::Text(text),
                                            PowerUserKind::RunAppleScript => {
                                                ActionTarget::AppleScript(text)
                                            }
                                            PowerUserKind::RunShellCommand => {
                                                ActionTarget::Shell(text)
                                            }
                                            PowerUserKind::Workflow => return,
                                        };
                                        (promote)(target, window, cx);
                                    }),
                            )
                        }))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("editor-cancel")
                                        .ghost()
                                        .label(tr!("Cancel"))
                                        .on_click(move |_e, window, cx| {
                                            (on_close)(window, cx);
                                        }),
                                )
                                .child(
                                    Button::new("editor-save")
                                        .primary()
                                        .label(tr!("Save"))
                                        .on_click(move |_e, window, cx| {
                                            let text = text_state.read(cx).value().to_string();
                                            let action = match kind {
                                                PowerUserKind::TypeText => Action::TypeText(text),
                                                PowerUserKind::RunAppleScript => {
                                                    Action::RunAppleScript(text)
                                                }
                                                PowerUserKind::RunShellCommand => {
                                                    Action::RunShellCommand(text)
                                                }
                                                PowerUserKind::Workflow => return,
                                            };
                                            (on_commit)(action, window, cx);
                                        }),
                                ),
                        ),
                ),
        )
}

#[expect(
    clippy::too_many_lines,
    reason = "assembles step list, adder bar, and step row interactions"
)]
pub fn workflow_steps_list_and_adder(
    steps_rc: &Rc<RefCell<Vec<WorkflowStepDraft>>>,
    pal: Palette,
    scroll_id: &'static str,
) -> gpui::Div {
    let borrowed = steps_rc.borrow();
    let is_empty = borrowed.is_empty();

    let rows: Vec<_> = borrowed
        .iter()
        .enumerate()
        .map(|(idx, draft)| {
            let (type_label, glyph, is_delay): (gpui::SharedString, &'static str, bool) =
                match &draft.step {
                    WorkflowStep::TypeText(_) => {
                        (tr!("Type Text"), "action-icons/keyboard.svg", false)
                    }
                    WorkflowStep::PressKey(_) => {
                        (tr!("Press Key"), "action-icons/keyboard.svg", false)
                    }
                    WorkflowStep::Delay { .. } => {
                        (tr!("Delay"), "action-icons/chevrons-right.svg", true)
                    }
                    WorkflowStep::RunAppleScript(_) => {
                        (tr!("AppleScript"), "action-icons/terminal.svg", false)
                    }
                    WorkflowStep::RunShellCommand(_) => {
                        (tr!("Shell"), "action-icons/terminal.svg", false)
                    }
                };
            let steps_remove = steps_rc.clone();

            v_flex()
                .gap_1()
                .p_1p5()
                .rounded(pal.control_radius)
                .border_1()
                .border_color(pal.border)
                .bg(pal.control)
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .items_center()
                                .gap_1p5()
                                .child(svg().path(glyph).size_4().text_color(pal.text_muted))
                                .child(
                                    div()
                                        .text_caption()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(pal.text_muted)
                                        .child(type_label),
                                ),
                        )
                        .child(
                            Button::new(format!("{scroll_id}-remove-step-{idx}"))
                                .compact()
                                .ghost()
                                .icon(IconName::Close)
                                .tooltip(tr!("Remove step"))
                                .on_click(move |_, window, _cx| {
                                    if idx < steps_remove.borrow().len() {
                                        steps_remove.borrow_mut().remove(idx);
                                        window.refresh();
                                    }
                                }),
                        ),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(control_input(&draft.input_state).cleanable(true)),
                        )
                        .when(is_delay, |row| {
                            row.child(
                                div()
                                    .flex_none()
                                    .text_caption()
                                    .text_color(pal.text_muted)
                                    .child(tr!("ms")),
                            )
                        }),
                )
        })
        .collect();

    v_flex()
        .gap_2()
        .child(v_flex().max_h(px(240.)).child(editor_scroll_list(
            scroll_id,
            if is_empty {
                vec![
                    div()
                        .py_3()
                        .px_2()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child(tr!("No steps configured. Add a step below.")),
                ]
            } else {
                rows
            },
        )))
        .child(divider(pal))
        .child(
            v_flex()
                .p_2()
                .gap_1p5()
                .child(
                    div()
                        .text_caption()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.text_muted)
                        .child(tr!("+ Add Step")),
                )
                .child(
                    h_flex().flex_wrap().gap_1().children(
                        WorkflowStepType::supported_on_current_platform()
                            .iter()
                            .map(|&step_type| {
                                let steps_add = steps_rc.clone();
                                Button::new(format!("{scroll_id}-add-{step_type:?}"))
                                    .compact()
                                    .ghost()
                                    .label(format!("+ {}", step_type.label()))
                                    .on_click(move |_, window, cx| {
                                        let step = step_type.default_step();
                                        let draft = WorkflowStepDraft::new(step, window, cx);
                                        steps_add.borrow_mut().push(draft);
                                        window.refresh();
                                    })
                            }),
                    ),
                ),
        )
}

fn workflow_editor_card(
    target_label: &str,
    steps_rc: &Rc<RefCell<Vec<WorkflowStepDraft>>>,
    on_commit: CommitActionFn,
    on_close: CloseActionFn,
    on_promote: Option<PromoteActionFn>,
    pal: Palette,
    _cx: &App,
) -> gpui::Div {
    let steps_save = steps_rc.clone();

    compact_panel(pal)
        .w(px(320.))
        .child(title(
            tr!("Workflow · %{target}", target => target_label.to_string()),
            pal,
        ))
        .child(divider(pal))
        .child(workflow_steps_list_and_adder(
            steps_rc,
            pal,
            "workflow-steps-scroll",
        ))
        .child(divider(pal))
        .child(
            h_flex()
                .p_2()
                .gap_2()
                .justify_between()
                .child(h_flex().when_some(on_promote, |row, promote| {
                    let steps_promote = steps_rc.clone();
                    row.child(
                        Button::new("wf-promote")
                            .compact()
                            .ghost()
                            .label(tr!("Save to Custom Actions…"))
                            .on_click(move |_e, window, cx| {
                                let steps = steps_promote
                                    .borrow()
                                    .iter()
                                    .map(|draft| draft.build_step(cx))
                                    .collect();
                                (promote)(ActionTarget::Workflow(steps), window, cx);
                            }),
                    )
                }))
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("wf-cancel")
                                .ghost()
                                .label(tr!("Cancel"))
                                .on_click(move |_e, window, cx| {
                                    (on_close)(window, cx);
                                }),
                        )
                        .child(
                            Button::new("wf-save")
                                .primary()
                                .label(tr!("Save Workflow"))
                                .on_click(move |_e, window, cx| {
                                    let steps = steps_save
                                        .borrow()
                                        .iter()
                                        .map(|draft| draft.build_step(cx))
                                        .collect();
                                    (on_commit)(Action::Workflow(steps), window, cx);
                                }),
                        ),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_editor_seed_only_uses_matching_power_user_action() {
        assert_eq!(
            text_editor_seed(
                Some(&Action::RunAppleScript(
                    "tell app \"Finder\" to activate".into()
                )),
                PowerUserKind::RunAppleScript,
            ),
            "tell app \"Finder\" to activate"
        );
        assert_eq!(
            text_editor_seed(
                Some(&Action::RunShellCommand("echo nope".into())),
                PowerUserKind::RunAppleScript,
            ),
            ""
        );
    }

    #[test]
    fn workflow_editor_seed_only_uses_workflow_action() {
        let steps = vec![WorkflowStep::TypeText("hello".into())];
        assert_eq!(
            workflow_editor_seed(Some(&Action::Workflow(steps.clone()))),
            steps
        );
        assert_eq!(
            workflow_editor_seed(Some(&Action::RunAppleScript(
                "display dialog \"Hello\"".into()
            ))),
            [] as [openlogi_core::binding::WorkflowStep; 0]
        );
    }

    #[test]
    fn supported_on_current_platform_matches_contract() {
        let supported = PowerUserKind::supported_on_current_platform();
        #[cfg(target_os = "macos")]
        {
            assert_eq!(
                supported,
                &[
                    PowerUserKind::TypeText,
                    PowerUserKind::RunAppleScript,
                    PowerUserKind::RunShellCommand,
                    PowerUserKind::Workflow,
                ]
            );
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(
                supported,
                &[PowerUserKind::RunShellCommand, PowerUserKind::Workflow]
            );
        }
    }

    #[test]
    fn power_user_kind_headings_are_non_empty() {
        for kind in PowerUserKind::supported_on_current_platform() {
            assert!(!kind.heading().is_empty());
        }
    }

    #[test]
    fn workflow_step_types_supported_and_defaults() {
        let supported = WorkflowStepType::supported_on_current_platform();
        assert_ne!(supported, []);
        for &step_type in supported {
            assert!(!step_type.label().is_empty());
            let default_step = step_type.default_step();
            match (step_type, default_step) {
                (WorkflowStepType::Delay, WorkflowStep::Delay { millis }) => {
                    assert_eq!(millis, 100);
                }
                (WorkflowStepType::TypeText, WorkflowStep::TypeText(_))
                | (WorkflowStepType::PressKey, WorkflowStep::PressKey(_))
                | (WorkflowStepType::RunAppleScript, WorkflowStep::RunAppleScript(_))
                | (WorkflowStepType::RunShellCommand, WorkflowStep::RunShellCommand(_)) => {}
                _ => panic!("Step type mismatch"),
            }
        }
    }

    #[test]
    fn power_user_editor_state_lifecycle() {
        let mut state = PowerUserEditorState::new();
        assert!(!state.is_open());
        assert_eq!(state.active_kind(), None);

        state.active_editor = Some(PowerUserKind::Workflow);
        assert!(state.is_open());
        assert_eq!(state.active_kind(), Some(PowerUserKind::Workflow));

        state.close();
        assert!(!state.is_open());
        assert_eq!(state.active_kind(), None);
    }

    struct TestView;
    impl gpui::Render for TestView {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            gpui::div()
        }
    }

    #[gpui::test]
    fn power_user_editor_state_workflow_manipulation(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let window = cx.open_window(gpui::size(gpui::px(200.0), gpui::px(200.0)), |_, _| {
            TestView
        });
        cx.run_until_parked();

        window
            .update(cx, |_, window, cx| {
                let mut state = PowerUserEditorState::new();
                assert!(!state.is_open());
                assert_eq!(state.active_kind(), None);

                state.open(PowerUserKind::Workflow, None, window, cx);
                assert!(state.is_open());
                assert_eq!(state.active_kind(), Some(PowerUserKind::Workflow));
                assert_eq!(state.workflow_steps_len(), 0);

                state.push_step(WorkflowStep::Delay { millis: 200 }, window, cx);
                assert_eq!(state.workflow_steps_len(), 1);

                state.push_step(WorkflowStep::RunShellCommand("echo 1".into()), window, cx);
                assert_eq!(state.workflow_steps_len(), 2);

                state.update_workflow_step(0, WorkflowStep::Delay { millis: 500 }, window, cx);

                let built = state.build_workflow_steps(cx);
                assert_eq!(built.len(), 2);
                assert_eq!(built[0], WorkflowStep::Delay { millis: 500 });
                assert_eq!(built[1], WorkflowStep::RunShellCommand("echo 1".into()));

                state.remove_step(0);
                assert_eq!(state.workflow_steps_len(), 1);

                state.open(PowerUserKind::RunShellCommand, None, window, cx);
                assert_eq!(state.active_kind(), Some(PowerUserKind::RunShellCommand));
                assert!(state.text_state().is_some());

                state.close();
                assert!(!state.is_open());
                assert_eq!(state.text_state(), None);
            })
            .unwrap();
    }
}
