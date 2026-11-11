//! Entry point of the app (its name is in `navo_compare_profile::brand`). `--version` prints the version; otherwise one empty window
//! opens (spike U0; the screens arrive with Stage 2 of `plans/2026-11-11-standalone.md`).

mod shell;
#[cfg(test)]
mod shell_tests;
#[cfg(test)]
mod test_window;
mod window;

use std::process::ExitCode;

/// Prints the version for `--version`; any other call opens the window.
/// Returns exit code 0 for `--version` and the window's exit code otherwise.
#[expect(clippy::print_stdout, reason = "command-line output: the version line")]
fn main() -> ExitCode {
    let wants_version = std::env::args().skip(1).any(|arg| arg == "--version");
    if wants_version {
        println!(
            "{} {}",
            navo_compare_profile::brand::SLUG,
            env!("CARGO_PKG_VERSION")
        );
        return ExitCode::SUCCESS;
    }
    window::run()
}
