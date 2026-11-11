//! Session records with resolved sides and engine rule values (NS9), plus the
//! format version and id rules the store enforces.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The layout version written to each session file.
pub const SESSION_VERSION: u32 = 1;

/// How many unnamed sessions the recent list keeps (NC8).
pub const RECENT_KEPT: usize = 20;

/// The whitespace values the compare engine understands (NS9).
pub(crate) const WHITESPACE_VALUES: [&str; 4] = ["keep", "all", "changes", "trailing"];

/// The filter values a session may store.
pub(crate) const FILTER_VALUES: [&str; 4] = ["all", "differences", "orphans", "same"];

/// One saved compare: two resolved sides, rules, filter and view stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Format version of the file.
    pub version: u32,
    /// File name stem, valid per NS5.
    pub id: String,
    /// User-given name; `None` makes it a recent session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the session compares folders or two texts.
    pub kind: SessionKind,
    /// Left side.
    pub left: SideRef,
    /// Right side.
    pub right: SideRef,
    /// Compare rules.
    pub rules: Rules,
    /// Active filter: `all`, `differences`, `orphans` or `same`.
    pub filter: String,
    /// When the session was opened, in Unix seconds.
    pub opened: i64,
    /// Opaque view stack of NC8.
    #[serde(default)]
    pub view: Vec<String>,
}

/// What a session compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionKind {
    /// Two folders or two Git revisions.
    Pair,
    /// Two pasted texts, which are never stored (NC26).
    Texts,
}

/// One resolved side: absolute path plus, for Git, the revision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SideRef {
    /// Whether the side is a path or a Git revision.
    pub kind: SideKind,
    /// Absolute path (NS9).
    pub path: PathBuf,
    /// Git revision; `None` for a path side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
}

/// Where a side's content comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SideKind {
    /// A file, folder or archive on disk.
    Path,
    /// A revision read through Git.
    Git,
}

/// The compare rules stored with a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    /// Whitespace handling: `keep`, `all`, `changes` or `trailing`.
    pub whitespace: String,
    /// Ignore a byte-order mark.
    pub ignore_bom: bool,
    /// Treat different line endings as equal.
    pub ignore_line_endings: bool,
    /// Compare case-insensitively.
    pub ignore_case: bool,
    /// Leave out files ignored by `.gitignore`.
    pub respect_gitignore: bool,
}

/// Returns true when `id` is 1–40 characters of `0-9`, `a-f` or `-` (NS5).
pub(crate) fn is_valid_id(id: &str) -> bool {
    (1..=40).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte) || byte == b'-')
}

/// Returns true when the whitespace value is one of the engine's four names.
pub(crate) fn rules_valid(rules: &Rules) -> bool {
    WHITESPACE_VALUES.contains(&rules.whitespace.as_str())
}

/// Returns true when the filter is one of the four stored names.
pub(crate) fn filter_valid(filter: &str) -> bool {
    FILTER_VALUES.contains(&filter)
}

/// Returns true when both sides are absolute and their rev matches their kind.
pub(crate) fn sides_valid(session: &Session) -> bool {
    side_valid(&session.left) && side_valid(&session.right)
}

/// Returns true when one side can be stored under NS9.
fn side_valid(side: &SideRef) -> bool {
    if !side.path.is_absolute() {
        return false;
    }
    match side.kind {
        SideKind::Path => side.rev.is_none(),
        SideKind::Git => side.rev.as_deref().is_some_and(rev_valid),
    }
}

/// Returns true when a Git revision can be stored: 1–256 bytes, not starting
/// with `-`, and without control characters.
fn rev_valid(rev: &str) -> bool {
    !rev.is_empty()
        && rev.len() <= 256
        && !rev.starts_with('-')
        && !rev.chars().any(char::is_control)
}
