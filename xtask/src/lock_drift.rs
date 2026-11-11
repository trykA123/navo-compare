//! Lock drift between this repository and another lockfile (NS14).
//! Only sourced packages count; workspace crates have no `source` line.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// One package name whose versions on our side are not all on the other side.
#[derive(Debug, PartialEq, Eq)]
pub struct Drift {
    /// Package name.
    pub name: String,
    /// Versions held by this repository, sorted as text.
    pub ours: Vec<String>,
    /// Versions held by the other lockfile, sorted as text.
    pub theirs: Vec<String>,
}

/// Result of comparing two lockfiles.
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    /// Number of package names held by both lockfiles.
    pub shared: usize,
    /// Drifting packages, sorted by name.
    pub drifts: Vec<Drift>,
}

/// Why a lockfile could not be read or parsed.
#[derive(Debug)]
pub enum LockError {
    /// The file could not be read.
    Read {
        /// Path that failed.
        path: PathBuf,
        /// Underlying filesystem error.
        error: std::io::Error,
    },
    /// A package table has no name or version.
    Malformed {
        /// Path that failed.
        path: PathBuf,
        /// 1-based line of the package table.
        line: usize,
    },
}

impl fmt::Display for LockError {
    /// Writes the user-facing error sentence.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, error } => {
                write!(
                    formatter,
                    "cannot read lockfile {}: {error}",
                    path.display()
                )
            }
            Self::Malformed { path, line } => {
                write!(
                    formatter,
                    "malformed lockfile {} at line {line}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for LockError {}

/// Package names mapped to every version held for that name.
pub type Versions = BTreeMap<String, BTreeSet<String>>;

/// One package table while it is being read.
struct Package {
    /// 1-based line of the `[[package]]` table.
    start: usize,
    /// Value of the `name` key.
    name: Option<String>,
    /// Value of the `version` key.
    version: Option<String>,
    /// Whether a `source` key was seen.
    sourced: bool,
}

impl Package {
    /// Starts a package table at line `start` with every key unset.
    fn new(start: usize) -> Self {
        Self {
            start,
            name: None,
            version: None,
            sourced: false,
        }
    }

    /// Stores the package when sourced, or returns its table line when broken.
    fn finish(self, versions: &mut Versions) -> Result<(), usize> {
        let Some(name) = self.name else {
            return Err(self.start);
        };
        let Some(version) = self.version else {
            return Err(self.start);
        };
        if self.sourced {
            versions.entry(name).or_default().insert(version);
        }
        Ok(())
    }
}

/// Parses sourced packages from `text`, or returns the 1-based bad line.
pub fn versions(text: &str) -> Result<Versions, usize> {
    let mut versions = Versions::new();
    let mut current: Option<Package> = None;
    for (offset, raw) in text.lines().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let number = offset + 1;
        if line.starts_with('[') {
            if let Some(package) = current.take() {
                package.finish(&mut versions)?;
            }
            if line.trim_end() == "[[package]]" {
                current = Some(Package::new(number));
            }
            continue;
        }
        let Some(package) = current.as_mut() else {
            continue;
        };
        if let Some(value) = quoted_value(line, "name") {
            package.name = Some(value.to_owned());
        } else if let Some(value) = quoted_value(line, "version") {
            package.version = Some(value.to_owned());
        } else if quoted_key(line) == Some("source") {
            package.sourced = true;
        }
    }
    if let Some(package) = current {
        package.finish(&mut versions)?;
    }
    Ok(versions)
}

/// Returns the quoted value of `key` on `line`, if the line holds that key.
fn quoted_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (found, value) = line.split_once('=')?;
    if found.trim() != key {
        return None;
    }
    value.trim().strip_prefix('"')?.strip_suffix('"')
}

/// Returns the key on one `key = value` line.
fn quoted_key(line: &str) -> Option<&str> {
    line.split_once('=').map(|(key, _)| key.trim())
}

/// Compares two parsed lockfiles and reports shared names and drifts.
pub fn compare(ours: &Versions, theirs: &Versions) -> Report {
    let mut shared = 0;
    let mut drifts = Vec::new();
    for (name, our_versions) in ours {
        let Some(their_versions) = theirs.get(name) else {
            continue;
        };
        shared += 1;
        if !our_versions.is_subset(their_versions) {
            drifts.push(Drift {
                name: name.clone(),
                ours: our_versions.iter().cloned().collect(),
                theirs: their_versions.iter().cloned().collect(),
            });
        }
    }
    Report { shared, drifts }
}

/// Reads both lockfiles, parses them and compares their versions.
pub fn run(ours: &Path, theirs: &Path) -> Result<Report, LockError> {
    let our_text = read_lock(ours)?;
    let our_versions = versions(&our_text).map_err(|line| LockError::Malformed {
        path: ours.to_path_buf(),
        line,
    })?;
    let their_text = read_lock(theirs)?;
    let their_versions = versions(&their_text).map_err(|line| LockError::Malformed {
        path: theirs.to_path_buf(),
        line,
    })?;
    Ok(compare(&our_versions, &their_versions))
}

/// Reads one lockfile as text, naming it when the read fails.
fn read_lock(path: &Path) -> Result<String, LockError> {
    fs::read_to_string(path).map_err(|error| LockError::Read {
        path: path.to_path_buf(),
        error,
    })
}
