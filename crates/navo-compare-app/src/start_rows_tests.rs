//! Tests of the start screen's row click: it selects the row, then opens it.

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::{AppContext as _, TestAppContext};
use navo_compare_profile::recent::{RowKind, StartRow, StartRows};

use crate::start::{StartEvent, StartScreen};
use crate::test_window::open;

/// One recent row with the given id.
fn row(id: &str) -> StartRow {
    StartRow {
        id: id.into(),
        name: None,
        left: "a".into(),
        right: "b".into(),
        kind: RowKind::Paths,
        age: "now".into(),
    }
}

#[gpui_kit::test]
fn clicking_a_row_selects_it_and_opens_it(cx: &mut TestAppContext) {
    let rows = StartRows {
        recent: vec![row("r1"), row("r2")],
        named: vec![row("n1")],
    };
    let (window, view) = open(cx, move |window, cx| {
        StartScreen::new(rows, std::env::temp_dir(), window, cx)
    });
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    cx.update(|cx| {
        cx.subscribe(&view, move |_, event: &StartEvent, _| {
            sink.borrow_mut().push(event.clone());
        })
        .detach();
    });
    cx.update_window(window, |_, _, cx| {
        view.update(cx, |screen, cx| screen.click_row(2, cx));
    })
    .expect("window");
    assert_eq!(view.read_with(cx, |screen, _| screen.selected), Some(2));
    assert!(matches!(
        log.borrow().as_slice(),
        [StartEvent::Open { id }] if id == "n1"
    ));
}
