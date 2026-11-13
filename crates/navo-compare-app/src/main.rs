//! Entry point of the app (its name is in `navo_compare_profile::brand`). `--version` prints the version;
//! otherwise the window opens with the shell (tabs and the start screen).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg_attr(not(test), expect(dead_code, reason = "U11c and the chrome read it"))]
mod icons;
mod shell;
#[cfg(test)]
mod shell_tests;
mod start;
mod start_rows;
#[cfg(test)]
mod start_rows_tests;
#[cfg(test)]
mod start_tests;
mod start_view;
#[cfg(test)]
mod test_window;
mod theme;
#[expect(dead_code, reason = "the theme injector wires these tokens in U12a")]
mod theme_data;
#[cfg(test)]
mod theme_tests;
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
