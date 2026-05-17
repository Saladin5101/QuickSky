use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoRegistry {
    pub repos: HashMap<String, PathBuf>,
}

impl RepoRegistry {
    fn get_registry_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
        Ok(PathBuf::from(home).join(".quicksky_registry.json"))
    }

    fn load() -> Result<Self> {
        let registry_path = Self::get_registry_path()?;
        if registry_path.exists() {
            let content = fs::read_to_string(registry_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self {
                repos: HashMap::new(),
            })
        }
    }

    fn save(&self) -> Result<()> {
        let registry_path = Self::get_registry_path()?;
        fs::write(registry_path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    fn register_current_repo() -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(".quicksky/config.toml");
        
        if !config_path.exists() {
            return Ok(()); // Not a QuickSky repo
        }

        // Extract repo name from directory name
        let repo_name = current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid directory name"))?
            .to_string();

        let mut registry = Self::load()?;
        registry.repos.insert(repo_name, current_dir);
        registry.save()?;
        
        Ok(())
    }
}

/// Find repository path by name
pub fn find_repo(repo_name: &str) -> Result<PathBuf> {
    // Auto-register current repo if it's a QuickSky repo
    let _ = RepoRegistry::register_current_repo();
    
    let registry = RepoRegistry::load()?;
    
    if let Some(path) = registry.repos.get(repo_name) {
        if path.exists() {
            Ok(path.clone())
        } else {
            Err(anyhow!("Repository '{}' path no longer exists: {}", repo_name, path.display()))
        }
    } else {
        Err(anyhow!("Repository '{}' not found in registry", repo_name))
    }
}

/// Register a repository manually
pub fn register_repo(repo_name: &str, repo_path: &Path) -> Result<()> {
    let mut registry = RepoRegistry::load()?;
    registry.repos.insert(repo_name.to_string(), repo_path.to_path_buf());
    registry.save()?;
    Ok(())
}

/// List all registered repositories
#[allow(dead_code)]
pub fn list_repos() -> Result<HashMap<String, PathBuf>> {
    let registry = RepoRegistry::load()?;
    Ok(registry.repos)
}