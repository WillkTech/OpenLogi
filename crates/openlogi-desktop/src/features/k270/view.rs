//! The dedicated Media keys tab for the Logitech Wireless Keyboard K270.

use std::rc::Rc;

use gpui::{
    App, AppContext as _, Context, Entity, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, Role, StatefulInteractiveElement as _, Styled, Subscription, Window,
    div, prelude::FluentBuilder as _, px, rgb, svg,
};
use gpui_base::Disableable as _;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{InputEvent, InputState},
    v_flex,
};
use openlogi_core::binding::Action;
use openlogi_core::device::k270::K270MediaKey;

use super::geometry::{K270_ASPECT_RATIO, K270_HOTSPOTS, K270Hotspot};
use crate::app_assets::K270_MEDIAKEYS;
use crate::features::mouse::picker::{EditFn, PickFn, editor_scroll_list};
use crate::state::{AppState, DeviceRecord, StateEvent};
use crate::ui::action::{action_icon_path, localized_action_label};
use crate::ui::action_library::{ActionLibrary, OpenPowerUserFn, editor_section};
use crate::ui::components::{control_button, control_input};
use crate::ui::theme::{self, ACCENT_BLUE, Palette, Typography as _};

/// Target max render width of the K270 media keys diagram.
const DIAGRAM_MAX_W: f32 = 760.0;
/// Inspector and picker panel width.
const INSPECTOR_W: f32 = 760.0;

/// The dedicated Media keys tab view.
pub struct K270MediaKeysView {
    /// Currently selected key. Defaults to Play/Pause on startup.
    selected_key: Option<K270MediaKey>,
    /// Filter string for action picker.
    search_query: String,
    /// Search input entity.
    search_state: Option<Entity<InputState>>,
    shortcut_builder: Option<Entity<crate::ui::shortcut_builder::ShortcutBuilderModal>>,
    pub(crate) power_user: crate::ui::power_user::PowerUserEditorState,
    _subscription: Subscription,
}

impl K270MediaKeysView {
    /// Create a new K270 media keys configuration view.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_state =
            cx.new(|cx| InputState::new(window, cx).placeholder(tr!("Search actions…")));

        let search_sub = cx.subscribe(&search_state, |this, _input, event: &InputEvent, cx| {
            if let InputEvent::Change = event
                && let Some(state) = &this.search_state
            {
                this.search_query = state.read(cx).value().to_string();
                cx.notify();
            }
        });

        let state_sub = cx.subscribe(
            &AppState::global(cx),
            |_this, _state, event: &StateEvent, cx| match event {
                StateEvent::DeviceSelected(_)
                | StateEvent::DeviceConfigChanged(_)
                | StateEvent::InventoryChanged
                | StateEvent::CustomShortcutsChanged => {
                    cx.notify();
                }
                _ => {}
            },
        );

        Self {
            selected_key: Some(K270MediaKey::PlayPause),
            search_query: String::new(),
            search_state: Some(search_state),
            shortcut_builder: None,
            power_user: crate::ui::power_user::PowerUserEditorState::default(),
            _subscription: Subscription::new(move || {
                drop(search_sub);
                drop(state_sub);
            }),
        }
    }

    /// Open shortcut builder modal pre-filled with target and icon.
    pub fn open_shortcut_builder_with(
        &mut self,
        existing_id: Option<String>,
        name: &str,
        target: Option<&openlogi_core::binding::ActionTarget>,
        icon: Option<openlogi_core::binding::ActionRingIcon>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity();
        let modal = crate::ui::shortcut_builder::ShortcutBuilderModal::open_for_app(
            existing_id,
            name,
            target,
            icon,
            move |_window, cx| {
                view.update(cx, |view, cx| {
                    view.close_shortcut_builder();
                    cx.notify();
                });
            },
            window,
            cx,
        );
        self.shortcut_builder = Some(modal);
    }

    /// Open blank shortcut builder modal.
    pub fn open_shortcut_builder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_shortcut_builder_with(None, "", None, None, window, cx);
    }

    /// Close the shortcut builder modal.
    pub fn close_shortcut_builder(&mut self) {
        self.shortcut_builder = None;
    }

    /// Select a key for inspection and editing.
    pub fn select_key(&mut self, key: K270MediaKey, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_key != Some(key) {
            self.selected_key = Some(key);
            self.power_user.close();
            if let Some(state) = &self.search_state {
                state.update(cx, |input, cx| {
                    input.set_value("", window, cx);
                });
            }
            self.search_query.clear();
            cx.notify();
        }
    }

    /// Return the currently selected key.
    #[must_use]
    #[expect(dead_code, reason = "selected key query API for K270 Media Keys view")]
    pub fn selected_key(&self) -> Option<K270MediaKey> {
        self.selected_key
    }
}

