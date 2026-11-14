//! Shared action catalog library, row generation, and action picker component.

use std::rc::Rc;

use gpui::{
    App, Entity, ParentElement, Role, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, rgb, svg,
};
use gpui_component::{
    Icon, IconName, Selectable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::InputState,
    v_flex,
};
use openlogi_core::binding::{Action, Category};

use crate::features::keyboard::editors::PowerUserKind;
use crate::state::AppState;
use crate::ui::action::action_icon_path;
use crate::ui::components::{MenuRow, control_input};
use crate::ui::theme::{ACCENT_BLUE, Palette, Typography as _};

/// Commit callback invoked when an action row is clicked.
pub(crate) type PickFn = Rc<dyn Fn(Action, &mut Window, &mut App)>;

/// Callback invoked when a pencil edit button on a saved custom action row is clicked.
pub(crate) type EditFn =
    Rc<dyn Fn(openlogi_core::config::CustomShortcutEntry, &mut Window, &mut App)>;

/// Callback invoked when "+ New Shortcut" is clicked.
pub(crate) type NewShortcutFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Callback invoked when a Power User action item is clicked.
pub(crate) type OpenPowerUserFn = Rc<dyn Fn(PowerUserKind, &mut Window, &mut App)>;

/// The action catalog grouped by [`Category`], preserving catalog order within
/// each group and first-seen order across groups. Includes user-created custom shortcuts.
#[expect(
    dead_code,
    reason = "convenience wrapper for grouped_catalog_filtered without predicate"
)]
pub(crate) fn grouped_catalog(cx: &App) -> Vec<(Category, Vec<Action>)> {
    grouped_catalog_filtered(cx, |_| true)
}

/// Filtered catalog grouped by [`Category`].
pub(crate) fn grouped_catalog_filtered(
    cx: &App,
    filter: impl Fn(&Action) -> bool,
) -> Vec<(Category, Vec<Action>)> {
    let mut sections: Vec<(Category, Vec<Action>)> = Vec::new();

    if let Some(state) = AppState::try_read(cx) {
        let custom_actions: Vec<Action> = state
            .custom_shortcuts()
            .iter()
            .map(|entry| Action::SavedShortcut {
                id: entry.id.clone(),
                name: entry.name.clone(),
                target: entry.target.clone(),
                icon: entry.icon,
            })
            .filter(|action| filter(action))
            .collect();
        if !custom_actions.is_empty() {
            sections.push((Category::CustomShortcuts, custom_actions));
        }
    }

    for action in Action::catalog() {
        if !filter(&action) {
            continue;
        }
        let category = action.category();
        if let Some(section) = sections
            .iter_mut()
            .find(|(existing, _)| *existing == category)
        {
            section.1.push(action);
        } else {
            sections.push((category, vec![action]));
        }
    }
    sections
}

/// Build every category-grouped action row without filtering.
#[expect(
    dead_code,
    reason = "convenience wrapper for action_rows_matching without query"
)]
pub(crate) fn action_rows(
    id_prefix: &'static str,
    current: Option<&Action>,
    on_pick: &PickFn,
    on_edit: Option<&EditFn>,
    pal: Palette,
    cx: &App,
) -> Vec<gpui::Div> {
    action_rows_matching(id_prefix, current, "", on_pick, on_edit, pal, cx)
}

/// Build action rows filtered by localized action or category name.
pub(crate) fn action_rows_matching(
    id_prefix: &'static str,
    current: Option<&Action>,
    query: &str,
    on_pick: &PickFn,
    on_edit: Option<&EditFn>,
    pal: Palette,
    cx: &App,
) -> Vec<gpui::Div> {
    action_rows_matching_filtered(
        id_prefix,
        current,
        query,
        |_| true,
        on_pick,
        on_edit,
        pal,
        cx,
    )
}

