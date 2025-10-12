use super::config::RepoConfig;
use crate::ignore;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use md5::{Md5, Digest}; // Updated to use Md5 from digest crate
use serde::{Serialize, Deserialize}; // Added import serde
use ::ignore::gitignore; // Correct import: use ignore::gitignore

/// File status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
}

/// Detect file changes (compare current state with the last commit)
pub fn detect_changes(repo_root: &Path) -> Result<HashMap<PathBuf, FileStatus>, anyhow::Error> {
    let ignore_rules = crate::ignore::load_ignore_rules(repo_root)?; // Load ignore rules
    let current_files = collect_current_files(repo_root, &ignore_rules)?; // Collect current files
    let last_files = load_last_files(repo_root)?; // Load last commit files
    let mut changes = HashMap::new();

    // Detect added/modified files
    for (path, current_hash) in &current_files {
        match last_files.get(path) {
            None => changes.insert(path.clone(), FileStatus::Added),
            Some(last_hash) => {
                if current_hash != last_hash {
                    changes.insert(path.clone(), FileStatus::Modified)
                } else {
                    None
                }
            }
        };
    }

    // Detect deleted files
    for (path, _) in &last_files {
        if !current_files.contains_key(path) {
            changes.insert(path.clone(), FileStatus::Deleted);
        }
    }

    Ok(changes)
}

/// Collect hashes of current files (relative paths)
fn collect_current_files(
    repo_root: &Path,
     ignore_rules: &gitignore::Gitignore // Changed type to match the correct import
) -> Result<HashMap<PathBuf, String>, anyhow::Error> {
    let mut files = HashMap::new();
    for entry in WalkDir::new(repo_root).min_depth(1) {
        let entry = entry?;
        let path = entry.path();
        if ignore::is_ignored(ignore_rules, repo_root, path) {
            continue;
        }
        if path.is_file() {
            let rel_path = path.strip_prefix(repo_root)?.to_path_buf();
            let content = fs::read(path)?;
            // let hash = format!("{:x}", md5::compute(&content));
            let hash = format!("{:x}", Md5::digest(&content)); // Updated to use Md5 from digest crate
            files.insert(rel_path, hash);
        }
    }
    Ok(files)
}

/// Load file hashes from the last commit
fn load_last_files(repo_root: &Path) -> Result<HashMap<PathBuf, String>, anyhow::Error> {
    let last_path = repo_root.join(".quicksky/last_files.json");
    if last_path.exists() {
        let content = fs::read_to_string(last_path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(HashMap::new())
    }
}

/// Save current file hashes for the next comparison
pub fn save_current_files(repo_root: &Path) -> Result<(), anyhow::Error> {
    let ignore_rules = ignore::load_ignore_rules(repo_root)?;
    let current_files = collect_current_files(repo_root, &ignore_rules)?;
    let last_path = repo_root.join(".quicksky/last_files.json");
    fs::write(last_path, serde_json::to_string_pretty(&current_files)?)?; // Save as JSON
    Ok(())
}