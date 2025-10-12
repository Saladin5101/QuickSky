use anyhow::{Result, anyhow}; // Added import anyhow
use serde::{Serialize, Deserialize}; // Added import serde
use super::change::{detect_changes, save_current_files, FileStatus};
use super::config::RepoConfig;
use chrono::Local;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use ::ignore::gitignore; // Correct import: use ignore::gitignore
use uuid::Uuid;

fn generate_uuid_v4() -> String {
    use rand::Rng; 
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
  
    bytes[6] = (bytes[6] & 0x0F) | 0x40;

    bytes[8] = (bytes[8] & 0x3F) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11],
        bytes[12], bytes[13], bytes[14], bytes[15]
    )
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Commit {
    pub id: String,
    pub author: String,
    pub timestamp: String,
    pub message: String,
    pub changes: HashMap<PathBuf, FileStatus>,
}

impl Commit {
    /// Create a new commit
    pub fn create(repo_root: &Path, config: &RepoConfig, message: &str) -> Result<Self, anyhow::Error> {
        let changes = detect_changes(repo_root)?;
        if changes.is_empty() {
            return Err(anyhow!("No file changes detected, commit not necessary"));
        }

        let commit = Self {
            /*id: Builder::new()
                .set_variant(Variant::RFC4122)
                .set_version(Version::Random)
                .build()
                .to_string(),*/ // Old version using uuid crate
            id: generate_uuid_v4(),// New version using uuid crate
            author: config.user.name.clone(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            message: message.to_string(),
            changes: changes.clone(),
        };

        // Save the commit to .quicksky/commits
        let commit_dir = repo_root.join(".quicksky/commits");
        fs::create_dir_all(&commit_dir)?;
        let commit_path = commit_dir.join(format!("{}.bin", commit.id));
        fs::write(commit_path, bincode::serialize(&commit)?)?;

        // Update the last file hash
        save_current_files(repo_root)?;

        Ok(commit)
    }

    /// Load all commits (in reverse chronological order)
    pub fn load_all(repo_root: &Path) -> Result<Vec<Self>, anyhow::Error> {
        let commit_dir = repo_root.join(".quicksky/commits");
        if !commit_dir.exists() {
            return Ok(vec![]);
        }

        let mut commits = Vec::new();
        for entry in fs::read_dir(commit_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                let data = fs::read(path)?;
                let commit: Commit = bincode::deserialize(&data)?;
                commits.push(commit);
            }
        }

        // Sort by timestamp in descending order (newest first)
        commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(commits)
    }
}