/// Build action rows filtered by query and predicate.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "filters and renders action catalog picker rows"
)]
pub(crate) fn action_rows_matching_filtered(
    id_prefix: &'static str,
    current: Option<&Action>,
    query: &str,
    filter: impl Fn(&Action) -> bool,
    on_pick: &PickFn,
    on_edit: Option<&EditFn>,
    pal: Palette,
    cx: &App,
) -> Vec<gpui::Div> {
    let query = query.trim().to_lowercase();
    let mut catalog_index = 0usize;
    let mut sections = Vec::new();
    for (category, actions) in grouped_catalog_filtered(cx, filter) {
        let category_label = rust_i18n::t!(category.label());
        let category_matches = category_label.to_lowercase().contains(&query);
        // Number the full catalog before filtering so typing in the search box
        // never changes an action row's element identity.
        let actions: Vec<(usize, Action)> = actions
            .into_iter()
            .map(|action| {
                let key = catalog_index;
                catalog_index += 1;
                (key, action)
            })
            .filter(|(_, action)| {
                query.is_empty()
                    || category_matches
                    || rust_i18n::t!(action.label())
                        .to_lowercase()
                        .contains(&query)
                    || action.label().to_lowercase().contains(&query)
            })
            .collect();
        if actions.is_empty() {
            continue;
        }
        sections.push(
            v_flex()
                .child(editor_section(category_label.into_owned(), pal))
                .children(actions.into_iter().map(|(action_key, action)| {
                    let selected = current == Some(&action);
                    let label = tr!(action.label());
                    let accessible_label = label.clone();
                    let icon_path = action_icon_path(&action);
                    let is_custom = matches!(&action, Action::SavedShortcut { .. });
                    let custom_entry = if let Action::SavedShortcut {
                        id,
                        name,
                        target,
                        icon,
                    } = &action
                    {
                        Some(openlogi_core::config::CustomShortcutEntry {
                            id: id.clone(),
                            name: name.clone(),
                            target: target.clone(),
                            icon: *icon,
                        })
                    } else {
                        None
                    };
                    let on_pick = on_pick.clone();
                    let on_edit = on_edit.cloned();
                    let mut row = MenuRow::new((id_prefix, action_key))
                        .selected(selected)
                        .role(Role::MenuItem)
                        .aria_label(accessible_label);
                    if matches!(&action, Action::None) {
                        row = row.tooltip(tr!(
                            "Captures the input and does nothing (use Reset to default for native behavior)"
                        ));
                    }
                    row.child(
                        h_flex()
                            .flex_1()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        svg()
                                            .path(icon_path)
                                            .size_4()
                                            .flex_none()
                                            .text_color(pal.text_muted),
                                    )
                                    .child(div().child(label)),
                            )
                            .when_some(custom_entry, |parent, entry| {
                                let id = entry.id.clone();
                                let edit_entry = entry.clone();
                                parent.child(
                                    h_flex()
                                        .items_center()
                                        .gap_1()
                                        .when(selected, |controls| {
                                            controls.child(
                                                Icon::new(IconName::Check)
                                                    .size_3()
                                                    .text_color(rgb(ACCENT_BLUE)),
                                            )
                                        })
                                        .when_some(on_edit, |controls, on_edit| {
                                            controls
                                                .child(
                                                    Button::new((
                                                        "edit-custom-shortcut",
                                                        action_key,
                                                    ))
                                                    .icon(
                                                        Icon::empty()
                                                            .path("action-icons/pencil.svg")
                                                            .text_color(pal.text_muted),
                                                    )
                                                    .compact()
                                                    .ghost()
                                                    .tooltip(tr!("Edit shortcut"))
                                                    .on_click(move |_event, window, cx| {
                                                        (on_edit)(edit_entry.clone(), window, cx);
                                                    }),
                                                )
                                                .child(
                                                    Button::new((
                                                        "delete-custom-shortcut",
                                                        action_key,
                                                    ))
                                                    .icon(IconName::Close)
                                                    .compact()
                                                    .ghost()
                                                    .tooltip(tr!("Delete shortcut"))
                                                    .on_click(move |_event, _, cx| {
                                                        let id = id.clone();
                                                        AppState::update(cx, |state, cx| {
                                                            state.delete_custom_shortcut(&id, cx);
                                                        });
                                                    }),
                                                )
                                        }),
                                )
                            }),
                    )
                    .when(selected && !is_custom, |row| {
                        row.child(
                            Icon::new(IconName::Check)
                                .size_3()
                                .text_color(rgb(ACCENT_BLUE)),
                        )
                    })
                    .on_click(move |_event, window, cx| (on_pick)(action.clone(), window, cx))
                })),
        );
    }
    sections
}

/// A group heading inset to line up with the rows under it.
pub(crate) fn editor_section(label: impl Into<gpui::SharedString>, pal: Palette) -> gpui::Div {
    crate::ui::section::section_label(label, pal)
        .w_full()
        .px_2()
        .pt_2()
        .pb_0p5()
}

