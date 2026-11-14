//! Unified launch and shortcut target payload.

use serde::{Deserialize, Serialize};

use super::action::WorkflowStep;
use super::application_target::ApplicationTarget;
use super::key_combo::KeyCombo;

/// Target payload for a custom action: key chord, application/folder launch target,
/// typed text, script, shell command, or multi-step workflow.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionTarget {
    /// Replay or hold a key combo chord.
    Shortcut(KeyCombo),
    /// Launch an application, open a folder, or navigate a URL.
    Application(ApplicationTarget),
    /// Type a unicode string.
    Text(String),
    /// Run an AppleScript.
    AppleScript(String),
    /// Run a shell command.
    Shell(String),
    /// Execute a sequence of workflow steps.
    Workflow(Vec<WorkflowStep>),
}

impl ActionTarget {
    /// Human-readable rendered label.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Shortcut(combo) => combo.rendered_label(),
            Self::Application(target) => target.display_name().to_string(),
            Self::Text(text) => {
                if text.is_empty() {
                    "Type Text".to_string()
                } else {
                    format!("“{text}”")
                }
            }
            Self::AppleScript(script) => {
                if script.is_empty() {
                    "AppleScript".to_string()
                } else {
                    format!("AppleScript: {script}")
                }
            }
            Self::Shell(cmd) => {
                if cmd.is_empty() {
                    "Shell".to_string()
                } else {
                    format!("$ {cmd}")
                }
            }
            Self::Workflow(steps) => format!("Workflow ({} steps)", steps.len()),
        }
    }

    /// Short human-readable display name or summary of the target.
    #[must_use]
    pub fn display_name(&self) -> String {
        self.label()
    }
}
