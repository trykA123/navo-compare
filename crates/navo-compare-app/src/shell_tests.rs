//! View tests of the shell: tab rules (NS11), keys, focus and closing the last tab.

use gpui_kit::test::TestWindowExt;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

use crate::shell::Shell;
use crate::start::Side;
use crate::test_window::open;

/// Opens a shell in a test window and renders one frame.
fn shell(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Shell>) {
    let (window, view) = open(cx, Shell::new);
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .expect("window");
    (window, view)
}

/// Runs `f` on the shell inside its window and renders the next frame.
fn act(
    cx: &mut TestAppContext,
    (window, view): &(AnyWindowHandle, Entity<Shell>),
    f: impl FnOnce(&mut Shell, &mut gpui_kit::Window, &mut gpui_kit::Context<Shell>),
) {
    cx.update_window(*window, |_, window, cx| {
        view.update(cx, |shell, cx| f(shell, window, cx));
        window.render_frame(cx);
    })
    .expect("window");
}

/// Returns (tab count, active index) of the shell.
fn state(cx: &mut TestAppContext, view: &Entity<Shell>) -> (usize, usize) {
    view.read_with(cx, |shell, _| (shell.tab_count(), shell.active()))
}

#[gpui_kit::test]
fn one_start_tab_at_launch(cx: &mut TestAppContext) {
    let (_, view) = shell(cx);
    assert_eq!(state(cx, &view), (1, 0));
}

#[gpui_kit::test]
fn new_tabs_go_to_the_end_and_become_active(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| shell.new_tab(window, cx));
    act(cx, &s, |shell, window, cx| shell.new_tab(window, cx));
    assert_eq!(state(cx, &s.1), (3, 2));
}

#[gpui_kit::test]
fn closing_the_middle_tab_activates_its_right_neighbour(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| {
        shell.new_tab(window, cx);
        shell.new_tab(window, cx);
        shell.select_tab(1, window, cx);
        shell.close_tab(1, window, cx);
    });
    assert_eq!(state(cx, &s.1), (2, 1));
}

#[gpui_kit::test]
fn closing_the_last_index_activates_the_new_last(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| {
        shell.new_tab(window, cx);
        shell.new_tab(window, cx);
        shell.close_tab(2, window, cx);
    });
    assert_eq!(state(cx, &s.1), (2, 1));
}

#[gpui_kit::test]
fn closing_a_tab_left_of_the_active_one_keeps_the_active_tab(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| {
        shell.new_tab(window, cx);
        shell.new_tab(window, cx);
        shell.close_tab(0, window, cx);
    });
    assert_eq!(state(cx, &s.1), (2, 1));
}

#[gpui_kit::test]
fn next_and_previous_wrap(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| {
        shell.new_tab(window, cx);
        shell.new_tab(window, cx);
        shell.next_tab(window, cx);
    });
    assert_eq!(state(cx, &s.1), (3, 0));
    act(cx, &s, |shell, window, cx| shell.prev_tab(window, cx));
    assert_eq!(state(cx, &s.1), (3, 2));
}

#[gpui_kit::test]
fn an_index_out_of_range_changes_nothing(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| {
        shell.new_tab(window, cx);
        shell.select_tab(9, window, cx);
        shell.close_tab(9, window, cx);
    });
    assert_eq!(state(cx, &s.1), (2, 1));
    assert_eq!(cx.windows().len(), 1);
}

#[gpui_kit::test]
fn ctrl_t_adds_a_tab_and_ctrl_w_closes_one(cx: &mut TestAppContext) {
    let s = shell(cx);
    cx.simulate_keystrokes(s.0, "ctrl-t");
    assert_eq!(state(cx, &s.1), (2, 1));
    cx.simulate_keystrokes(s.0, "ctrl-w");
    assert_eq!(state(cx, &s.1), (1, 0));
}

#[gpui_kit::test]
fn ctrl_tab_and_ctrl_shift_tab_move_between_tabs(cx: &mut TestAppContext) {
    let s = shell(cx);
    cx.simulate_keystrokes(s.0, "ctrl-t");
    cx.simulate_keystrokes(s.0, "ctrl-tab");
    assert_eq!(state(cx, &s.1), (2, 0));
    cx.simulate_keystrokes(s.0, "ctrl-shift-tab");
    assert_eq!(state(cx, &s.1), (2, 1));
}

#[gpui_kit::test]
fn clicking_a_tab_selects_it_and_its_close_button_closes_it(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| shell.new_tab(window, cx));
    cx.update_window(s.0, |_, window, cx| {
        window.within("tabs").click(0usize, cx);
    })
    .expect("window");
    assert_eq!(state(cx, &s.1), (2, 0));
    cx.update_window(s.0, |_, window, cx| {
        window.within("tabs").click(("tab-close", 0usize), cx);
    })
    .expect("window");
    assert_eq!(state(cx, &s.1), (1, 0));
}

#[gpui_kit::test]
fn the_first_tab_has_a_width(cx: &mut TestAppContext) {
    let s = shell(cx);
    let width = cx
        .update_window(s.0, |_, window, _| {
            window.within("tabs").find(0usize).bounds().size.width
        })
        .expect("window");
    assert!(f32::from(width) > 0.);
}

#[gpui_kit::test]
fn the_new_tabs_left_field_has_focus(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| shell.new_tab(window, cx));
    let side = cx
        .update_window(s.0, |_, window, cx| {
            s.1.read(cx)
                .active_screen()
                .and_then(|screen| screen.read(cx).focused_side(window, cx))
        })
        .expect("window");
    assert_eq!(side, Some(Side::Left));
}

#[gpui_kit::test]
fn closing_the_only_tab_closes_the_window(cx: &mut TestAppContext) {
    let s = shell(cx);
    act(cx, &s, |shell, window, cx| shell.close_tab(0, window, cx));
    cx.run_until_parked();
    assert_eq!(cx.windows().len(), 0);
}
