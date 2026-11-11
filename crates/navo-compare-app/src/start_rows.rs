//! The Recent and Named sections of the start screen: the row order, the sections and their drawing.
//! The rows are plain data from `navo_compare_profile::recent` (NS13); this file reads no file.

use gpui_kit::{FocusHandle, TestSupportExt};
use navo_compare_profile::recent::{RowKind, StartRow, StartRows};
use navo_ui::prelude::*;
use navo_ui::tokens;

use crate::start::StartScreen;

/// Key context of the row list; the Up, Down and Enter keys are bound inside it.
pub(crate) const LIST_CONTEXT: &str = "StartList";

/// Number of rows over both sections (recent first, then named).
pub(crate) fn row_count(rows: &StartRows) -> usize {
    rows.recent.len() + rows.named.len()
}

/// The row at `index` over both sections, or `None` past the end.
pub(crate) fn row_at(rows: &StartRows, index: usize) -> Option<&StartRow> {
    rows.recent.iter().chain(rows.named.iter()).nth(index)
}

/// The sections that have rows: title, rows and flat position of the first row.
fn sections(rows: &StartRows) -> Vec<(&'static str, &[StartRow], usize)> {
    let all = [
        ("Recent", rows.recent.as_slice(), 0),
        ("Named", rows.named.as_slice(), rows.recent.len()),
    ];
    all.into_iter()
        .filter(|(_, rows, _)| !rows.is_empty())
        .collect()
}

/// The titles of the sections that have rows, in drawing order.
#[cfg(test)]
pub(crate) fn section_titles(rows: &StartRows) -> Vec<&'static str> {
    sections(rows)
        .into_iter()
        .map(|(title, _, _)| title)
        .collect()
}

/// The label of a row's kind.
fn kind_label(kind: &RowKind) -> &'static str {
    match kind {
        RowKind::GitRefs => "Git refs",
        RowKind::Paths => "Paths",
        RowKind::Mixed => "Mixed",
    }
}

/// A section heading.
fn heading(title: &'static str, cx: &App) -> Div {
    let t = tokens::current(cx);
    div()
        .px(t.spacing.md)
        .pt(t.spacing.xxl)
        .pb(t.spacing.xs)
        .text_size(t.typography.tiny)
        .text_color(t.colors.faint)
        .child(title)
}

/// One row at flat position `index`; a click opens it, the selected row is highlighted.
fn row_view(
    index: usize,
    row: &StartRow,
    selected: bool,
    cx: &Context<StartScreen>,
) -> impl IntoElement {
    let t = tokens::current(cx);
    let side = |text: &str| {
        div()
            .font_family(t.typography.mono_font.clone())
            .child(text.to_owned())
    };
    let (background, text, muted) = if selected {
        (
            t.colors.selection,
            t.colors.on_selection,
            t.colors.on_selection,
        )
    } else {
        (gpui_kit::transparent_black(), t.colors.text, t.colors.muted)
    };
    div()
        .id(("start-row", index))
        .test_support()
        .flex()
        .items_center()
        .gap(t.spacing.lg)
        .w_full()
        .h(t.spacing.row_height)
        .px(t.spacing.md)
        .rounded(t.radii.sm)
        .bg(background)
        .text_color(text)
        .text_size(t.typography.small)
        .cursor_pointer()
        .hover(|style| style.bg(t.colors.hover))
        .on_click(cx.listener(move |this, _, _, cx| this.click_row(index, cx)))
        .children(row.name.as_ref().map(|name| div().child(name.clone())))
        .child(side(&row.left))
        .child(div().text_color(muted).child("\u{2194}"))
        .child(side(&row.right))
        .child(div().text_color(muted).child(kind_label(&row.kind)))
        .child(
            div()
                .flex_1()
                .flex()
                .justify_end()
                .text_color(muted)
                .child(row.age.clone()),
        )
}

/// The scrollable row list: a heading and the rows of each section that has rows.
/// `selected` is a flat position; `focus` makes the list a keyboard target.
pub(crate) fn rows_view(
    rows: &StartRows,
    selected: Option<usize>,
    focus: &FocusHandle,
    cx: &Context<StartScreen>,
) -> impl IntoElement {
    let mut list = div()
        .id("start-rows")
        .test_support()
        .key_context(LIST_CONTEXT)
        .track_focus(focus)
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll();
    for (title, section, first) in sections(rows) {
        list = list.child(heading(title, cx));
        for (offset, row) in section.iter().enumerate() {
            let index = first + offset;
            list = list.child(row_view(index, row, selected == Some(index), cx));
        }
    }
    list
}
