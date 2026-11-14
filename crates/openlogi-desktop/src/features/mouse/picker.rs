//! Shared action-catalog rows and menu surfaces used by binding editors.

use gpui::{
    InteractiveElement as _, IntoElement, ParentElement, StatefulInteractiveElement as _, Styled,
    div, px,
};
use gpui_component::v_flex;

use crate::ui::theme::{Palette, Typography as _};

pub(crate) use crate::ui::action::{GESTURE_BUTTON_ICON, action_icon_path};
pub(crate) use crate::ui::action_library::{EditFn, PickFn, editor_section};

/// Height cap shared by compact inspector and editor lists.
pub(crate) const EDITOR_LIST_MAX_H: f32 = 360.;

/// Shared card surface for compact binding panels and menus.
pub(crate) fn compact_panel(pal: Palette) -> gpui::Div {
    v_flex()
        .bg(pal.panel)
        .border_1()
        .border_color(pal.border)
        .rounded(pal.card_radius)
        .shadow_md()
        .p_1p5()
}

/// Compact editor title.
pub(crate) fn title(text: impl Into<gpui::SharedString>, pal: Palette) -> impl IntoElement {
    div()
        .px_2()
        .pb_1()
        .text_subheading()
        .text_color(pal.text_muted)
        .child(text.into())
}

/// Hairline separating compact editor regions.
pub(crate) fn divider(pal: Palette) -> impl IntoElement {
    div().mb_1().h(px(1.)).w_full().bg(pal.border)
}

/// Height-capped scroll region for compact editor rows.
pub(crate) fn editor_scroll_list(
    id: &'static str,
    rows: impl IntoIterator<Item = impl IntoElement>,
) -> impl IntoElement {
    div()
        .id(id)
        .max_h(px(EDITOR_LIST_MAX_H))
        .overflow_y_scroll()
        .children(rows)
}
