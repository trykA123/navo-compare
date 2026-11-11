//! Tests for the seeded bench data generator: counts, determinism and set L scaling.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::bench_data::{self, Error, Request, Set};
use crate::bench_data_large;

/// Owns one unique temporary folder and removes it at the end of a test.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Creates a unique empty folder named after `label`.
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("navo-bench-{}-{label}-{n}", std::process::id());
        let path = std::env::temp_dir().join(name);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    /// Returns the folder path.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    /// Removes the folder and everything the test wrote in it.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// One file in a generated tree, reduced to its path, size and content hash.
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    relative: String,
    len: u64,
    hash: u64,
}

/// Hashes a byte slice so two runs can be compared without keeping the bytes.
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Collects every file under `dir`, sorted by relative path, with size and content hash.
fn manifest(dir: &Path) -> Vec<Entry> {
    let mut entries = Vec::new();
    collect_dir(dir, dir, &mut entries);
    entries.sort_by(|a, b| a.relative.cmp(&b.relative));
    entries
}

/// Walks one directory recursively and appends its files to `entries`.
fn collect_dir(root: &Path, dir: &Path, entries: &mut Vec<Entry>) {
    for item in fs::read_dir(dir).unwrap() {
        let item = item.unwrap();
        let path = item.path();
        if path.is_dir() {
            collect_dir(root, &path, entries);
        } else {
            let bytes = fs::read(&path).unwrap();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            entries.push(Entry {
                relative,
                len: bytes.len() as u64,
                hash: hash_bytes(&bytes),
            });
        }
    }
}

/// Returns paths that exist on both sides with equal size but different content.
fn content_changes(left: &[Entry], right: &[Entry]) -> Vec<String> {
    let by_path: HashMap<&str, &Entry> = right
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect();
    left.iter()
        .filter_map(|entry| {
            let other = by_path.get(entry.relative.as_str())?;
            (entry.len == other.len && entry.hash != other.hash).then(|| entry.relative.clone())
        })
        .collect()
}

/// Returns the changed line numbers of a log pair.
fn changed_line_numbers(left: &[u8], right: &[u8]) -> Vec<u64> {
    left.as_chunks::<{ bench_data_large::LOG_LINE_BYTES }>()
        .0
        .iter()
        .zip(
            right
                .as_chunks::<{ bench_data_large::LOG_LINE_BYTES }>()
                .0
                .iter(),
        )
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .map(|(line, _)| line as u64)
        .collect()
}

/// Builds a request for `set` with a fixed seed and file count.
fn request<'a>(set: Set, seed: u64, out: &'a Path, files: Option<u64>) -> Request<'a> {
    Request {
        set,
        seed,
        out,
        files,
    }
}

/// Runs set S with 400 files and checks every count against the percentages.
#[test]
fn counts_match_percentages_for_400_files() {
    let temp = TempDir::new("counts");
    let summary = bench_data::run(&request(Set::S, 7, temp.path(), Some(400))).unwrap();
    assert_eq!(summary.files, 400);
    assert_eq!(summary.same, 384);
    assert_eq!(summary.changed, 8);
    assert_eq!(summary.resized, 4);
    assert_eq!(summary.only_left, 2);
    assert_eq!(summary.only_right, 2);
    assert!(summary.bytes > 0);

    let left = manifest(&temp.path().join("left"));
    let right = manifest(&temp.path().join("right"));
    assert_eq!(left.len(), 398);
    assert_eq!(right.len(), 398);
    assert_eq!(left.len() as u64, summary.files - summary.only_right);
    assert_eq!(right.len() as u64, summary.files - summary.only_left);
}

/// Two runs with one seed produce the same paths, sizes and bytes.
#[test]
fn same_seed_gives_same_manifest_and_bytes() {
    let first = TempDir::new("seed-a");
    let second = TempDir::new("seed-b");
    let a = bench_data::run(&request(Set::S, 42, first.path(), Some(400))).unwrap();
    let b = bench_data::run(&request(Set::S, 42, second.path(), Some(400))).unwrap();
    assert_eq!(a, b);
    assert_eq!(manifest(first.path()), manifest(second.path()));
}

/// Two different seeds produce different bytes.
#[test]
fn different_seeds_differ() {
    let first = TempDir::new("seed-c");
    let second = TempDir::new("seed-d");
    let a = bench_data::run(&request(Set::S, 1, first.path(), Some(400))).unwrap();
    let b = bench_data::run(&request(Set::S, 2, second.path(), Some(400))).unwrap();
    assert_ne!(a, b);
    assert_ne!(manifest(first.path()), manifest(second.path()));
}

/// Seed zero still produces non-zero content.
#[test]
fn seed_zero_writes_nonzero_bytes() {
    let temp = TempDir::new("seed-zero");
    let summary = bench_data::run(&request(Set::S, 0, temp.path(), Some(400))).unwrap();
    assert!(summary.bytes > 0);
    let left = manifest(&temp.path().join("left"));
    assert!(left.iter().any(|entry| entry.len > 0));
}

/// A non-empty output folder is refused and left untouched.
#[test]
fn non_empty_out_is_refused() {
    let temp = TempDir::new("non-empty");
    let out = temp.path().join("out");
    fs::create_dir(&out).unwrap();
    fs::write(out.join("keep.txt"), b"keep").unwrap();

    let error = bench_data::run(&request(Set::S, 7, &out, Some(400))).unwrap_err();
    assert!(matches!(error, Error::OutNotEmpty(_)));
    assert!(!out.join("left").exists());
    assert!(!out.join("right").exists());
    assert_eq!(fs::read(out.join("keep.txt")).unwrap(), b"keep");
}

