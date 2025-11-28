use ignore::gitignore::{Gitignore, GitignoreBuilder};  // 1. Correct import: use ignore::Gitignore
use std::path::{Path};
use anyhow::{Result, anyhow}; // Add import anyhow

/// Load .skyhide rules (QuickSky's native ignore file)
pub fn load_ignore_rules(repo_root: &Path) -> Result<Gitignore, anyhow::Error> {
    let mut builder = GitignoreBuilder::new(repo_root);
    
    // Load .skyhide if it exists (QuickSky's native ignore file)
    let skyhide = repo_root.join(".skyhide");
    if skyhide.exists() {
        builder.add(&skyhide); // Ignore errors, .skyhide is optional
    }
    
    // Add default ignore patterns for QuickSky
    let default_patterns = [
        ".quicksky/",
        "target/",
        "*.tmp",
        ".DS_Store",
        "Thumbs.db"
    ];
    
    for pattern in &default_patterns {
        builder.add_line(None, pattern).map_err(|e| anyhow!("Failed to add default pattern: {}", e))?;
    }
    
    Ok(builder.build()?)
}

/// Check if a file is ignored
pub fn is_ignored(ignore_rules: &Gitignore, repo_root: &Path, file_path: &Path) -> bool {
    let rel_path = file_path.strip_prefix(repo_root).unwrap_or(file_path);
    ignore_rules.matched(rel_path, file_path.is_dir()).is_ignore()
}