//! The main window of Sidewise: starts the app and opens the themed shell.

use navo_compare_profile::brand;
use navo_ui::prelude::*;
use navo_ui::tokens::Mode;

use crate::shell::{self, Shell};

/// Starts the GPUI app, opens the window and returns the process exit code (0 after a normal close).
pub fn run() -> std::process::ExitCode {
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let run_failed = failed.clone();
    navo_ui::application().run(move |cx| {
        let initialised = navo_ui::init(Mode::Light, cx);
        shell::bind_keys(cx);
        let opened = initialised
            .map_err(|error| error.to_string())
            .and_then(|()| open(cx));
        if let Err(message) = opened {
            report(&message);
            run_failed.store(true, std::sync::atomic::Ordering::SeqCst);
            cx.quit();
        }
    });
    if failed.load(std::sync::atomic::Ordering::SeqCst) {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

/// Opens the 1,200 x 760 window titled with the product name.
fn open(cx: &mut App) -> Result<(), String> {
    let bounds = Bounds::centered(None, size(px(1200.), px(760.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        app_id: Some(brand::SLUG.into()),
        ..navo_ui::window_options(brand::NAME)
    };
    navo_ui::open_window(options, cx, |window, cx| {
        cx.new(|cx| Shell::new(window, cx))
    })
    .map(|_| ())
    .map_err(|error| format!("could not open the window: {error:#}"))
}

/// Prints a start-up failure on the terminal.
#[expect(clippy::print_stderr, reason = "start-up failures go to the terminal")]
fn report(message: &str) {
    eprintln!("{}: {message}", brand::SLUG);
}
