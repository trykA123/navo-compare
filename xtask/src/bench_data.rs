//! Seeded bench data generator for the `bench-data` xtask command.
//! It writes sets S and M as two folder trees; set L lives in `bench_data_large`.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bench_data_large;
use crate::bench_rng::{Rng, pick_positions};

/// File count of set S when `--files` is absent.
const DEFAULT_S_FILES: u64 = 2_000;
/// File count of set M when `--files` is absent.
const DEFAULT_M_FILES: u64 = 200_000;
/// Files per output folder, as the plan caps it.
const FILES_PER_FOLDER: u64 = 40;
/// Folder depths used in turn by the generated tree.
const FOLDER_DEPTHS: u64 = 6;
/// Largest generated text file in bytes.
const MAX_FILE_BYTES: u64 = 4_096;
/// Characters used for generated text.
const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,;:-_/\n";

/// A generated set from NC33.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    /// 2,000 generated files by default.
    S,
    /// 200,000 generated files by default.
    M,
    /// Two log pairs and two binary pairs, 1 GiB each by default.
    L,
}

/// One generator request parsed from the command line.
pub struct Request<'a> {
    /// Set to generate.
    pub set: Set,
    /// Seed for the deterministic pseudo-random generator.
    pub seed: u64,
    /// Output folder, which must be missing or empty.
    pub out: &'a Path,
    /// File count override, used by the tests.
    pub files: Option<u64>,
}

/// Counts and total bytes of one generated set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// Union file count across both sides.
    pub files: u64,
    /// Files with identical bytes on both sides.
    pub same: u64,
    /// Files with a content change and the same size.
    pub changed: u64,
    /// Files with a size change.
    pub resized: u64,
    /// Files that exist only on the left.
    pub only_left: u64,
    /// Files that exist only on the right.
    pub only_right: u64,
    /// Total bytes written across both sides.
    pub bytes: u64,
}

/// Everything the generator can refuse or fail on.
#[derive(Debug)]
pub enum Error {
    /// The output folder exists and holds at least one entry.
    OutNotEmpty(PathBuf),
    /// An argument or count is outside the contract.
    InvalidInput(String),
    /// A filesystem call failed.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    /// Writes the user-facing error text.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutNotEmpty(path) => {
                write!(formatter, "output folder is not empty: {}", path.display())
            }
            Self::InvalidInput(message) => write!(formatter, "{message}"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl From<std::io::Error> for Error {
    /// Wraps a filesystem error without changing it.
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Generates the requested set and returns its summary.
pub fn run(request: &Request<'_>) -> Result<Summary, Error> {
    let files = effective_files(request)?;
    check_out(request.out)?;
    fs::create_dir_all(request.out)?;
    match request.set {
        Set::S | Set::M => write_folders(request.out, request.seed, files),
        Set::L => bench_data_large::write_set_l(request.out, request.seed, files),
    }
}

/// Resolves the file count, refusing zero.
fn effective_files(request: &Request<'_>) -> Result<u64, Error> {
    let default = match request.set {
        Set::S => DEFAULT_S_FILES,
        Set::M => DEFAULT_M_FILES,
        Set::L => bench_data_large::default_files(),
    };
    let files = request.files.unwrap_or(default);
    if files == 0 {
        return Err(Error::InvalidInput("--files must be at least 1".to_owned()));
    }
    Ok(files)
}

/// Refuses an output path that exists as a file or holds any entry.
fn check_out(out: &Path) -> Result<(), Error> {
    match fs::metadata(out) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(Error::InvalidInput(format!(
                    "output path is not a folder: {}",
                    out.display()
                )));
            }
            let mut entries = fs::read_dir(out)?;
            if entries.next().is_some() {
                return Err(Error::OutNotEmpty(out.to_path_buf()));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::Io(error)),
    }
}

/// Returns `files * basis_points / 10,000`, rounded half up.
fn rounded_share(files: u64, basis_points: u64) -> u64 {
    let numerator = u128::from(files) * u128::from(basis_points) + 5_000;
    (numerator / 10_000) as u64
}

