//! The window's shell: a title bar, a row of tabs, the active tab's body and a status line.
//! It owns the tab list and the browser-style tab keys (NS11, NS15, NS16).

use gpui_kit::assets::IconName;
use gpui_kit::{FocusHandle, KeyBinding, Subscription};
use navo_compare_profile::brand;
use navo_compare_profile::recent::StartRows;
use navo_ui::NavoTitleBar;
use navo_ui::prelude::*;
use navo_ui::primitives::{IconButton, Kbd, Tab, Tabs};
use navo_ui::tokens;

use crate::start::{self, StartScreen};
use crate::theme::mode_of;

gpui_kit::actions!(shell, [NewTab, CloseTab, NextTab, PrevTab]);

/// Key context of the shell's root element; the tab keys are bound inside it.
const CONTEXT: &str = "Shell";

/// Width of the "+" button at the end of the tab row (NS15).
const PLUS_WIDTH: f32 = 28.;

/// What a tab shows. Only the start screen exists so far; each start tab owns its screen.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TabKind {
    Start(Entity<StartScreen>),
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
    _appearance: Subscription,
}

/// Binds the tab keys in the `Shell` context and the start screen's keys;
/// call once after `navo_ui::init`.
pub fn bind_keys(cx: &mut App) {
    start::bind_keys(cx);
    cx.bind_keys([
        KeyBinding::new("ctrl-t", NewTab, Some(CONTEXT)),
        KeyBinding::new("ctrl-w", CloseTab, Some(CONTEXT)),
        KeyBinding::new("ctrl-tab", NextTab, Some(CONTEXT)),
        KeyBinding::new("ctrl-shift-tab", PrevTab, Some(CONTEXT)),
    ]);
}

/// A fresh start tab titled "New compare", with its own start screen over no stored rows yet.
fn start_tab(window: &mut Window, cx: &mut Context<Shell>) -> ShellTab {
    let cwd = std::env::current_dir().unwrap_or_default();
    let screen = cx.new(|cx| StartScreen::new(StartRows::default(), cwd, window, cx));
    ShellTab {
        title: "New compare".into(),
        kind: TabKind::Start(screen),
    }
}

impl Shell {
    /// Makes a shell with one start tab and focuses that tab's left field.
    /// It applies the token mode of the window's appearance and follows later changes (NS19).
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        tokens::apply(mode_of(window.appearance()), cx);
        let appearance = cx.observe_window_appearance(window, |_, window, cx| {
            tokens::apply(mode_of(window.appearance()), cx);
            cx.notify();
        });
        let shell = Self {
            tabs: vec![start_tab(window, cx)],
            active: 0,
            handle: cx.focus_handle(),
            _appearance: appearance,
        };
        shell.activated(window, cx);
        shell
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

    /// The start screen entity ids of all tabs, in order (lets tests tell tabs apart).
    #[cfg(test)]
    pub(crate) fn tab_ids(&self) -> Vec<gpui_kit::EntityId> {
        self.tabs
            .iter()
            .map(|tab| match &tab.kind {
                TabKind::Start(screen) => screen.entity_id(),
            })
            .collect()
    }

    /// The start screen of the active tab.
    pub fn active_screen(&self) -> Option<&Entity<StartScreen>> {
        match self.tabs.get(self.active).map(|tab| &tab.kind) {
            Some(TabKind::Start(screen)) => Some(screen),
            None => None,
        }
    }

    /// Appends a start tab, makes it active and focuses its left field.
    pub fn new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tab = start_tab(window, cx);
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        self.activated(window, cx);
    }

    /// Closes the tab at `index`; the tab to its right (else the new last) becomes active.
    /// A tab left of the active one keeps the active tab. Closing the last tab closes the window.
    /// Only closing the active tab moves the focus; closing another tab leaves it. An index out
    /// of range does nothing.
    pub fn close_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        if self.tabs.len() == 1 {
            window.remove_window();
            return;
        }
        let was_active = index == self.active;
        self.tabs.remove(index);
        if index < self.active {
            self.active -= 1;
        }
        self.active = self.active.min(self.tabs.len() - 1);
        if was_active {
            self.activated(window, cx);
        } else {
            cx.notify();
        }
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

    /// Focuses the active start tab's left field (NS16), else the shell; redraws.
    fn activated(&self, window: &mut Window, cx: &mut Context<Self>) {
        match self.active_screen() {
            Some(screen) => screen.update(cx, |screen, cx| screen.focus_left(window, cx)),
            None => self.handle.focus(window, cx),
        }
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

    /// The active tab's body: the start screen of a start tab.
    fn body(&self, cx: &App) -> impl IntoElement {
        let background = tokens::current(cx).colors.background;
        div()
            .flex_1()
            .w_full()
            .min_h_0()
            .bg(background)
            .children(self.active_screen().cloned())
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
