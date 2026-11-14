//! Display helpers for the shared action vocabulary.

use gpui::SharedString;
use openlogi_core::binding::{Action, ActionRingIcon, ActionTarget};

/// Localized label for an action, including its dynamic payload.
pub(crate) fn localized_action_label(action: &Action) -> SharedString {
    match action {
        Action::SetDpiPreset(index) => {
            tr!("DPI Preset %{index}", index => (index + 1).to_string())
        }
        Action::CustomShortcut(combo) => combo.rendered_label().into(),
        Action::HoldShortcut(combo) => {
            tr!("Hold %{chord}", chord => combo.rendered_label())
        }
        _ => tr!(action.label()),
    }
}

/// Icon for a gesture-mode button's five-direction summary.
pub(crate) const GESTURE_BUTTON_ICON: &str = "action-icons/move.svg";

/// Asset path of the vendored glyph for an action. Exhaustive so a new action
/// must deliberately choose an icon.
pub(crate) fn action_icon_path(action: &Action) -> &'static str {
    match action {
        Action::None => "action-icons/ban.svg",
        Action::LeftClick | Action::DoubleLeftClick | Action::RightClick => {
            "action-icons/mouse-pointer-click.svg"
        }
        Action::MiddleClick => "action-icons/mouse.svg",
        Action::MouseBack => "action-icons/circle-arrow-left.svg",
        Action::MouseForward => "action-icons/circle-arrow-right.svg",
        Action::Copy => "action-icons/copy.svg",
        Action::Paste => "action-icons/clipboard-paste.svg",
        Action::Cut => "action-icons/scissors.svg",
        Action::Undo => "action-icons/undo-2.svg",
        Action::Redo => "action-icons/redo-2.svg",
        Action::SelectAll | Action::Workflow(_) => "action-icons/list-checks.svg",
        Action::Find => "action-icons/search.svg",
        Action::Save => "action-icons/save.svg",
        Action::BrowserBack => "action-icons/arrow-left.svg",
        Action::BrowserForward => "action-icons/arrow-right.svg",
        Action::NewTab => "action-icons/square-plus.svg",
        Action::CloseTab => "action-icons/square-x.svg",
        Action::ReopenTab => "action-icons/rotate-ccw.svg",
        Action::NextTab => "action-icons/chevron-right.svg",
        Action::PrevTab => "action-icons/chevron-left.svg",
        Action::ReloadPage => "action-icons/rotate-cw.svg",
        Action::MissionControl | Action::ShowActionsRing => "action-icons/layout-grid.svg",
        Action::AppExpose => "action-icons/layers.svg",
        Action::PreviousDesktop => "action-icons/square-arrow-left.svg",
        Action::NextDesktop => "action-icons/square-arrow-right.svg",
        Action::ShowDesktop => "action-icons/monitor.svg",
        Action::LaunchpadShow | Action::OpenApplication(_) => "action-icons/grid-3x3.svg",
        Action::LockScreen => "action-icons/lock.svg",
        Action::Screenshot | Action::CaptureRegion => "action-icons/camera.svg",
        Action::Sleep => "action-icons/moon.svg",
        Action::PlayPause => "action-icons/play.svg",
        Action::NextTrack => "action-icons/skip-forward.svg",
        Action::PrevTrack => "action-icons/skip-back.svg",
        Action::VolumeUp => "action-icons/volume-2.svg",
        Action::VolumeDown => "action-icons/volume-1.svg",
        Action::MuteVolume => "action-icons/volume-x.svg",
        Action::CycleDpiPresets | Action::SetDpiPreset(_) => "action-icons/gauge.svg",
        Action::ToggleSmartShift => "action-icons/refresh-cw.svg",
        Action::ScrollUp => "action-icons/chevrons-up.svg",
        Action::ScrollDown => "action-icons/chevrons-down.svg",
        Action::HorizontalScrollLeft => "action-icons/chevrons-left.svg",
        Action::HorizontalScrollRight => "action-icons/chevrons-right.svg",
        Action::SavedShortcut { icon, target, .. } => icon.map_or_else(
            || match target {
                ActionTarget::Shortcut(_) | ActionTarget::Text(_) => "action-icons/keyboard.svg",
                ActionTarget::Application(_) => "action-icons/grid-3x3.svg",
                ActionTarget::AppleScript(_) | ActionTarget::Shell(_) => {
                    "action-icons/terminal.svg"
                }
                ActionTarget::Workflow(_) => "action-icons/list-checks.svg",
            },
            ActionRingIcon::asset_path,
        ),
        Action::CustomShortcut(_) | Action::HoldShortcut(_) | Action::TypeText(_) => {
            "action-icons/keyboard.svg"
        }
        Action::RunAppleScript(_) | Action::RunShellCommand(_) => "action-icons/terminal.svg",
    }
}
