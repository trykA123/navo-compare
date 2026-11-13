//! The sessions sidebar (NC66): 280 px expanded, a 56 px icon rail collapsed, toggled by
//! Ctrl+\. The width lerps over the side token through the U12b reduced-motion gate.

use gpui_kit::assets::IconName;
use gpui_kit::{AnimationExt, FocusHandle, KeyBinding, ReadGlobal, img};
use navo_compare_profile::recent::{StartRow, StartRows};
use navo_ui::prelude::*;
use navo_ui::primitives::icon;

use crate::icons::{MaterialIcon, cache::IconCache, icon_id};
use crate::motion;
use crate::theme::SidewiseTheme;
use crate::theme_data::palettes::{DIMS, ModeTokens};

gpui_kit::actions!(sidebar, [ToggleRail]);

/// Key context of the sidebar root; Ctrl+\ is bound inside it.
const CONTEXT: &str = "Sidebar";
/// The mark side both modes draw: the Material pack's 1x bucket.
const MARK: u32 = 16;
/// The mark side as a length.
const MARK_PX: f32 = MARK as f32;

/// Binds Ctrl+\ in the `Sidebar` context; the shell calls it from its own `bind_keys`.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("ctrl-\\", ToggleRail, Some(CONTEXT))]);
}

/// A row's mark: a bundled chrome glyph, or a Material file mark from U11c.
#[derive(Clone, Copy)]
enum Mark {
    Chrome(IconName),
    File(MaterialIcon),
}

/// One entry: its mark, its label and its optional key hint.
struct Entry {
    mark: Mark,
    label: SharedString,
    hint: Option<&'static str>,
}

/// One headed list of entries; an empty heading is the unheaded primary block.
struct Section {
    heading: &'static str,
    entries: Vec<Entry>,
}

/// The sidebar: the rail flag, the width it animates from, and the lists it shows.
pub struct Sidebar {
    rail: bool,
    from: Pixels,
    rows: StartRows,
    tabs: Vec<SharedString>,
    handle: FocusHandle,
}

impl Sidebar {
    /// An expanded sidebar over the stored rows and the open tab titles.
    pub fn new(rows: StartRows, tabs: Vec<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            rail: false,
            from: DIMS.sidebar_w,
            rows,
            tabs,
            handle: cx.focus_handle(),
        }
    }

    /// Whether the icon rail is showing.
    pub fn rail(&self) -> bool {
        self.rail
    }

    /// The width the sidebar settles at: the rail or the full sidebar.
    pub fn width(&self) -> Pixels {
        if self.rail {
            DIMS.rail_w
        } else {
            DIMS.sidebar_w
        }
    }

    /// Ctrl+\: swap rail and sidebar, animating from the width being left.
    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.from = self.width();
        self.rail = !self.rail;
        cx.notify();
    }

    /// The primary actions, then the three lists, in prototype order.
    fn sections(&self) -> Vec<Section> {
        let recent = self
            .rows
            .recent
            .iter()
            .map(|r| entry_of(r, false))
            .collect();
        let named = self.rows.named.iter().map(|r| entry_of(r, true)).collect();
        let tabs = self
            .tabs
            .iter()
            .map(|title| Entry {
                mark: Mark::File(MaterialIcon::Document),
                label: title.clone(),
                hint: None,
            })
            .collect();
        vec![
            Section {
                heading: "",
                entries: primary(),
            },
            Section {
                heading: "Open tabs",
                entries: tabs,
            },
            Section {
                heading: "Recent",
                entries: recent,
            },
            Section {
                heading: "Named",
                entries: named,
            },
        ]
    }

    /// The label and hint of one entry, or nothing in the rail: the rail is icon-only.
    fn label_view(&self, entry: &Entry, m: &ModeTokens) -> Option<Div> {
        if self.rail {
            return None;
        }
        let mut text = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .text_color(m.text)
            .child(entry.label.clone());
        if let Some(hint) = entry.hint {
            text = text.child(div().text_color(m.faint).child(hint));
        }
        Some(text)
    }

    /// One row: its mark, then its label only when the sidebar is expanded.
    fn row_view(&self, entry: &Entry, m: &ModeTokens, cx: &mut Context<Self>) -> Div {
        let mut row = div()
            .flex()
            .items_center()
            .gap_2()
            .min_h(DIMS.row_h)
            .child(mark_view(entry.mark, m.muted, cx));
        if let Some(label) = self.label_view(entry, m) {
            row = row.child(label);
        }
        row
    }

    /// One headed list; the rail drops the heading along with the labels.
    fn group_view(&self, section: &Section, m: &ModeTokens, cx: &mut Context<Self>) -> Div {
        let mut group = div().flex().flex_col().gap_1();
        if !section.heading.is_empty() && !self.rail {
            group = group.child(div().text_color(m.muted).child(section.heading));
        }
        group.children(
            section
                .entries
                .iter()
                .map(|entry| self.row_view(entry, m, cx)),
        )
    }

    /// The footer actions, pushed to the bottom and icon-only in the rail.
    fn footer_view(&self, m: &ModeTokens, cx: &mut Context<Self>) -> Div {
        let accounts = Entry {
            mark: Mark::File(MaterialIcon::Lock),
            label: "Accounts".into(),
            hint: None,
        };
        let appearance = Entry {
            mark: Mark::Chrome(IconName::Palette),
            label: "Appearance".into(),
            hint: None,
        };
        div()
            .mt_auto()
            .flex()
            .flex_col()
            .gap_1()
            .child(self.row_view(&accounts, m, cx))
            .child(self.row_view(&appearance, m, cx))
    }
}