impl Render for K270MediaKeysView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = theme::palette(cx);
        let selected = self.selected_key;
        let view_entity = cx.entity();

        div()
            .relative()
            .w_full()
            .child(
                v_flex()
                    .w_full()
                    .items_center()
                    .gap_6()
                    .py_6()
                    .px_4()
                    // 1. Header & description
                    .child(
                        v_flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_heading()
                                    .child(tr!("Media keys")),
                            )
                            .child(
                                div()
                                    .text_caption()
                                    .text_color(pal.text_muted)
                                    .child(tr!(
                                        "Customize the eight dedicated media and shortcut keys on your Wireless Keyboard K270."
                                    )),
                            ),
                    )
                    // 2. Interactive keyboard diagram
                    .child(render_diagram(selected, &view_entity, pal, cx))
                    // 3. Key assignment editor & inspector
                    .when_some(selected, |parent, key| {
                        parent.child(render_inspector(key, self, pal, cx))
                    })
                    // 4. Reset all keys section
                    .child(render_reset_all_section(pal)),
            )
            .when_some(self.shortcut_builder.clone(), |parent, modal_entity| {
                parent.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(gpui::rgba(0x0000_0088))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(div().w(px(460.0)).max_w(px(520.0)).child(modal_entity)),
                )
            })
    }
}

/// Render the interactive K270 diagram and hotspots.
fn render_diagram(
    selected: Option<K270MediaKey>,
    view: &Entity<K270MediaKeysView>,
    pal: Palette,
    cx: &mut Context<K270MediaKeysView>,
) -> impl IntoElement {
    let diagram_w = DIAGRAM_MAX_W;
    let diagram_h = diagram_w / K270_ASPECT_RATIO;
    let app_state = AppState::try_read(cx);

    v_flex().w(px(diagram_w)).items_center().gap_2().child(
        div()
            .id("k270-mediakeys-diagram-container")
            .relative()
            .w(px(diagram_w))
            .h(px(diagram_h))
            .bg(pal.panel)
            .rounded(pal.card_radius)
            .border_1()
            .border_color(pal.border)
            .overflow_hidden()
            // Base vector diagram rendering from SVG
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .child(
                        svg()
                            .path(K270_MEDIAKEYS)
                            .size_full()
                            .text_color(pal.text_primary),
                    ),
            )
            // Hotspot overlays for the 8 keys
            .children(K270_HOTSPOTS.iter().map(|spot| {
                render_hotspot(spot, diagram_w, diagram_h, selected, app_state, view)
            })),
    )
}

/// Render a single interactive hotspot.
fn render_hotspot(
    spot: &K270Hotspot,
    diagram_w: f32,
    diagram_h: f32,
    selected: Option<K270MediaKey>,
    app_state: Option<&AppState>,
    view: &Entity<K270MediaKeysView>,
) -> impl IntoElement {
    let (left, top, w, h) = spot.rect(diagram_w, diagram_h);
    let is_selected = selected == Some(spot.key);
    let is_customized = app_state.is_some_and(|s| s.is_k270_media_key_customized(spot.key));
    let effective_action = app_state.map_or_else(
        || spot.key.default_action(),
        |s| s.effective_k270_media_key_action(spot.key),
    );

    let key = spot.key;
    let view_click = view.clone();

    let accessible_label = format!(
        "{}: {}",
        key.display_name(),
        localized_action_label(&effective_action)
    );

    div()
        .id(("k270-key-region", spot.key as usize))
        .role(Role::Button)
        .aria_label(accessible_label)
        .absolute()
        .left(px(left))
        .top(px(top))
        .w(px(w))
        .h(px(h))
        .cursor_pointer()
        .rounded_sm()
        .when(is_selected, |this| {
            this.border_2()
                .border_color(rgb(ACCENT_BLUE))
                .bg(theme::accent_tint())
        })
        .when(!is_selected, |this| {
            this.border_1()
                .border_color(gpui::transparent_black())
                .hover(|s| {
                    s.border_color(rgb(ACCENT_BLUE))
                        .bg(gpui::hsla(0.58, 0.9, 0.5, 0.08))
                })
        })
        // Customization indicator dot (small dot at top-right)
        .when(is_customized, |this| {
            this.child(
                div()
                    .absolute()
                    .top(px(2.))
                    .right(px(3.))
                    .size(px(5.))
                    .rounded_full()
                    .bg(rgb(ACCENT_BLUE)),
            )
        })
        .on_click(move |_event, window, cx| {
            view_click.update(cx, |this, cx| {
                this.select_key(key, window, cx);
            });
        })
}

