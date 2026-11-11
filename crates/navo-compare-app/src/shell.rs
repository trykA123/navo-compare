//! The window's shell: a title bar, a row of tabs, the active tab's body and a status line.
//! It owns the tab list and the browser-style tab keys (NS11, NS15, NS16).

use gpui_kit::assets::IconName;
use gpui_kit::{FocusHandle, KeyBinding};
use navo_compare_profile::brand;
use navo_ui::NavoTitleBar;
use navo_ui::prelude::*;
use navo_ui::primitives::{IconButton, Kbd, Tab, Tabs};
use navo_ui::tokens;

gpui_kit::actions!(shell, [NewTab, CloseTab, NextTab, PrevTab]);

/// Key context of the shell's root element; the tab keys are bound inside it.
const CONTEXT: &str = "Shell";

/// Width of the "+" button at the end of the tab row (NS15).
const PLUS_WIDTH: f32 = 28.;

/// What a tab shows. Only the start screen exists so far.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabKind {
    Start,
}

/// One tab: the title in the strip and the kind of body it shows.
pub struct ShellTab {
    pub title: SharedString,
    pub kind: TabKind,
}

/// The shell view: the tab list, the active index and the focus handle.
pub struct Shell {
    tabs: Vec<ShellTab>,
    active: usize,
    handle: FocusHandle,
}

/// Binds the tab keys in the `Shell` context; call once after `navo_ui::init`.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-t", NewTab, Some(CONTEXT)),
        KeyBinding::new("ctrl-w", CloseTab, Some(CONTEXT)),
        KeyBinding::new("ctrl-tab", NextTab, Some(CONTEXT)),
        KeyBinding::new("ctrl-shift-tab", PrevTab, Some(CONTEXT)),
    ]);
}

/// A fresh start tab titled "New compare".
fn start_tab() -> ShellTab {
    ShellTab {
        title: "New compare".into(),
        kind: TabKind::Start,
    }
}

impl Shell {
    /// Makes a shell with one start tab and focuses its handle.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let handle = cx.focus_handle();
        handle.focus(window, cx);
        Self {
            tabs: vec![start_tab()],
            active: 0,
            handle,
        }
    }

    /// Number of open tabs.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "read by the view tests; the start screen reads it later"
        )
    )]
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Index of the active tab.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "read by the view tests; the start screen reads it later"
        )
    )]
    pub fn active(&self) -> usize {
        self.active
    }

    /// The handle the shell's root element tracks.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "read by the view tests; the start screen reads it later"
        )
    )]
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.handle
    }

    /// Appends a start tab, makes it active and focuses the shell.
    pub fn new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tabs.push(start_tab());
        self.active = self.tabs.len() - 1;
        self.activated(window, cx);
    }

    /// Closes the tab at `index`; the tab to its right (else the new last) becomes active.
    /// A tab left of the active one keeps the active tab. Closing the last tab closes the window.
    /// An index out of range does nothing.
    pub fn close_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        if self.tabs.len() == 1 {
            window.remove_window();
            return;
        }
        self.tabs.remove(index);
        if index < self.active {
            self.active -= 1;
        }
        self.active = self.active.min(self.tabs.len() - 1);
        self.activated(window, cx);
    }

    /// Makes the tab at `index` active; an index out of range does nothing.
    pub fn select_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.tabs.len() {
            self.active = index;
            self.activated(window, cx);
        }
    }

    /// Moves to the next tab, wrapping from the last to the first.
    pub fn next_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_tab((self.active + 1) % self.tabs.len(), window, cx);
    }

    /// Moves to the previous tab, wrapping from the first to the last.
    pub fn prev_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.tabs.len();
        self.select_tab((self.active + count - 1) % count, window, cx);
    }

    /// Focuses the shell (NS16) and redraws after the active tab changed.
    fn activated(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.handle.focus(window, cx);
        cx.notify();
    }

    /// The tab strip with the "+" button at its end.
    fn tab_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = tokens::current(cx);
        let plus = IconButton::new("new-tab", IconName::Plus, "New tab")
            .on_click(cx.listener(|this, _, window, cx| this.new_tab(window, cx)));
        let tabs = Tabs::new(
            "tabs",
            self.tabs.iter().map(|tab| Tab::new(tab.title.clone())),
        )
        .selected(self.active)
        .trailing(plus, px(PLUS_WIDTH))
        .on_select(cx.listener(|this, index: &usize, window, cx| {
            this.select_tab(*index, window, cx);
        }))
        .on_close(cx.listener(|this, index: &usize, window, cx| {
            this.close_tab(*index, window, cx);
        }));
        div()
            .w_full()
            .flex_none()
            .px(t.spacing.sm)
            .py(t.spacing.xs)
            .border_b_1()
            .border_color(t.colors.line)
            .child(tabs)
    }

    /// The active tab's body: empty for a start tab until the start screen exists.
    fn body(&self, cx: &App) -> impl IntoElement {
        let background = tokens::current(cx).colors.background;
        let kind = self.tabs.get(self.active).map(|tab| tab.kind);
        match kind {
            Some(TabKind::Start) | None => div().flex_1().w_full().min_h_0().bg(background),
        }
    }

    /// The status line: the key hints; a hint whose key does not parse is left out.
    fn status_line(&self, cx: &App) -> impl IntoElement {
        let t = tokens::current(cx);
        let hints = [
            ("hint-new", "ctrl-t", "New tab"),
            ("hint-close", "ctrl-w", "Close tab"),
        ];
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(t.spacing.lg)
            .w_full()
            .h(t.spacing.status_height)
            .px(t.spacing.lg)
            .border_t_1()
            .border_color(t.colors.line)
            .text_size(t.typography.small)
            .text_color(t.colors.muted)
            .children(hints.into_iter().filter_map(|(id, keys, label)| {
                Kbd::new(id, keys).map(|kbd| {
                    div()
                        .flex()
                        .items_center()
                        .gap(t.spacing.sm)
                        .child(kbd)
                        .child(label)
                })
            }))
    }
}

impl Render for Shell {
    /// Draws the title bar, the tab row, the body and the status line inside the `Shell` context.
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = tokens::current(cx);
        let (background, text) = (t.colors.background, t.colors.text);
        let (font, size) = (t.typography.ui_font.clone(), t.typography.body);
        div()
            .key_context(CONTEXT)
            .track_focus(&self.handle)
            .on_action(cx.listener(|this, _: &NewTab, window, cx| this.new_tab(window, cx)))
            .on_action(cx.listener(|this, _: &CloseTab, window, cx| {
                this.close_tab(this.active, window, cx);
            }))
            .on_action(cx.listener(|this, _: &NextTab, window, cx| this.next_tab(window, cx)))
            .on_action(cx.listener(|this, _: &PrevTab, window, cx| this.prev_tab(window, cx)))
            .flex()
            .flex_col()
            .size_full()
            .bg(background)
            .text_color(text)
            .font_family(font)
            .text_size(size)
            .child(NavoTitleBar::new(brand::NAME))
            .child(self.tab_row(cx))
            .child(self.body(cx))
            .child(self.status_line(cx))
    }
}