/// The two primary actions of the prototype, with their key hints.
#[rustfmt::skip]
fn primary() -> Vec<Entry> {
    vec![
        Entry { mark: Mark::Chrome(IconName::Plus), label: "New comparison".into(), hint: Some("Ctrl+T") },
        Entry { mark: Mark::File(MaterialIcon::Git), label: "Git compare".into(), hint: Some("6") },
    ]
}

/// One stored session as an entry: a named session keeps its name, a recent one its two sides.
fn entry_of(row: &StartRow, named: bool) -> Entry {
    let mark = if named {
        Mark::Chrome(IconName::Star)
    } else {
        Mark::File(icon_id(&row.left, true, false))
    };
    let label = match &row.name {
        Some(name) if named => name.clone().into(),
        _ => format!("{} ↔ {}", row.left, row.right).into(),
    };
    Entry {
        mark,
        label,
        hint: None,
    }
}

/// Draws one mark at the 1x bucket; a missing raster leaves the row without a glyph.
fn mark_view(mark: Mark, color: Hsla, cx: &mut App) -> AnyElement {
    match mark {
        Mark::Chrome(name) => icon(name, px(MARK_PX), color).into_any_element(),
        Mark::File(material) => match IconCache::image(material, MARK, cx) {
            Some(image) => img(image).size(px(MARK_PX)).flex_none().into_any_element(),
            None => div().size(px(MARK_PX)).flex_none().into_any_element(),
        },
    }
}

impl Render for Sidebar {
    /// The sidebar on `--bg2` with a `--sep` hairline, its text kept 16 px off the sides and
    /// 12 px off the top. The width lerps from the previous one, or snaps under reduced motion.
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let side = *SidewiseTheme::global(cx);
        let m = if side.mode.is_dark() {
            side.spec.dark
        } else {
            side.spec.light
        };
        let (from, to) = (self.from, self.width());
        let sections = self.sections();
        let footer = self.footer_view(&m, cx);
        let body = div()
            .key_context(CONTEXT)
            .track_focus(&self.handle)
            .on_action(cx.listener(|this, _: &ToggleRail, _, cx| this.toggle(cx)))
            .flex()
            .flex_col()
            .flex_none()
            .h_full()
            .w(to)
            .overflow_hidden()
            .bg(m.surface2)
            .border_r_1()
            .border_color(m.sep)
            .px(DIMS.inset_x)
            .pt(DIMS.inset_y)
            .gap_3()
            .children(
                sections
                    .iter()
                    .map(|section| self.group_view(section, &m, cx)),
            )
            .child(footer);
        match motion::anim("sidebar-width", side.motion.side_ms, cx) {
            Some(enter) => body
                .with_animation("sidebar-width", enter, move |el, delta| {
                    el.w(from + (to - from) * delta)
                })
                .into_any_element(),
            None => body.into_any_element(),
        }
    }
}

#[cfg(test)]
mod sidebar_tests {
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{AnyWindowHandle, TestAppContext};
    use navo_compare_profile::appearance::PaletteId;
    use navo_compare_profile::recent::RowKind;
    use navo_ui::tokens::Mode;

    use super::*;
    use crate::test_window::open;
    use crate::theme::apply_sidewise;

    /// One recent and one named session, so every list has a row.
    fn rows() -> StartRows {
        let row = StartRow {
            id: "1".into(),
            name: None,
            left: "left".into(),
            right: "right".into(),
            kind: RowKind::Paths,
            age: "now".into(),
        };
        StartRows {
            recent: vec![row.clone()],
            named: vec![StartRow {
                name: Some("Saved".into()),
                ..row
            }],
        }
    }