/// Writes the left and right trees for sets S and M.
fn write_folders(out: &Path, seed: u64, files: u64) -> Result<Summary, Error> {
    let changed = rounded_share(files, 200);
    let resized = rounded_share(files, 100);
    let only_left = rounded_share(files, 50);
    let only_right = rounded_share(files, 50);
    let same = files - changed - resized - only_left - only_right;
    let mut rng = Rng::new(seed);
    let positions = pick_positions(&mut rng, files, changed);
    let mut resized_left = resized;
    let mut left_left = only_left;
    let mut right_left = only_right;
    let mut bytes = 0_u64;
    for index in 0..files {
        let relative = relative_path(index);
        bytes += if positions.binary_search(&index).is_ok() {
            write_changed(out, &relative, &mut rng)?
        } else if resized_left > 0 {
            resized_left -= 1;
            write_resized(out, &relative, &mut rng)?
        } else if left_left > 0 {
            left_left -= 1;
            write_only(out, &relative, &mut rng, "left")?
        } else if right_left > 0 {
            right_left -= 1;
            write_only(out, &relative, &mut rng, "right")?
        } else {
            write_same(out, &relative, &mut rng)?
        };
    }
    Ok(Summary {
        files,
        same,
        changed,
        resized,
        only_left,
        only_right,
        bytes,
    })
}

/// Maps one union index to a path at 1 + (index / 40) % 6 folder levels.
fn relative_path(index: u64) -> PathBuf {
    let block = index / FILES_PER_FOLDER;
    let depth = 1 + block % FOLDER_DEPTHS;
    let file = index % FILES_PER_FOLDER;
    let folder = format!("d{block:06}");
    let mut path = PathBuf::new();
    for _ in 0..depth {
        path.push(&folder);
    }
    path.join(format!("f{file:02}.txt"))
}

/// Writes identical bytes to both sides and returns the bytes written.
fn write_same(out: &Path, relative: &Path, rng: &mut Rng) -> Result<u64, Error> {
    let len = random_len(rng);
    let content = generated_text(rng, len);
    write_side(out, "left", relative, &content)?;
    write_side(out, "right", relative, &content)?;
    Ok((len as u64) * 2)
}

/// Writes a same-size content change to both sides and returns the bytes written.
fn write_changed(out: &Path, relative: &Path, rng: &mut Rng) -> Result<u64, Error> {
    let len = random_len(rng);
    let left = generated_text(rng, len);
    let mut right = generated_text(rng, len);
    if left == right {
        right[0] = flip(right[0]);
    }
    write_side(out, "left", relative, &left)?;
    write_side(out, "right", relative, &right)?;
    Ok((len as u64) * 2)
}

/// Writes a size change to both sides and returns the bytes written.
fn write_resized(out: &Path, relative: &Path, rng: &mut Rng) -> Result<u64, Error> {
    let left_len = random_len(rng);
    let mut right_len = random_len(rng);
    if left_len == right_len {
        right_len = if left_len == MAX_FILE_BYTES as usize {
            left_len - 1
        } else {
            left_len + 1
        };
    }
    let left = generated_text(rng, left_len);
    let right = generated_text(rng, right_len);
    write_side(out, "left", relative, &left)?;
    write_side(out, "right", relative, &right)?;
    Ok((left_len + right_len) as u64)
}

/// Writes one file on one side and returns its byte length.
fn write_only(out: &Path, relative: &Path, rng: &mut Rng, side: &str) -> Result<u64, Error> {
    let len = random_len(rng);
    let content = generated_text(rng, len);
    write_side(out, side, relative, &content)?;
    Ok(len as u64)
}

/// Returns one generated length in the inclusive 1..=4,096 range.
fn random_len(rng: &mut Rng) -> usize {
    1 + (rng.next_u64() % MAX_FILE_BYTES) as usize
}

/// Builds `len` bytes of printable generated text.
fn generated_text(rng: &mut Rng, len: usize) -> Vec<u8> {
    let mut text = Vec::with_capacity(len);
    for _ in 0..len {
        let index = (rng.next_u64() % ALPHABET.len() as u64) as usize;
        text.push(ALPHABET[index]);
    }
    text
}

/// Returns a different printable byte for an existing one.
pub(crate) fn flip(byte: u8) -> u8 {
    if byte == b'a' { b'b' } else { b'a' }
}

/// Writes `bytes` to `<out>/<side>/<relative>`, creating its folder.
fn write_side(out: &Path, side: &str, relative: &Path, bytes: &[u8]) -> Result<(), Error> {
    let path = out.join(side).join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}