/// Set L with `--files 64` differs in the scaled number of lines and bytes.
#[test]
fn set_l_scales_changed_lines_and_bytes() {
    let temp = TempDir::new("set-l");
    let summary = bench_data::run(&request(Set::L, 9, temp.path(), Some(64))).unwrap();
    let bytes = 64 * 1_024;
    assert_eq!(summary.files, 4);
    assert_eq!(summary.same, 0);
    assert_eq!(summary.changed, 0);
    assert_eq!(summary.resized, 0);
    assert_eq!(summary.only_left, 0);
    assert_eq!(summary.only_right, 0);
    assert_eq!(summary.bytes, bytes * 4);

    let left_log = fs::read(temp.path().join("left.log")).unwrap();
    let right_log = fs::read(temp.path().join("right.log")).unwrap();
    assert_eq!(left_log.len() as u64, bytes);
    assert_eq!(right_log.len() as u64, bytes);
    let lines = left_log
        .as_chunks::<{ bench_data_large::LOG_LINE_BYTES }>()
        .0
        .len() as u64;
    let expected_lines = (lines / 8).min(1_000);
    let changed_lines = left_log
        .as_chunks::<{ bench_data_large::LOG_LINE_BYTES }>()
        .0
        .iter()
        .zip(
            right_log
                .as_chunks::<{ bench_data_large::LOG_LINE_BYTES }>()
                .0
                .iter(),
        )
        .filter(|(left, right)| left != right)
        .count() as u64;
    assert_eq!(changed_lines, expected_lines);

    let left_bin = fs::read(temp.path().join("left.bin")).unwrap();
    let right_bin = fs::read(temp.path().join("right.bin")).unwrap();
    assert_eq!(left_bin.len() as u64, bytes);
    assert_eq!(right_bin.len() as u64, bytes);
    let expected_bytes = (bytes / 4_096).min(16);
    let changed_bytes = left_bin
        .iter()
        .zip(&right_bin)
        .filter(|(left, right)| left != right)
        .count() as u64;
    assert_eq!(changed_bytes, expected_bytes);
}

/// Set S uses all six folder depths and no folder holds more than 40 files.
#[test]
fn folder_depth_and_fanout_match_rule() {
    let temp = TempDir::new("depths");
    bench_data::run(&request(Set::S, 7, temp.path(), Some(400))).unwrap();
    let left = manifest(&temp.path().join("left"));
    let mut folders: HashMap<String, usize> = HashMap::new();
    let mut min_depth = usize::MAX;
    let mut max_depth = 0;
    for entry in &left {
        let depth = entry.relative.split('/').count() - 1;
        min_depth = min_depth.min(depth);
        max_depth = max_depth.max(depth);
        let folder = entry.relative.rsplit_once('/').unwrap().0;
        *folders.entry(folder.to_owned()).or_default() += 1;
    }
    assert_eq!(min_depth, 1);
    assert_eq!(max_depth, 6);
    assert!(folders.values().all(|&count| count <= 40));
    assert!(folders.values().any(|&count| count == 40));
}

/// Returns the generated index encoded in a path.
fn generated_index(relative: &str) -> u64 {
    let mut parts = relative.split('/');
    let folder = parts
        .next()
        .unwrap()
        .trim_start_matches('d')
        .parse::<u64>()
        .unwrap();
    let file = parts
        .next_back()
        .unwrap()
        .trim_start_matches('f')
        .trim_end_matches(".txt")
        .parse::<u64>()
        .unwrap();
    folder * 40 + file
}

/// Set S spreads its eight changed files instead of changing the first eight.
#[test]
fn changed_files_are_not_the_first_eight() {
    let temp = TempDir::new("changed-files");
    bench_data::run(&request(Set::S, 7, temp.path(), Some(400))).unwrap();
    let left = manifest(&temp.path().join("left"));
    let right = manifest(&temp.path().join("right"));
    let changed = content_changes(&left, &right);
    assert_eq!(changed.len(), 8);
    let mut indices: Vec<u64> = changed.iter().map(|path| generated_index(path)).collect();
    indices.sort_unstable();
    assert!(indices.iter().any(|&index| index < 200));
    assert!(indices.iter().any(|&index| index >= 200));
    assert_ne!(indices, (0..8).collect::<Vec<u64>>());
}

/// Set L spreads changed log lines and repeats them for the same seed.
#[test]
fn changed_log_lines_are_spread_and_seeded() {
    let first = TempDir::new("lines-a");
    let second = TempDir::new("lines-b");
    bench_data::run(&request(Set::L, 9, first.path(), Some(64))).unwrap();
    bench_data::run(&request(Set::L, 9, second.path(), Some(64))).unwrap();
    let left = fs::read(first.path().join("left.log")).unwrap();
    let right = fs::read(first.path().join("right.log")).unwrap();
    let left_again = fs::read(second.path().join("left.log")).unwrap();
    let right_again = fs::read(second.path().join("right.log")).unwrap();
    let changed = changed_line_numbers(&left, &right);
    let repeated = changed_line_numbers(&left_again, &right_again);
    assert_eq!(changed, repeated);
    let lines = left
        .as_chunks::<{ bench_data_large::LOG_LINE_BYTES }>()
        .0
        .len() as u64;
    assert!(changed.iter().any(|&line| line >= lines / 2));
}