/// Optional Power User section builder.
pub(crate) fn power_user_section(
    id_prefix: &'static str,
    current: Option<&Action>,
    query: &str,
    on_open_power_user: &OpenPowerUserFn,
    pal: Palette,
) -> Option<gpui::Div> {
    let power_user_actions: &[(PowerUserKind, &str, &'static str)] = &[
        (
            PowerUserKind::TypeText,
            "Type Text…",
            "action-icons/keyboard.svg",
        ),
        (
            PowerUserKind::RunAppleScript,
            "Run AppleScript…",
            "action-icons/terminal.svg",
        ),
        (
            PowerUserKind::RunShellCommand,
            "Run Shell Command…",
            "action-icons/terminal.svg",
        ),
        (
            PowerUserKind::Workflow,
            "Workflow…",
            "action-icons/list-checks.svg",
        ),
    ];

    let supported_kinds = PowerUserKind::supported_on_current_platform();
    let q = query.trim().to_lowercase();
    let power_user_label = tr!("Power User");
    let category_matches = !q.is_empty()
        && (power_user_label.to_lowercase().contains(&q) || "power user".contains(&q));

    let filtered_power_user: Vec<_> = power_user_actions
        .iter()
        .filter(|(kind, _, _)| supported_kinds.contains(kind))
        .filter(|(_, label, _)| {
            let label_str: &str = label;
            q.is_empty()
                || category_matches
                || rust_i18n::t!(label_str).to_lowercase().contains(&q)
                || label.to_lowercase().contains(&q)
        })
        .collect();

    if filtered_power_user.is_empty() {
        return None;
    }

    Some(
        v_flex()
            .child(editor_section(power_user_label, pal))
            .children(filtered_power_user.into_iter().enumerate().map(
                |(idx, (kind, label, icon_path))| {
                    let kind = *kind;
                    let on_open_power_user = on_open_power_user.clone();
                    let selected = matches!(
                        (current, kind),
                        (Some(Action::TypeText(_)), PowerUserKind::TypeText)
                            | (
                                Some(Action::RunAppleScript(_)),
                                PowerUserKind::RunAppleScript
                            )
                            | (
                                Some(Action::RunShellCommand(_)),
                                PowerUserKind::RunShellCommand
                            )
                            | (Some(Action::Workflow(_)), PowerUserKind::Workflow)
                    );
                    MenuRow::new(format!("{id_prefix}-power-{idx}"))
                        .selected(selected)
                        .role(Role::MenuItem)
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    svg()
                                        .path(*icon_path)
                                        .size_4()
                                        .flex_none()
                                        .text_color(pal.text_muted),
                                )
                                .child(div().child(tr!(*label))),
                        )
                        .when(selected, |s| {
                            s.child(
                                Icon::new(IconName::Check)
                                    .size_3()
                                    .text_color(rgb(ACCENT_BLUE)),
                            )
                        })
                        .on_click(move |_ev, window, cx| {
                            (on_open_power_user)(kind, window, cx);
                        })
                },
            )),
    )
}

/// Configurable action library component builder.
pub(crate) struct ActionLibrary<'a, F = fn(&Action) -> bool> {
    id_prefix: &'static str,
    current: Option<&'a Action>,
    query: &'a str,
    filter: F,
    on_pick: &'a PickFn,
    on_edit: Option<&'a EditFn>,
    on_new_shortcut: Option<NewShortcutFn>,
    on_open_power_user: Option<OpenPowerUserFn>,
    search_state: Option<&'a Entity<InputState>>,
}

impl<'a> ActionLibrary<'a, fn(&Action) -> bool> {
    pub(crate) fn new(id_prefix: &'static str, on_pick: &'a PickFn) -> Self {
        Self {
            id_prefix,
            current: None,
            query: "",
            filter: |_| true,
            on_pick,
            on_edit: None,
            on_new_shortcut: None,
            on_open_power_user: None,
            search_state: None,
        }
    }
}

impl<'a, F: Fn(&Action) -> bool> ActionLibrary<'a, F> {
    pub(crate) fn current(mut self, current: Option<&'a Action>) -> Self {
        self.current = current;
        self
    }

    pub(crate) fn query(mut self, query: &'a str) -> Self {
        self.query = query;
        self
    }

