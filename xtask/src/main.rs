//! Developer commands for this repository, run as `cargo xtask <command>`.
//! Commands arrive with their atoms; for now every call reports that none exists.

use std::process::ExitCode;

/// Reports the unknown command on stderr and returns exit code 2.
#[expect(
    clippy::print_stderr,
    reason = "command-line tool: errors go to stderr"
)]
fn main() -> ExitCode {
    let command = std::env::args().nth(1).unwrap_or_default();
    eprintln!("xtask: unknown command '{command}'");
    ExitCode::from(2)
}
