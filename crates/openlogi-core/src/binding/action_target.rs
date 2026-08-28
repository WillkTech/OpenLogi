//! Unified launch and shortcut target payload.

use serde::{Deserialize, Serialize};

use super::application_target::ApplicationTarget;
use super::key_combo::KeyCombo;

/// Target payload for a custom action: either a key chord or an application/folder launch target.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionTarget {
    /// Replay or hold a key combo chord.
    Shortcut(KeyCombo),
    /// Launch an application, open a folder, or navigate a URL.
    Application(ApplicationTarget),
}

impl ActionTarget {
    /// Human-readable rendered label.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Shortcut(combo) => combo.rendered_label(),
            Self::Application(target) => target.display_name().to_string(),
        }
    }
}