fn commit_media_key(key: K270MediaKey, action: Option<Action>, cx: &mut App) {
    AppState::update(cx, |state, cx| {
        let dev_key = state.current_record().map(DeviceRecord::device_key);
        state.commit_k270_media_key(key, action);
        if let Some(dev_key) = dev_key {
            cx.emit(StateEvent::DeviceConfigChanged(dev_key));
        }
    });
}

fn reset_all_media_keys(cx: &mut App) {
    AppState::update(cx, |state, cx| {
        let dev_key = state.current_record().map(DeviceRecord::device_key);
        state.reset_k270_media_keys();
        if let Some(dev_key) = dev_key {
            cx.emit(StateEvent::DeviceConfigChanged(dev_key));
        }
    });
}

/// Render the key inspector panel and action picker for `key`.
#[expect(
    clippy::too_many_lines,
    reason = "assembles header, action display, hardware notes, search, and picker list"
)]
fn render_inspector(
    key: K270MediaKey,
    view: &mut K270MediaKeysView,
    pal: Palette,
    cx: &mut Context<K270MediaKeysView>,
) -> impl IntoElement {
    if view.power_user.is_open() {
        let view_entity = cx.entity();
        let view_close = view_entity.clone();
        let view_commit = view_entity.clone();
        let view_promote = view_entity.clone();
        let on_promote: crate::ui::power_user::PromoteActionFn =
            std::sync::Arc::new(move |target, window, cx| {
                view_promote.update(cx, |v, cx| {
                    v.power_user.close();
                    v.open_shortcut_builder_with(None, "", Some(&target), None, window, cx);
                    cx.notify();
                });
            });
        return view
            .power_user
            .render_card_with_promote(
                &tr!(key.label_key()),
                move |action, _window, cx| {
                    commit_media_key(key, Some(action), cx);
                    view_commit.update(cx, |v, cx| {
                        v.power_user.close();
                        cx.notify();
                    });
                },
                move |_window, cx| {
                    view_close.update(cx, |v, cx| {
                        v.power_user.close();
                        cx.notify();
                    });
                },
                Some(on_promote),
                pal,
                cx,
            )
            .w(px(INSPECTOR_W));
    }

    let app_state = AppState::try_read(cx);
    let is_customized = app_state.is_some_and(|s| s.is_k270_media_key_customized(key));
    let effective_action = app_state.map_or_else(
        || key.default_action(),
        |s| s.effective_k270_media_key_action(key),
    );
    let action_label = localized_action_label(&effective_action);

    let on_pick: PickFn = Rc::new(move |action, _window, cx| {
        commit_media_key(key, Some(action), cx);
    });

    let view_entity = cx.entity();
    let view_edit = view_entity.clone();
    let on_edit: EditFn = Rc::new(move |entry, window, cx| {
        view_edit.update(cx, |view, cx| {
            view.open_shortcut_builder_with(
                Some(entry.id),
                &entry.name,
                Some(&entry.target),
                entry.icon,
                window,
                cx,
            );
            cx.notify();
        });
    });
    let view_new = view_entity.clone();
    let view_pw = view_entity.clone();
    let current_action_for_pw = effective_action.clone();
    let on_open_power_user: OpenPowerUserFn = Rc::new(move |kind, window, cx| {
        view_pw.update(cx, |view, cx| {
            view.power_user
                .open(kind, Some(&current_action_for_pw), window, cx);
            cx.notify();
        });
    });

    let query = view.search_query.clone();

    v_flex()
        .w(px(INSPECTOR_W))
        .bg(pal.panel)
        .border_1()
        .border_color(pal.border)
        .rounded(pal.card_radius)
        .p_4()
        .gap_4()
        // Header row: Key name, state badge, and Reset button
        .child(
            h_flex()
                .w_full()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .size_8()
                                .rounded_md()
                                .bg(pal.control)
                                .border_1()
                                .border_color(pal.border)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg()
                                        .path(key.icon_path())
                                        .size_4()
                                        .text_color(pal.text_primary),
                                ),
                        )
                        .child(
                            v_flex()
                                .child(
                                    div()
                                        .text_heading()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(tr!(key.label_key())),
                                )
                                .child(
                                    div()
                                        .text_caption()
                                        .text_color(pal.text_muted)
                                        .child(tr!("Current action")),
                                ),
                        ),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        // Status badge
                        .when(is_customized, |this| {
                            this.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .text_caption()
                                    .font_weight(FontWeight::MEDIUM)
                                    .bg(theme::accent_tint())
                                    .text_color(rgb(ACCENT_BLUE))
                                    .child(tr!("Customized")),
                            )
                        })
                        .when(!is_customized, |this| {
                            this.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .text_caption()
                                    .bg(pal.control)
                                    .text_color(pal.text_muted)
                                    .child(tr!("Default")),
                            )
                        })
                        .when(!key.is_suppressible(), |this| {
                            this.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .text_caption()
                                    .bg(pal.control)
                                    .text_color(pal.text_muted)
                                    .child(tr!("Hardware managed")),
                            )
                        })
                        // Reset to default button
                        .child(
                            Button::new("k270-reset-key-btn")
                                .ghost()
                                .disabled(!is_customized)
                                .label(tr!("Reset to default"))
                                .on_click(move |_event, _window, cx| {
                                    commit_media_key(key, None, cx);
                                }),
                        ),
                ),
        )
        // Current Action display
        .child(
            h_flex()
                .w_full()
                .items_center()
                .gap_2()
                .p_2p5()
                .rounded(pal.control_radius)
                .bg(pal.control)
                .border_1()
                .border_color(pal.border)
                .child(
                    svg()
                        .path(action_icon_path(&effective_action))
                        .size_4()
                        .text_color(pal.text_muted),
                )
                .child(
                    div()
                        .text_subheading()
                        .font_weight(FontWeight::MEDIUM)
                        .child(action_label),
                ),
        )
        // Hardware classification & behavior notes
        .child(
            v_flex()
                .gap_1()
                .p_2p5()
                .rounded(pal.control_radius)
                .bg(pal.control)
                .child(
                    div()
                        .text_caption()
                        .text_color(pal.text_muted)
                        .child(format!("{}: {}", tr!("Hardware classification"), key.hardware_event_type())),
                )
                .when(!key.is_suppressible(), |this| {
                    this.child(
                        div()
                            .text_caption()
                            .text_color(pal.text_muted)
                            .child(tr!(
                                "Note: System power and sleep events are managed directly by macOS and cannot be suppressed by user-space applications."
                            )),
                    )
                }),
        )
        // Actions header with + New Shortcut button
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .px_1()
                .child(editor_section(tr!("Actions"), pal))
                .child(
                    Button::new("k270-new-shortcut-btn")
                        .compact()
                        .ghost()
                        .label(tr!("+ New Shortcut"))
                        .on_click(move |_, window, cx| {
                            view_new.update(cx, |view, cx| {
                                view.open_shortcut_builder(window, cx);
                            });
                        }),
                ),
        )
        // Action picker search bar
        .when_some(view.search_state.as_ref(), |parent, search| {
            parent.child(
                h_flex()
                    .w_full()
                    .child(control_input(search).cleanable(true)),
            )
        })
        // Action catalog list
        .child(
            editor_scroll_list(
                "k270-action-picker-list",
                ActionLibrary::new("k270-action-row", &on_pick)
                    .current(Some(&effective_action))
                    .query(&query)
                    .on_edit(Some(&on_edit))
                    .on_open_power_user(Some(on_open_power_user))
                    .build_rows(pal, cx),
            ),
        )
}

/// Bottom section with global reset action.
fn render_reset_all_section(pal: Palette) -> impl IntoElement {
    h_flex()
        .w(px(INSPECTOR_W))
        .items_center()
        .justify_between()
        .p_4()
        .rounded(pal.card_radius)
        .border_1()
        .border_color(pal.border)
        .bg(pal.panel)
        .child(
            v_flex()
                .child(
                    div()
                        .text_subheading()
                        .font_weight(FontWeight::MEDIUM)
                        .child(tr!("Reset all media keys to defaults")),
                )
                .child(div().text_caption().text_color(pal.text_muted).child(tr!(
                    "All keys will be restored to their factory default actions."
                ))),
        )
        .child(
            control_button("k270-reset-all-btn")
                .label(tr!("Reset all media keys"))
                .on_click(|_event, _window, cx| {
                    reset_all_media_keys(cx);
                }),
        )
}
