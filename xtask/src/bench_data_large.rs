//! Set L of the bench data generator: two large logs and two large binaries.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::bench_data::{Error, Summary, flip};
use crate::bench_rng::{Rng, pick_positions, splitmix64};

/// Unit count that makes each set L file 1 GiB.
const DEFAULT_L_FILES: u64 = 1_048_576;
/// Bytes in one set L unit (one per `--files`).
const L_UNIT_BYTES: u64 = 1_024;
/// Bytes in one generated log line, including the newline.
pub const LOG_LINE_BYTES: usize = 64;
/// Bytes of one binary write chunk.
const BINARY_CHUNK: usize = 1_048_576;

/// Returns the default set L unit count.
pub fn default_files() -> u64 {
    DEFAULT_L_FILES
}

/// Returns the total bytes of set L for an optional `--files` unit count.
pub fn l_total_bytes(files: Option<u64>) -> u64 {
    files
        .unwrap_or(DEFAULT_L_FILES)
        .saturating_mul(L_UNIT_BYTES)
        .saturating_mul(4)
}

/// Writes set L and returns its summary.
pub fn write_set_l(out: &Path, seed: u64, units: u64) -> Result<Summary, Error> {
    let bytes = units.saturating_mul(L_UNIT_BYTES);
    let line_count = bytes / LOG_LINE_BYTES as u64;
    let changed_lines = (line_count / 8).min(1_000);
    let changed_bytes = (bytes / 4_096).min(16);
    let mut total = write_log_pair(out, seed, bytes, changed_lines)?;
    total += write_binary_pair(out, seed, bytes, changed_bytes)?;
    Ok(Summary {
        files: 4,
        bytes: total,
        ..Summary::default()
    })
}

/// Writes the two logs, changing `changed` spread lines, and returns their bytes.
fn write_log_pair(out: &Path, seed: u64, bytes: u64, changed: u64) -> Result<u64, Error> {
    let mut left = BufWriter::new(File::create(out.join("left.log"))?);
    let mut right = BufWriter::new(File::create(out.join("right.log"))?);
    let lines = bytes / LOG_LINE_BYTES as u64;
    let mut rng = Rng::new(seed ^ 0x30);
    let positions = pick_positions(&mut rng, lines, changed);
    let seed_word = splitmix64(seed ^ 0x10);
    let mut next = 0;
    for line in 0..lines {
        let content = log_line(seed_word, line);
        left.write_all(&content)?;
        let mut other = content;
        if positions.get(next) == Some(&line) {
            other[0] = flip(other[0]);
            next += 1;
        }
        right.write_all(&other)?;
    }
    left.flush()?;
    right.flush()?;
    Ok(bytes.saturating_mul(2))
}

/// Builds one 64-byte log line from the seed and line number.
fn log_line(seed: u64, line: u64) -> [u8; LOG_LINE_BYTES] {
    let text = format!("line {line:016x} seed {seed:016x}");
    let mut content = [b' '; LOG_LINE_BYTES];
    let copied = text.len().min(LOG_LINE_BYTES - 1);
    content[..copied].copy_from_slice(&text.as_bytes()[..copied]);
    content[LOG_LINE_BYTES - 1] = b'\n';
    content
}

/// Writes the two binaries, changing `changed` spread bytes, and returns their bytes.
fn write_binary_pair(out: &Path, seed: u64, bytes: u64, changed: u64) -> Result<u64, Error> {
    let mut left = BufWriter::new(File::create(out.join("left.bin"))?);
    let mut right = BufWriter::new(File::create(out.join("right.bin"))?);
    let mut rng = Rng::new(seed ^ 0x20);
    let positions = pick_positions(&mut rng, bytes, changed);
    let mut buffer = vec![0_u8; BINARY_CHUNK];
    let mut offset = 0_u64;
    while offset < bytes {
        let len = (bytes - offset).min(BINARY_CHUNK as u64) as usize;
        let chunk = &mut buffer[..len];
        rng.fill(chunk);
        left.write_all(chunk)?;
        let mut other = chunk.to_vec();
        for &position in &positions {
            if position >= offset && position < offset + len as u64 {
                other[(position - offset) as usize] ^= 0xFF;
            }
        }
        right.write_all(&other)?;
        offset += len as u64;
    }
    left.flush()?;
    right.flush()?;
    Ok(bytes.saturating_mul(2))
}
