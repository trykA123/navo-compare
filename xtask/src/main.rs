//! Developer commands for this repository, run as `cargo xtask <command>`.
//! `bench-data` writes seeded bench sets; `lock-drift` compares lockfiles.

mod bench_data;
mod bench_data_large;
#[cfg(test)]
mod bench_data_tests;
mod bench_rng;
mod lock_drift;
#[cfg(test)]
mod lock_drift_tests;

use std::path::PathBuf;
use std::process::ExitCode;

use bench_data::{Error, Request, Set};

/// Options parsed from one `bench-data` command line.
struct Options {
    /// Set to generate.
    set: Set,
    /// Seed for the generator.
    seed: u64,
    /// Output folder.
    out: PathBuf,
    /// File count override.
    files: Option<u64>,
}

/// Dispatches one command and returns its process exit code.
#[expect(
    clippy::print_stderr,
    reason = "command-line tool: errors go to the terminal"
)]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("xtask: no command given");
        return ExitCode::from(2);
    };
    match command.as_str() {
        "bench-data" => run_bench_data(args.collect()),
        "lock-drift" => run_lock_drift(args.collect()),
        other => {
            eprintln!("xtask: unknown command '{other}'");
            ExitCode::from(2)
        }
    }
}

/// Runs `bench-data` with the remaining arguments and prints one summary line.
#[expect(
    clippy::print_stderr,
    clippy::print_stdout,
    reason = "command-line tool: output and errors go to the terminal"
)]
fn run_bench_data(args: Vec<String>) -> ExitCode {
    let options = match parse_options(&args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("xtask bench-data: {message}");
            return ExitCode::from(2);
        }
    };
    if options.set == Set::L {
        println!("bytes={}", bench_data_large::l_total_bytes(options.files));
    }
    let request = Request {
        set: options.set,
        seed: options.seed,
        out: &options.out,
        files: options.files,
    };
    match bench_data::run(&request) {
        Ok(summary) => {
            println!(
                "files={} same={} changed={} resized={} only_left={} only_right={} bytes={}",
                summary.files,
                summary.same,
                summary.changed,
                summary.resized,
                summary.only_left,
                summary.only_right,
                summary.bytes
            );
            ExitCode::SUCCESS
        }
        Err(Error::OutNotEmpty(path)) => {
            eprintln!(
                "xtask bench-data: output folder is not empty: {}",
                path.display()
            );
            ExitCode::from(2)
        }
        Err(Error::InvalidInput(message)) => {
            eprintln!("xtask bench-data: {message}");
            ExitCode::from(2)
        }
        Err(Error::Io(error)) => {
            eprintln!("xtask bench-data: {error}");
            ExitCode::from(1)
        }
    }
}

/// Runs `lock-drift` against `--other` and prints one line per drift.
#[expect(
    clippy::print_stderr,
    clippy::print_stdout,
    reason = "command-line tool: output and errors go to the terminal"
)]
fn run_lock_drift(args: Vec<String>) -> ExitCode {
    let (ours, theirs) = match parse_lock_options(&args) {
        Ok(paths) => paths,
        Err(message) => {
            eprintln!("xtask lock-drift: {message}");
            return ExitCode::from(2);
        }
    };
    match lock_drift::run(&ours, &theirs) {
        Ok(report) => {
            for drift in &report.drifts {
                println!(
                    "{}: ours {}; theirs {}",
                    drift.name,
                    drift.ours.join(", "),
                    drift.theirs.join(", ")
                );
            }
            println!("drift={} shared={}", report.drifts.len(), report.shared);
            if report.drifts.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("xtask lock-drift: {error}");
            ExitCode::from(2)
        }
    }
}

/// Parses `--other <file>` and an optional `--ours <file>`.
fn parse_lock_options(args: &[String]) -> Result<(PathBuf, PathBuf), String> {
    let mut ours = None;
    let mut other = None;
    let mut index = 0;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("'{flag}' needs a value"))?;
        match flag.as_str() {
            "--other" => other = Some(PathBuf::from(value)),
            "--ours" => ours = Some(PathBuf::from(value)),
            unknown => return Err(format!("unknown option '{unknown}'")),
        }
        index += 2;
    }
    Ok((
        ours.unwrap_or_else(|| PathBuf::from("Cargo.lock")),
        other.ok_or_else(|| "--other is required".to_owned())?,
    ))
}

/// Parses `--set`, `--seed`, `--out` and an optional `--files`.
fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut set = None;
    let mut seed = None;
    let mut out = None;
    let mut files = None;
    let mut index = 0;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("'{flag}' needs a value"))?;
        match flag.as_str() {
            "--set" => set = Some(parse_set(value)?),
            "--seed" => {
                seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("invalid seed '{value}'"))?,
                );
            }
            "--out" => out = Some(PathBuf::from(value)),
            "--files" => {
                let count = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid file count '{value}'"))?;
                if count == 0 {
                    return Err("--files must be at least 1".to_owned());
                }
                files = Some(count);
            }
            other => return Err(format!("unknown option '{other}'")),
        }
        index += 2;
    }
    Ok(Options {
        set: set.ok_or_else(|| "--set is required".to_owned())?,
        seed: seed.ok_or_else(|| "--seed is required".to_owned())?,
        out: out.ok_or_else(|| "--out is required".to_owned())?,
        files,
    })
}

/// Parses the set name `s`, `m` or `l`.
fn parse_set(value: &str) -> Result<Set, String> {
    match value {
        "s" => Ok(Set::S),
        "m" => Ok(Set::M),
        "l" => Ok(Set::L),
        other => Err(format!("invalid set '{other}', expected s, m or l")),
    }
}
