use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

/// Reference manager for branches and tags
pub struct RefManager {
    refs_dir: PathBuf,
}

impl RefManager {
    /// Create a new ref manager at `.sky/refs/`
    pub fn new(repo_root: &Path) -> Result<Self> {
        let refs_dir = repo_root.join(".sky/refs");
        fs::create_dir_all(refs_dir.join("heads"))?;
        fs::create_dir_all(refs_dir.join("remotes"))?;
        fs::create_dir_all(refs_dir.join("tags"))?;
        Ok(Self { refs_dir })
    }

    /// Create or update a branch pointer to a commit hash
    pub fn write_branch(&self, branch_name: &str, commit_hash: &str) -> Result<()> {
        let branch_path = self.refs_dir.join("heads").join(branch_name);
        fs::write(branch_path, commit_hash)?;
        Ok(())
    }

    /// Read the commit hash for a branch
    pub fn read_branch(&self, branch_name: &str) -> Result<String> {
        let branch_path = self.refs_dir.join("heads").join(branch_name);
        
        if !branch_path.exists() {
            return Err(anyhow!("Branch not found: {}", branch_name));
        }

        Ok(fs::read_to_string(branch_path)?.trim().to_string())
    }

    /// Check if a branch exists
    pub fn branch_exists(&self, branch_name: &str) -> bool {
        self.refs_dir.join("heads").join(branch_name).exists()
    }

    /// List all local branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let heads_dir = self.refs_dir.join("heads");
        if !heads_dir.exists() {
            return Ok(Vec::new());
        }

        let mut branches = Vec::new();
        for entry in fs::read_dir(heads_dir)? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                if let Some(name) = entry.file_name().into_string().ok() {
                    branches.push(name);
                }
            }
        }

        Ok(branches)
    }

    /// Delete a branch
    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        let branch_path = self.refs_dir.join("heads").join(branch_name);
        if !branch_path.exists() {
            return Err(anyhow!("Branch not found: {}", branch_name));
        }
        fs::remove_file(branch_path)?;
        Ok(())
    }

    /// Write a remote tracking branch: refs/remotes/{remote}/{branch}
    pub fn write_remote_branch(&self, remote_name: &str, branch_name: &str, commit_hash: &str) -> Result<()> {
        let remote_dir = self.refs_dir.join("remotes").join(remote_name);
        fs::create_dir_all(&remote_dir)?;
        let branch_path = remote_dir.join(branch_name);
        fs::write(branch_path, commit_hash)?;
        Ok(())
    }

    /// Read a remote tracking branch
    pub fn read_remote_branch(&self, remote_name: &str, branch_name: &str) -> Result<String> {
        let branch_path = self.refs_dir.join("remotes").join(remote_name).join(branch_name);
        
        if !branch_path.exists() {
            return Err(anyhow!("Remote branch not found: {}/{}", remote_name, branch_name));
        }

        Ok(fs::read_to_string(branch_path)?.trim().to_string())
    }

    /// List branches from a remote
    pub fn list_remote_branches(&self, remote_name: &str) -> Result<Vec<String>> {
        let remote_dir = self.refs_dir.join("remotes").join(remote_name);
        if !remote_dir.exists() {
            return Ok(Vec::new());
        }

        let mut branches = Vec::new();
        for entry in fs::read_dir(remote_dir)? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                if let Some(name) = entry.file_name().into_string().ok() {
                    branches.push(name);
                }
            }
        }

        Ok(branches)
    }

    /// Write a tag
    pub fn write_tag(&self, tag_name: &str, commit_hash: &str) -> Result<()> {
        let tag_path = self.refs_dir.join("tags").join(tag_name);
        fs::write(tag_path, commit_hash)?;
        Ok(())
    }

    /// Read a tag
    pub fn read_tag(&self, tag_name: &str) -> Result<String> {
        let tag_path = self.refs_dir.join("tags").join(tag_name);
        
        if !tag_path.exists() {
            return Err(anyhow!("Tag not found: {}", tag_name));
        }

        Ok(fs::read_to_string(tag_path)?.trim().to_string())
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let tags_dir = self.refs_dir.join("tags");
        if !tags_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tags = Vec::new();
        for entry in fs::read_dir(tags_dir)? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                if let Some(name) = entry.file_name().into_string().ok() {
                    tags.push(name);
                }
            }
        }

        Ok(tags)
    }

    /// Delete a tag
    pub fn delete_tag(&self, tag_name: &str) -> Result<()> {
        let tag_path = self.refs_dir.join("tags").join(tag_name);
        if !tag_path.exists() {
            return Err(anyhow!("Tag not found: {}", tag_name));
        }
        fs::remove_file(tag_path)?;
        Ok(())
    }

    /// Set HEAD pointer to a branch or commit
    pub fn set_head(&self, target: &str) -> Result<()> {
        let head_path = self.refs_dir.parent().unwrap().join("HEAD");
        fs::write(head_path, target)?;
        Ok(())
    }

    /// Get current HEAD pointer
    pub fn get_head(&self) -> Result<String> {
        let head_path = self.refs_dir.parent().unwrap().join("HEAD");
        
        if !head_path.exists() {
            return Ok("refs/heads/main".to_string());
        }

        Ok(fs::read_to_string(head_path)?.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_branch_refs() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let refs = RefManager::new(temp_dir.path())?;

        refs.write_branch("main", "abc123")?;
        assert!(refs.branch_exists("main"));

        let hash = refs.read_branch("main")?;
        assert_eq!(hash, "abc123");

        Ok(())
    }

    #[test]
    fn test_list_branches() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let refs = RefManager::new(temp_dir.path())?;

        refs.write_branch("main", "abc123")?;
        refs.write_branch("develop", "def456")?;

        let branches = refs.list_branches()?;
        assert_eq!(branches.len(), 2);

        Ok(())
    }

    #[test]
    fn test_remote_branches() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let refs = RefManager::new(temp_dir.path())?;

        refs.write_remote_branch("origin", "main", "abc123")?;
        let hash = refs.read_remote_branch("origin", "main")?;
        assert_eq!(hash, "abc123");

        Ok(())
    }

    #[test]
    fn test_tags() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let refs = RefManager::new(temp_dir.path())?;

        refs.write_tag("v1.0.0", "abc123")?;
        let hash = refs.read_tag("v1.0.0")?;
        assert_eq!(hash, "abc123");

        Ok(())
    }
}
