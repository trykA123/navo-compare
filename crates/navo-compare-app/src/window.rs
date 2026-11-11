//! The main window of Navo Compare: one themed, empty window (spike U0, before the screens).

use navo_compare_profile::brand;
use navo_ui::prelude::*;
use navo_ui::tokens::{self, Mode};

/// Root view: a full-size box in the theme's background colour and nothing else.
struct Blank;

impl Render for Blank {
    /// Draws the empty background from the current theme.
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(tokens::current(cx).colors.background)
    }
}

/// Starts the GPUI app, opens the window and returns the process exit code (0 after a normal close).
pub fn run() -> std::process::ExitCode {
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let run_failed = failed.clone();
    navo_ui::application().run(move |cx| {
        let initialised = navo_ui::init(Mode::Light, cx);
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
    navo_ui::open_window(options, cx, |_window, cx| cx.new(|_| Blank))
        .map(|_| ())
        .map_err(|error| format!("could not open the window: {error:#}"))
}

/// Prints a start-up failure on the terminal.
#[expect(clippy::print_stderr, reason = "start-up failures go to the terminal")]
fn report(message: &str) {
    eprintln!("{}: {message}", brand::SLUG);
}
