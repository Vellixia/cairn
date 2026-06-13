//! Active guardrails (thin slice): verify a proposed edit against the current file before it is
//! accepted. The research motivating Cairn found frontier models silently delete ~25% of content
//! over long delegated edits; this catches large, unreplaced deletions and snapshots the original
//! into the blob store first, so the pre-edit state is always recoverable.

use cairn_core::Result;
use cairn_store::Store;
use serde::Serialize;
use similar::{ChangeTag, TextDiff};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Ok,
    Warn,
    Danger,
}

/// The outcome of verifying a proposed edit.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub path: String,
    /// Handle to the retained pre-edit original (None if the file didn't exist).
    pub baseline_hash: Option<String>,
    pub baseline_lines: usize,
    pub new_lines: usize,
    pub added: usize,
    pub removed: usize,
    pub removed_ratio: f32,
    pub risk: Risk,
    pub message: String,
}

pub struct Guard {
    store: Arc<Store>,
}

impl Guard {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Compare a proposed new version of `path` against the current on-disk file. The original is
    /// snapshotted into the blob store first so it is always recoverable.
    pub fn verify_edit(&self, path: &Path, new_content: &str) -> Result<VerifyReport> {
        let (baseline, baseline_hash) = match std::fs::read(path) {
            Ok(bytes) => {
                let hash = self.store.blobs().put(&bytes)?;
                (String::from_utf8_lossy(&bytes).into_owned(), Some(hash.0))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), None),
            Err(e) => return Err(e.into()),
        };

        let diff = TextDiff::from_lines(baseline.as_str(), new_content);
        let mut added = 0usize;
        let mut removed = 0usize;
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => added += 1,
                ChangeTag::Delete => removed += 1,
                ChangeTag::Equal => {}
            }
        }

        let baseline_lines = baseline.lines().count();
        let new_lines = new_content.lines().count();
        let removed_ratio = if baseline_lines == 0 {
            0.0
        } else {
            removed as f32 / baseline_lines as f32
        };

        let (risk, message) = if baseline.is_empty() {
            (Risk::Ok, format!("new file: {new_lines} lines"))
        } else if added == 0 && removed == 0 {
            (Risk::Ok, "no changes".to_string())
        } else if removed_ratio >= 0.5 && added < removed / 2 {
            (
                Risk::Danger,
                format!(
                    "removes {removed} of {baseline_lines} lines ({:.0}%) with little replacement — possible silent corruption",
                    removed_ratio * 100.0
                ),
            )
        } else if removed_ratio >= 0.2 {
            (
                Risk::Warn,
                format!(
                    "removes {removed} of {baseline_lines} lines ({:.0}%) — review before accepting",
                    removed_ratio * 100.0
                ),
            )
        } else {
            (Risk::Ok, format!("+{added} / -{removed} lines"))
        };

        Ok(VerifyReport {
            path: path.to_string_lossy().into_owned(),
            baseline_hash,
            baseline_lines,
            new_lines,
            added,
            removed,
            removed_ratio,
            risk,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairn_core::Config;

    fn guard() -> (Guard, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config::resolve(Some(dir.path().join("data"))).unwrap();
        (Guard::new(Arc::new(Store::open(&cfg).unwrap())), dir)
    }

    fn hundred_lines() -> String {
        (0..100).map(|i| format!("line {i}\n")).collect()
    }

    #[test]
    fn flags_large_unreplaced_deletion() {
        let (g, dir) = guard();
        let f = dir.path().join("f.txt");
        std::fs::write(&f, hundred_lines()).unwrap();
        let report = g.verify_edit(&f, "line 0\n").unwrap();
        assert_eq!(report.risk, Risk::Danger);
        assert!(report.baseline_hash.is_some(), "original must be retained");
    }

    #[test]
    fn small_append_is_ok() {
        let (g, dir) = guard();
        let f = dir.path().join("f.txt");
        let original = hundred_lines();
        std::fs::write(&f, &original).unwrap();
        let report = g.verify_edit(&f, &format!("{original}line 100\n")).unwrap();
        assert_eq!(report.risk, Risk::Ok);
    }

    #[test]
    fn new_file_is_ok_with_no_baseline() {
        let (g, dir) = guard();
        let f = dir.path().join("new.txt");
        let report = g.verify_edit(&f, "hello\nworld\n").unwrap();
        assert_eq!(report.risk, Risk::Ok);
        assert!(report.baseline_hash.is_none());
    }
}
