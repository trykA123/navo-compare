//! Drawing of the start screen: the two fields with their chips, the buttons, the hint line.
//! The rows below them come from `start_rows`.

use gpui_kit::assets::IconName;
use navo_ui::prelude::*;
use navo_ui::primitives::{Button, ButtonVariant, IconButton, Kbd, TextField};
use navo_ui::tokens;

use crate::start::{
    CONTEXT, EnterList, OpenSelected, OtherField, SelectNext, SelectPrev, Side, SideHint,
    StartScreen,
};
use crate::start_rows::rows_view;

/// The chip beside a field: "Path", "Git ref" or "Not found" (in the warning colour).
fn chip(hint: &SideHint, cx: &App) -> Option<Div> {
    let t = tokens::current(cx);
    let (label, color) = match hint {
        SideHint::Path(_) => ("Path", t.colors.muted),
        SideHint::GitRef(_) => ("Git ref", t.colors.muted),
        SideHint::NotFound => ("Not found", t.colors.warn),
        SideHint::Empty | SideHint::Checking => return None,
    };
    Some(
        div()
            .flex_none()
            .px(t.spacing.md)
            .py(t.spacing.xxs)
            .rounded(t.radii.pill)
            .border_1()
            .border_color(color)
            .text_size(t.typography.tiny)
            .text_color(color)
            .child(label),
    )
}

impl StartScreen {
    /// One side: its label, the text field and the chip.
    fn field_view(&self, side: Side, cx: &App) -> impl IntoElement {
        let t = tokens::current(cx);
        let (id, label) = match side {
            Side::Left => ("start-left", "Left"),
            Side::Right => ("start-right", "Right"),
        };
        let i = side.index();
        div()
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(t.spacing.md)
            .child(
                div()
                    .flex_none()
                    .text_size(t.typography.small)
                    .text_color(t.colors.faint)
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(TextField::new(id, &self.fields[i], label)),
            )
            .children(chip(&self.hints[i], cx))
    }

    /// The row of both fields with the swap and Compare buttons.
    fn fields_row(&self, cx: &Context<Self>) -> impl IntoElement {
        let t = tokens::current(cx);
        let swap = IconButton::new("start-swap", IconName::Replace, "Swap sides")
            .on_click(cx.listener(|this, _, window, cx| this.swap(window, cx)));
        let compare = Button::new("start-compare", "Compare")
            .variant(ButtonVariant::Primary)
            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx)));
        div()
            .flex()
            .items_center()
            .gap(t.spacing.md)
            .w_full()
            .child(self.field_view(Side::Left, cx))
            .child(swap)
            .child(self.field_view(Side::Right, cx))
            .child(compare)
    }

    /// The muted line under the fields, with the `Tab` and `Enter` hints.
    fn hint_line(&self, cx: &App) -> impl IntoElement {
        let t = tokens::current(cx);
        let key = |id: &'static str, keys: &str, label: &'static str| {
            Kbd::new(id, keys).map(|kbd| {
                div()
                    .flex()
                    .items_center()
                    .gap(t.spacing.sm)
                    .child(kbd)
                    .child(label)
            })
        };
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(t.spacing.lg)
            .px(t.spacing.xs)
            .py(t.spacing.xs)
            .text_size(t.typography.small)
            .text_color(t.colors.muted)
            .child("Type or paste a path, or write repo@branch.")
            .children(key("hint-tab", "tab", "next side"))
            .children(key("hint-enter", "enter", "compare"))
    }
}

impl Render for StartScreen {
    /// Draws the fields, the hint line and the rows inside the `Start` context.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = tokens::current(cx);
        let selected = self.selected.filter(|_| self.list_focus.is_focused(window));
        let rows = rows_view(&self.rows, selected, &self.list_focus, cx);
        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(|this, _: &OtherField, window, cx| this.other_field(window, cx)))
            .on_action(cx.listener(|this, _: &EnterList, window, cx| this.enter_list(window, cx)))
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| this.select_next(cx)))
            .on_action(cx.listener(|this, _: &SelectPrev, window, cx| this.select_prev(window, cx)))
            .on_action(cx.listener(|this, _: &OpenSelected, _, cx| this.open_selected(cx)))
            .flex()
            .flex_col()
            .size_full()
            .gap(t.spacing.sm)
            .px(t.spacing.xxl * 4.)
            .pt(t.spacing.xxl * 4.)
            .child(self.fields_row(cx))
            .child(self.hint_line(cx))
            .child(rows)
    }
}
