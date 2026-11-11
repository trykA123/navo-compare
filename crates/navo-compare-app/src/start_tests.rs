//! View tests of the start screen: hints, swap, submit, rows, keys and the shell's start tab.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui_kit::test::TestWindowExt;
use gpui_kit::{AnyWindowHandle, AppContext as _, Context, Entity, TestAppContext, Window};
use navo_compare_cli::args::SideSpec;
use navo_compare_profile::recent::{RowKind, StartRow, StartRows};

use crate::shell::Shell;
use crate::start::{Side, SideHint, StartEvent, StartScreen};
use crate::start_rows::section_titles;
use crate::test_window::open;

/// A folder under the system temp dir that is removed when the guard drops.
struct TempDir(PathBuf);

impl TempDir {
    /// Makes an empty folder with a `sub` folder inside; `name` keeps tests apart.
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("sidewise-start-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("sub")).expect("make temp folder");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// The folder as text, the way a user would type it.
    fn text(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

type Screen = (AnyWindowHandle, Entity<StartScreen>);

/// One recent row with the given id and sides.
fn row(id: &str, left: &str, right: &str) -> StartRow {
    StartRow {
        id: id.into(),
        name: None,
        left: left.into(),
        right: right.into(),
        kind: RowKind::Paths,
        age: "now".into(),
    }
}

/// Opens a start screen over `rows` with `cwd`, and renders one frame.
fn screen(cx: &mut TestAppContext, rows: StartRows, cwd: &Path) -> Screen {
    let cwd = cwd.to_path_buf();
    let (window, view) = open(cx, move |window, cx| {
        StartScreen::new(rows, cwd, window, cx)
    });
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .expect("window");
    (window, view)
}

/// Runs `f` on the screen inside its window and renders the next frame.
fn act(
    cx: &mut TestAppContext,
    (window, view): &Screen,
    f: impl FnOnce(&mut StartScreen, &mut Window, &mut Context<StartScreen>),
) {
    cx.update_window(*window, |_, window, cx| {
        view.update(cx, |screen, cx| f(screen, window, cx));
        window.render_frame(cx);
    })
    .expect("window");
}

/// Collects every event the screen emits from now on.
fn record(cx: &mut TestAppContext, view: &Entity<StartScreen>) -> Rc<RefCell<Vec<StartEvent>>> {
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    cx.update(|cx| {
        cx.subscribe(view, move |_, event: &StartEvent, _| {
            sink.borrow_mut().push(event.clone());
        })
        .detach();
    });
    log
}

/// The two hints, cloned out of the screen.
fn hints(cx: &mut TestAppContext, view: &Entity<StartScreen>) -> (SideHint, SideHint) {
    view.read_with(cx, |screen, _| {
        let (left, right) = screen.hints();
        (left.clone(), right.clone())
    })
}

/// Which field has the focus.
fn focused(cx: &mut TestAppContext, s: &Screen) -> Option<Side> {
    cx.update_window(s.0, |_, window, cx| s.1.read(cx).focused_side(window, cx))
        .expect("window")
}

/// Sets both sides, lets the background checks finish and returns the hints.
fn settle(cx: &mut TestAppContext, s: &Screen, left: &str, right: &str) -> (SideHint, SideHint) {
    act(cx, s, |screen, window, cx| {
        screen.set_sides(left, right, window, cx)
    });
    cx.run_until_parked();
    hints(cx, &s.1)
}

fn path(p: &Path) -> SideHint {
    SideHint::Path(SideSpec::Path(p.to_path_buf()))
}

fn git(repo: &Path, rev: &str) -> SideHint {
    SideHint::GitRef(SideSpec::GitRev {
        repo: repo.to_path_buf(),
        rev: rev.into(),
    })
}

#[gpui_kit::test]
fn the_four_settled_hints(cx: &mut TestAppContext) {
    let dir = TempDir::new("hints");
    let s = screen(cx, StartRows::default(), dir.path());
    assert_eq!(hints(cx, &s.1), (SideHint::Empty, SideHint::Empty));
    let at_main = format!("{}@main", dir.text());
    let got = settle(cx, &s, &dir.text(), "no/such/place");
    assert_eq!(got.0, path(dir.path()));
    assert_eq!(got.1, SideHint::NotFound);
    let got = settle(cx, &s, &at_main, "   ");
    assert_eq!(got, (git(dir.path(), "main"), SideHint::Empty));
    let got = settle(cx, &s, &format!("{}@", dir.text()), "");
    assert_eq!(got.0, SideHint::NotFound);
}

#[gpui_kit::test]
fn a_hint_is_checking_until_the_background_check_ends(cx: &mut TestAppContext) {
    let dir = TempDir::new("checking");
    let s = screen(cx, StartRows::default(), dir.path());
    act(cx, &s, |screen, window, cx| {
        screen.set_sides(&dir.text(), "", window, cx);
    });
    assert_eq!(hints(cx, &s.1).0, SideHint::Checking);
    cx.run_until_parked();
    assert_eq!(hints(cx, &s.1).0, path(dir.path()));
}

#[gpui_kit::test]
fn two_quick_set_sides_end_with_the_second_hints(cx: &mut TestAppContext) {
    let dir = TempDir::new("quick");
    let s = screen(cx, StartRows::default(), dir.path());
    act(cx, &s, |screen, window, cx| {
        screen.set_sides("missing-one", "missing-two", window, cx);
        screen.set_sides(&dir.text(), &format!("{}@dev", dir.text()), window, cx);
    });
    cx.run_until_parked();
    assert_eq!(hints(cx, &s.1), (path(dir.path()), git(dir.path(), "dev")));
}

#[gpui_kit::test]
fn typing_in_a_field_checks_it(cx: &mut TestAppContext) {
    let dir = TempDir::new("typing");
    let s = screen(cx, StartRows::default(), dir.path());
    act(cx, &s, |screen, window, cx| screen.focus_left(window, cx));
    cx.simulate_input(s.0, "sub");
    assert_eq!(hints(cx, &s.1).0, path(&dir.path().join("sub")));
    cx.simulate_input(s.0, "x");
    assert_eq!(hints(cx, &s.1).0, SideHint::NotFound);
}

#[gpui_kit::test]
fn swap_exchanges_texts_and_hints(cx: &mut TestAppContext) {
    let dir = TempDir::new("swap");
    let s = screen(cx, StartRows::default(), dir.path());
    let at_main = format!("{}@main", dir.text());
    settle(cx, &s, &dir.text(), &at_main);
    act(cx, &s, |screen, window, cx| screen.swap(window, cx));
    assert_eq!(hints(cx, &s.1), (git(dir.path(), "main"), path(dir.path())));
    cx.run_until_parked();
    assert_eq!(hints(cx, &s.1), (git(dir.path(), "main"), path(dir.path())));
}

#[gpui_kit::test]
fn swap_with_a_check_in_flight_still_settles(cx: &mut TestAppContext) {
    let dir = TempDir::new("swap-flight");
    let s = screen(cx, StartRows::default(), dir.path());
    act(cx, &s, |screen, window, cx| {
        screen.set_sides(&dir.text(), "missing", window, cx);
        screen.swap(window, cx);
    });
    cx.run_until_parked();
    assert_eq!(hints(cx, &s.1), (SideHint::NotFound, path(dir.path())));
}

#[gpui_kit::test]
fn submit_emits_one_compare_when_both_sides_are_good(cx: &mut TestAppContext) {
    let dir = TempDir::new("submit");
    let s = screen(cx, StartRows::default(), dir.path());
    let log = record(cx, &s.1);
    settle(cx, &s, &dir.text(), &format!("{}@main", dir.text()));
    act(cx, &s, |screen, window, cx| screen.submit(window, cx));
    let want = StartEvent::Compare {
        left: SideSpec::Path(dir.path().to_path_buf()),
        right: SideSpec::GitRev {
            repo: dir.path().to_path_buf(),
            rev: "main".into(),
        },
    };
    assert_eq!(*log.borrow(), vec![want]);
}

#[gpui_kit::test]
fn submit_emits_nothing_for_a_missing_or_checking_side(cx: &mut TestAppContext) {
    let dir = TempDir::new("submit-bad");
    let s = screen(cx, StartRows::default(), dir.path());
    let log = record(cx, &s.1);
    settle(cx, &s, &dir.text(), "missing");
    act(cx, &s, |screen, window, cx| screen.submit(window, cx));
    assert!(log.borrow().is_empty());
    assert_eq!(focused(cx, &s), Some(Side::Right));
    act(cx, &s, |screen, window, cx| {
        screen.set_sides(&dir.text(), &dir.text(), window, cx);
        screen.submit(window, cx);
    });
    assert!(log.borrow().is_empty());
    assert_eq!(focused(cx, &s), Some(Side::Left));
    cx.run_until_parked();
    act(cx, &s, |screen, window, cx| screen.submit(window, cx));
    assert_eq!(log.borrow().len(), 1);
}

#[gpui_kit::test]
fn enter_in_a_field_submits(cx: &mut TestAppContext) {
    let dir = TempDir::new("enter");
    let s = screen(cx, StartRows::default(), dir.path());
    let log = record(cx, &s.1);
    settle(cx, &s, &dir.text(), &dir.text());
    act(cx, &s, |screen, window, cx| screen.focus_left(window, cx));
    cx.simulate_keystrokes(s.0, "enter");
    assert_eq!(log.borrow().len(), 1);
}

#[gpui_kit::test]
fn open_row_emits_for_a_known_id_only(cx: &mut TestAppContext) {
    let dir = TempDir::new("open-row");
    let rows = StartRows {
        recent: vec![row("r1", "a", "b")],
        named: vec![row("n1", "c", "d")],
    };
    let s = screen(cx, rows, dir.path());
    let log = record(cx, &s.1);
    act(cx, &s, |screen, _, cx| screen.open_row("n1", cx));
    act(cx, &s, |screen, _, cx| screen.open_row("nope", cx));
    assert_eq!(*log.borrow(), vec![StartEvent::Open { id: "n1".into() }]);
}

#[gpui_kit::test]
fn a_section_is_drawn_only_when_it_has_rows(cx: &mut TestAppContext) {
    assert!(section_titles(&StartRows::default()).is_empty());
    let named_only = StartRows {
        recent: vec![],
        named: vec![row("n1", "c", "d")],
    };
    assert_eq!(section_titles(&named_only), vec!["Named"]);
    let both = StartRows {
        recent: vec![row("r1", "a", "b")],
        named: vec![row("n1", "c", "d")],
    };
    assert_eq!(section_titles(&both), vec!["Recent", "Named"]);
    let dir = TempDir::new("sections");
    let s = screen(cx, both, dir.path());
    let drawn = cx
        .update_window(s.0, |_, window, _| {
            let rows = window.within("start-rows");
            (
                rows.try_find(("start-row", 0usize)).is_some(),
                rows.try_find(("start-row", 1usize)).is_some(),
                rows.try_find(("start-row", 2usize)).is_some(),
            )
        })
        .expect("window");
    assert_eq!(drawn, (true, true, false));
}

#[gpui_kit::test]
fn tab_moves_between_the_fields(cx: &mut TestAppContext) {
    let dir = TempDir::new("tab");
    let s = screen(cx, StartRows::default(), dir.path());
    act(cx, &s, |screen, window, cx| screen.focus_left(window, cx));
    assert_eq!(focused(cx, &s), Some(Side::Left));
    cx.simulate_keystrokes(s.0, "tab");
    assert_eq!(focused(cx, &s), Some(Side::Right));
    cx.simulate_keystrokes(s.0, "tab");
    assert_eq!(focused(cx, &s), Some(Side::Left));
    cx.simulate_keystrokes(s.0, "shift-tab");
    assert_eq!(focused(cx, &s), Some(Side::Right));
}

#[gpui_kit::test]
fn down_and_up_move_between_the_fields_and_the_rows(cx: &mut TestAppContext) {
    let dir = TempDir::new("keys");
    let rows = StartRows {
        recent: vec![row("r1", "a", "b"), row("r2", "c", "d")],
        named: vec![row("n1", "e", "f")],
    };
    let s = screen(cx, rows, dir.path());
    let log = record(cx, &s.1);
    act(cx, &s, |screen, window, cx| screen.focus_left(window, cx));
    cx.simulate_keystrokes(s.0, "down");
    assert_eq!(focused(cx, &s), None);
    cx.simulate_keystrokes(s.0, "enter");
    cx.simulate_keystrokes(s.0, "down down");
    cx.simulate_keystrokes(s.0, "enter");
    cx.simulate_keystrokes(s.0, "down");
    cx.simulate_keystrokes(s.0, "enter");
    cx.simulate_keystrokes(s.0, "up up up");
    assert_eq!(focused(cx, &s), Some(Side::Left));
    let ids: Vec<_> = log
        .borrow()
        .iter()
        .map(|event| match event {
            StartEvent::Open { id } => id.clone(),
            StartEvent::Compare { .. } => "compare".into(),
        })
        .collect();
    assert_eq!(ids, ["r1", "n1", "n1"]);
}

#[gpui_kit::test]
fn down_with_no_rows_stays_in_the_field(cx: &mut TestAppContext) {
    let dir = TempDir::new("no-rows");
    let s = screen(cx, StartRows::default(), dir.path());
    act(cx, &s, |screen, window, cx| screen.focus_left(window, cx));
    cx.simulate_keystrokes(s.0, "down");
    assert_eq!(focused(cx, &s), Some(Side::Left));
}

#[gpui_kit::test]
fn the_shells_start_tab_focuses_its_left_field(cx: &mut TestAppContext) {
    let (window, shell) = open(cx, Shell::new);
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .expect("window");
    let side = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            shell
                .read(cx)
                .active_screen()
                .and_then(|screen| screen.read(cx).focused_side(window, cx))
        })
        .expect("window")
    };
    assert_eq!(side(cx), Some(Side::Left));
    cx.simulate_keystrokes(window, "tab");
    assert_eq!(side(cx), Some(Side::Right));
    cx.simulate_keystrokes(window, "ctrl-t");
    assert_eq!(side(cx), Some(Side::Left));
    cx.simulate_keystrokes(window, "ctrl-tab");
    assert_eq!(shell.read_with(cx, |shell, _| shell.active()), 0);
    cx.simulate_keystrokes(window, "ctrl-shift-tab");
    assert_eq!(shell.read_with(cx, |shell, _| shell.active()), 1);
    assert_eq!(side(cx), Some(Side::Left));
}
