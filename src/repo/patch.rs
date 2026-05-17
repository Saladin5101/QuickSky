use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use crate::repo::change::FileStatus;
use crate::repo::commit::Commit;

/// Generate a unified diff patch string from a commit and write it to `output_path`.
/// Returns the path written.
pub fn format_patch(repo_root: &Path, commit: &Commit, output_path: &Path) -> Result<PathBuf> {
    let mut patch = String::new();

    // Patch header
    patch.push_str(&format!("From: {}\n", commit.author));
    patch.push_str(&format!("Date: {}\n", commit.timestamp));
    patch.push_str(&format!("Subject: {}\n\n", commit.message));

    for (rel_path, status) in &commit.changes {
        let path_str = rel_path.to_string_lossy().replace('\\', "/");

        match status {
            FileStatus::Added => {
                let content = fs::read_to_string(repo_root.join(rel_path))
                    .unwrap_or_default();
                patch.push_str(&format!("--- /dev/null\n+++ b/{}\n", path_str));
                patch.push_str(&format!("@@ -0,0 +1,{} @@\n", content.lines().count()));
                for line in content.lines() {
                    patch.push_str(&format!("+{}\n", line));
                }
            }
            FileStatus::Deleted => {
                patch.push_str(&format!("--- a/{}\n+++ /dev/null\n", path_str));
                patch.push_str("@@ -1,0 +0,0 @@\n");
                patch.push_str(&format!("-<deleted: {}>\n", path_str));
            }
            FileStatus::Modified => {
                let current = fs::read_to_string(repo_root.join(rel_path))
                    .unwrap_or_default();
                patch.push_str(&format!("--- a/{}\n+++ b/{}\n", path_str, path_str));
                patch.push_str(&unified_diff("", &current, &path_str));
            }
        }
        patch.push('\n');
    }

    fs::write(output_path, &patch)?;
    Ok(output_path.to_path_buf())
}

/// Minimal unified diff between old and new text.
/// Since QuickSky doesn't store old file content in commits (only hashes),
/// for Modified files we show the full new content as context.
fn unified_diff(old: &str, new: &str, _path: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let mut out = String::new();

    out.push_str(&format!("@@ -{},{} +{},{} @@\n",
        1, old_lines.len(), 1, new_lines.len()));

    for line in &old_lines {
        out.push_str(&format!("-{}\n", line));
    }
    for line in &new_lines {
        out.push_str(&format!("+{}\n", line));
    }
    out
}

/// Load HEAD commit (most recent)
pub fn head_commit(repo_root: &Path) -> Result<Commit> {
    Commit::load_all(repo_root)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No commits found"))
}