    pub(crate) fn filter<F2: Fn(&Action) -> bool>(self, filter: F2) -> ActionLibrary<'a, F2> {
        ActionLibrary {
            id_prefix: self.id_prefix,
            current: self.current,
            query: self.query,
            filter,
            on_pick: self.on_pick,
            on_edit: self.on_edit,
            on_new_shortcut: self.on_new_shortcut,
            on_open_power_user: self.on_open_power_user,
            search_state: self.search_state,
        }
    }

    pub(crate) fn on_edit(mut self, on_edit: Option<&'a EditFn>) -> Self {
        self.on_edit = on_edit;
        self
    }

    pub(crate) fn on_new_shortcut(mut self, on_new_shortcut: Option<NewShortcutFn>) -> Self {
        self.on_new_shortcut = on_new_shortcut;
        self
    }

    pub(crate) fn on_open_power_user(
        mut self,
        on_open_power_user: Option<OpenPowerUserFn>,
    ) -> Self {
        self.on_open_power_user = on_open_power_user;
        self
    }

    pub(crate) fn search_state(mut self, search: Option<&'a Entity<InputState>>) -> Self {
        self.search_state = search;
        self
    }

    /// Build only the list of action rows (with optional power user section appended).
    pub(crate) fn build_rows(&self, pal: Palette, cx: &App) -> Vec<gpui::Div> {
        let mut rows = action_rows_matching_filtered(
            self.id_prefix,
            self.current,
            self.query,
            &self.filter,
            self.on_pick,
            self.on_edit,
            pal,
            cx,
        );
        if let Some(on_open_power_user) = &self.on_open_power_user
            && let Some(sec) = power_user_section(
                self.id_prefix,
                self.current,
                self.query,
                on_open_power_user,
                pal,
            )
        {
            rows.push(sec);
        }
        rows
    }

    /// Renders the complete action library panel:
    /// Optional header bar (`tr!("Actions")` + `+ New Shortcut`),
    /// optional `control_input(search)`,
    /// empty-state message (`tr!("No actions found")`) if rows are empty,
    /// and the rows.
    pub(crate) fn render(&self, pal: Palette, cx: &App) -> gpui::Div {
        let rows = self.build_rows(pal, cx);
        let on_new = self.on_new_shortcut.clone();

        v_flex()
            .gap_2()
            .pt_1()
            .when_some(on_new, |parent, on_new| {
                parent.child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(editor_section(tr!("Actions"), pal))
                        .child(
                            Button::new("new-custom-shortcut")
                                .compact()
                                .ghost()
                                .label(tr!("+ New Shortcut"))
                                .on_click(move |_, window, cx| {
                                    (on_new)(window, cx);
                                }),
                        ),
                )
            })
            .when_some(self.search_state, |parent, search| {
                parent.child(control_input(search).cleanable(true))
            })
            .child(
                v_flex()
                    .gap_0p5()
                    .when(rows.is_empty(), |list| {
                        list.child(
                            div()
                                .py_3()
                                .text_body()
                                .text_color(pal.text_muted)
                                .child(tr!("No actions found")),
                        )
                    })
                    .children(rows),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openlogi_core::binding::{
        ActionRingIcon, ActionTarget, ApplicationTarget, KeyCombo, RingAction,
    };

    #[test]
    fn ring_action_filter_excludes_none_and_show_actions_ring() {
        RingAction::new(Action::None).unwrap_err();
        RingAction::new(Action::ShowActionsRing).unwrap_err();

        // Valid catalog actions
        RingAction::new(Action::Copy).unwrap();
        RingAction::new(Action::Paste).unwrap();

        // Valid custom shortcuts
        let custom = Action::SavedShortcut {
            id: "test-id".into(),
            name: "Test".into(),
            target: ActionTarget::Shortcut("ctrl+c".parse::<KeyCombo>().unwrap()),
            icon: None,
        };
        RingAction::new(custom).unwrap();
    }

    #[test]
    fn action_icon_path_consistency() {
        assert_eq!(action_icon_path(&Action::None), "action-icons/ban.svg");
        assert_eq!(
            action_icon_path(&Action::LeftClick),
            "action-icons/mouse-pointer-click.svg"
        );
        assert_eq!(action_icon_path(&Action::Copy), "action-icons/copy.svg");
        assert_eq!(
            action_icon_path(&Action::TypeText("hello".into())),
            "action-icons/keyboard.svg"
        );
        assert_eq!(
            action_icon_path(&Action::RunShellCommand("ls".into())),
            "action-icons/terminal.svg"
        );

        let shortcut_target = Action::SavedShortcut {
            id: "1".into(),
            name: "Kbd".into(),
            target: ActionTarget::Shortcut("cmd+a".parse::<KeyCombo>().unwrap()),
            icon: None,
        };
        assert_eq!(
            action_icon_path(&shortcut_target),
            "action-icons/keyboard.svg"
        );

        let app_target = Action::SavedShortcut {
            id: "2".into(),
            name: "App".into(),
            target: ActionTarget::Application(ApplicationTarget::new("/bin/ls", "").unwrap()),
            icon: None,
        };
        assert_eq!(action_icon_path(&app_target), "action-icons/grid-3x3.svg");

        let custom_icon = Action::SavedShortcut {
            id: "3".into(),
            name: "CustomIcon".into(),
            target: ActionTarget::Shortcut("cmd+b".parse::<KeyCombo>().unwrap()),
            icon: Some(ActionRingIcon::Terminal),
        };
        assert_eq!(
            action_icon_path(&custom_icon),
            ActionRingIcon::Terminal.asset_path()
        );
    }
}
