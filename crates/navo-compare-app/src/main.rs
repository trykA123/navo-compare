//! Entry point of Navo Compare. For now it only answers `--version`; the window shell
//! arrives with Stage 2 of `plans/2026-11-11-standalone.md`.

use std::process::ExitCode;

/// Prints the version for `--version`; any other call says that the app is not built yet.
/// Returns exit code 0 for `--version` and 2 otherwise.
fn main() -> ExitCode {
    let wants_version = std::env::args().skip(1).any(|arg| arg == "--version");
    if wants_version {
        println!("navo-compare {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    eprintln!("navo-compare: the app shell is not built yet (see plans/2026-11-11-standalone.md)");
    ExitCode::from(2)
}
