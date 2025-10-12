use ignore::gitignore::{Gitignore, GitignoreBuilder};  // 1. Correct import: use ignore::Gitignore
use std::path::{Path};
use anyhow::{Result, anyhow}; // Add import anyhow

/// Load .skyhide and .gitignore rules (.skyhide takes precedence)
pub fn load_ignore_rules(repo_root: &Path) -> Result<Gitignore, anyhow::Error> {
    let mut builder = GitignoreBuilder::new(repo_root);
    let skyhide = repo_root.join(".skyhide");
    if skyhide.exists() {
        // Fix usage of ? (builder.add returns Option, convert to Result)
        builder.add(skyhide).ok_or_else(|| anyhow!("Failed to load .skyhide"))?;
    }
    let gitignore = repo_root.join(".gitignore");
    if gitignore.exists() {
        builder.add(gitignore).ok_or_else(|| anyhow!("Failed to load .gitignore"))?;
    }
    Ok(builder.build()?)
}

/// Check if a file is ignored
pub fn is_ignored(ignore_rules: &Gitignore, repo_root: &Path, file_path: &Path) -> bool {
    let rel_path = file_path.strip_prefix(repo_root).unwrap_or(file_path);
    ignore_rules.matched(rel_path, file_path.is_dir()).is_ignore()
}