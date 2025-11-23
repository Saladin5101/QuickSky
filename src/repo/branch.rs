use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, NaiveDate};
use super::commit::Commit;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BranchState {
    pub current: String,
    pub branches: Vec<String>,
    pub rebase_backup: Option<Vec<Commit>>,
}

impl BranchState {
    fn load(repo_root: &Path) -> Result<Self> {
        let branch_file = repo_root.join(".quicksky/branches.json");
        if branch_file.exists() {
            let content = fs::read_to_string(branch_file)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self {
                current: "main".to_string(),
                branches: vec!["main".to_string()],
                rebase_backup: None,
            })
        }
    }

    fn save(&self, repo_root: &Path) -> Result<()> {
        let branch_file = repo_root.join(".quicksky/branches.json");
        fs::write(branch_file, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Create and switch to new branch
pub fn create_and_switch(repo_root: &Path, branch_name: &str) -> Result<()> {
    let mut state = BranchState::load(repo_root)?;
    
    if state.branches.contains(&branch_name.to_string()) {
        return Err(anyhow!("Branch '{}' already exists", branch_name));
    }
    
    state.branches.push(branch_name.to_string());
    state.current = branch_name.to_string();
    state.save(repo_root)?;
    
    // Create branch-specific commit directory
    let branch_dir = repo_root.join(format!(".quicksky/branches/{}", branch_name));
    fs::create_dir_all(branch_dir)?;
    
    Ok(())
}

/// Switch to existing branch
pub fn switch(repo_root: &Path, branch_name: &str) -> Result<()> {
    let mut state = BranchState::load(repo_root)?;
    
    if !state.branches.contains(&branch_name.to_string()) {
        return Err(anyhow!("Branch '{}' does not exist", branch_name));
    }
    
    state.current = branch_name.to_string();
    state.save(repo_root)?;
    Ok(())
}

/// Delete branch
pub fn delete(repo_root: &Path, branch_name: &str) -> Result<()> {
    let mut state = BranchState::load(repo_root)?;
    
    if state.current == branch_name {
        return Err(anyhow!("Cannot delete current branch '{}'", branch_name));
    }
    
    state.branches.retain(|b| b != branch_name);
    state.save(repo_root)?;
    
    // Remove branch directory
    let branch_dir = repo_root.join(format!(".quicksky/branches/{}", branch_name));
    if branch_dir.exists() {
        fs::remove_dir_all(branch_dir)?;
    }
    
    Ok(())
}

/// Get current branch
pub fn get_current(repo_root: &Path) -> Result<String> {
    let state = BranchState::load(repo_root)?;
    Ok(state.current)
}

/// List all branches
pub fn list_all(repo_root: &Path) -> Result<Vec<String>> {
    let state = BranchState::load(repo_root)?;
    Ok(state.branches)
}

/// Rebase all local changes
pub fn rebase_all(repo_root: &Path) -> Result<()> {
    let mut state = BranchState::load(repo_root)?;
    let commits = Commit::load_all(repo_root)?;
    
    // Backup current commits
    state.rebase_backup = Some(commits.clone());
    state.save(repo_root)?;
    
    // Clear current commits and re-apply in order
    let commit_dir = repo_root.join(".quicksky/commits");
    if commit_dir.exists() {
        fs::remove_dir_all(&commit_dir)?;
        fs::create_dir_all(&commit_dir)?;
    }
    
    // Re-apply commits in chronological order
    let mut sorted_commits = commits;
    sorted_commits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    for commit in sorted_commits {
        let commit_path = commit_dir.join(format!("{}.bin", commit.id));
        fs::write(commit_path, bincode::serialize(&commit)?)?;
    }
    
    Ok(())
}

/// Rebase commits within date range
pub fn rebase_date_range(repo_root: &Path, start_date: &str, end_date: &str) -> Result<()> {
    let mut state = BranchState::load(repo_root)?;
    let commits = Commit::load_all(repo_root)?;
    
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?;
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?;
    
    // Backup current commits
    state.rebase_backup = Some(commits.clone());
    state.save(repo_root)?;
    
    // Filter commits by date range
    let mut filtered_commits = Vec::new();
    let mut other_commits = Vec::new();
    
    for commit in commits {
        let commit_date = DateTime::parse_from_str(&format!("{} +0000", commit.timestamp), "%Y-%m-%d %H:%M:%S %z")?;
        let commit_naive = commit_date.naive_local().date();
        
        if commit_naive >= start && commit_naive <= end {
            filtered_commits.push(commit);
        } else {
            other_commits.push(commit);
        }
    }
    
    // Clear and rebuild commits
    let commit_dir = repo_root.join(".quicksky/commits");
    if commit_dir.exists() {
        fs::remove_dir_all(&commit_dir)?;
        fs::create_dir_all(&commit_dir)?;
    }
    
    // Re-apply other commits first, then filtered commits
    other_commits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    filtered_commits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    for commit in other_commits.into_iter().chain(filtered_commits) {
        let commit_path = commit_dir.join(format!("{}.bin", commit.id));
        fs::write(commit_path, bincode::serialize(&commit)?)?;
    }
    
    Ok(())
}

/// Undo last rebase operation
pub fn undo_rebase(repo_root: &Path) -> Result<()> {
    let mut state = BranchState::load(repo_root)?;
    
    if let Some(backup_commits) = state.rebase_backup.take() {
        // Clear current commits
        let commit_dir = repo_root.join(".quicksky/commits");
        if commit_dir.exists() {
            fs::remove_dir_all(&commit_dir)?;
            fs::create_dir_all(&commit_dir)?;
        }
        
        // Restore backup commits
        for commit in backup_commits {
            let commit_path = commit_dir.join(format!("{}.bin", commit.id));
            fs::write(commit_path, bincode::serialize(&commit)?)?;
        }
        
        state.save(repo_root)?;
        Ok(())
    } else {
        Err(anyhow!("No rebase backup found"))
    }
}