    /// The active mode's tokens, as the render reads them.
    fn mode_tokens(cx: &App) -> ModeTokens {
        let side = SidewiseTheme::global(cx);
        if side.mode.is_dark() {
            side.spec.dark
        } else {
            side.spec.light
        }
    }

    /// A focused sidebar in a themed window; the tokens exist before the first frame.
    fn sidebar(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Sidebar>) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            apply_sidewise(PaletteId::Violet, Mode::Light, cx);
            bind_keys(cx);
        });
        let (window, view) = open(cx, |_, cx| Sidebar::new(rows(), vec!["tab".into()], cx));
        cx.update_window(window, |_, window, cx| {
            let handle = view.read(cx).handle.clone();
            handle.focus(window, cx);
            window.render_frame(cx);
        })
        .expect("window");
        (window, view)
    }

    /// Ctrl+\ collapses 280 -> 56 and drops the label bounds; a second press restores them.
    #[gpui_kit::test]
    fn ctrl_backslash_toggles_the_rail_and_the_labels(cx: &mut TestAppContext) {
        let (window, view) = sidebar(cx);
        cx.update(|cx| {
            let expanded = view.read(cx);
            assert_eq!(expanded.width(), DIMS.sidebar_w, "starts expanded at 280px");
            assert!(!expanded.rail(), "starts as the full sidebar");
            let entry = &expanded.sections()[1].entries[0];
            assert!(
                expanded.label_view(entry, &mode_tokens(cx)).is_some(),
                "the expanded sidebar shows labels"
            );
        });
        cx.simulate_keystrokes(window, "ctrl-\\");
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            let rail = view.read(cx);
            assert!(rail.rail(), "Ctrl+\\ must collapse to the rail");
            assert_eq!(rail.width(), DIMS.rail_w, "the rail is 56px");
            let entry = &rail.sections()[1].entries[0];
            assert!(
                rail.label_view(entry, &mode_tokens(cx)).is_none(),
                "the rail omits label bounds"
            );
        })
        .expect("window");
        cx.simulate_keystrokes(window, "ctrl-\\");
        cx.update(|cx| {
            assert!(!view.read(cx).rail(), "a second press expands again");
            assert_eq!(view.read(cx).width(), DIMS.sidebar_w, "back to 280px");
        });
    }

    /// Under reduced motion the width snaps to the rail and no animation is scheduled.
    #[gpui_kit::test]
    fn reduced_motion_snaps_the_width(cx: &mut TestAppContext) {
        let (window, view) = sidebar(cx);
        cx.update(|cx| cx.set_reduce_motion(true));
        cx.simulate_keystrokes(window, "ctrl-\\");
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(
                view.read(cx).width(),
                DIMS.rail_w,
                "the width snaps, not lerps"
            );
            assert!(
                motion::anim("sidebar", SidewiseTheme::global(cx).motion.side_ms, cx).is_none(),
                "reduced motion must stop the collapse animation"
            );
        })
        .expect("window");
    }

    /// The sections and the two primary actions match `sidewise-chosen`.
    #[gpui_kit::test]
    fn the_sections_match_the_prototype(cx: &mut TestAppContext) {
        let (_window, view) = sidebar(cx);
        cx.update(|cx| {
            let sections = view.read(cx).sections();
            let headings: Vec<_> = sections.iter().map(|s| s.heading).collect();
            assert_eq!(
                headings,
                ["", "Open tabs", "Recent", "Named"],
                "section order"
            );
            assert_eq!(sections[0].entries.len(), 2, "two primary actions");
            assert_eq!(
                sections[0].entries[0].label,
                SharedString::from("New comparison")
            );
            assert_eq!(
                sections[0].entries[1].label,
                SharedString::from("Git compare")
            );
            assert_eq!(sections[2].entries.len(), 1, "the recent list");
            assert_eq!(sections[3].entries.len(), 1, "the named list");
        });
    }

    /// Stop-if probe: the rail's icon fits between the side insets, and text keeps the owner's
    /// minimum 16 px side and 12 px top inset.
    #[test]
    fn the_rail_icon_fits_and_the_text_insets_hold() {
        let content = f32::from(DIMS.rail_w) - 2.0 * f32::from(DIMS.inset_x);
        assert!(
            content >= MARK_PX,
            "icon-only rows would overlap at 56px: {content}px for a {MARK}px mark"
        );
        assert!(
            f32::from(DIMS.inset_x) >= 16.0,
            "text needs 16px off the sides"
        );
        assert!(
            f32::from(DIMS.inset_y) >= 12.0,
            "text needs 12px off the top"
        );
    }
}
