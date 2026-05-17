use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub from: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoConfig {
    pub user: UserConfig,
    pub remote: RemoteConfig,
    pub branch: BranchConfig,
    pub smtp: Option<SmtpConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub name: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteConfig {
    pub url: String,
    pub name: String, // Remote name (e.g., origin)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BranchConfig {
    pub main: String, // Main branch name
    pub current: Option<String>, // Current branch name
}

impl RepoConfig {
    /// Create a new configuration
    pub fn new(name: String, token: Option<String>, remote_url: String, main_branch: String, smtp: Option<SmtpConfig>) -> Self {
        Self {
            user: UserConfig { name, token },
            remote: RemoteConfig { url: remote_url, name: "origin".into() },
            branch: BranchConfig { main: main_branch.clone(), current: Some(main_branch) },
            smtp,
        }
    }

    /// Load configuration from a file
    pub fn load(repo_root: &Path) -> Result<Self, anyhow::Error> {
        let config_path = repo_root.join(".quicksky/config.toml");
        let content = fs::read_to_string(config_path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save configuration to a file
    pub fn save(&self, repo_root: &Path) -> Result<(), anyhow::Error> {
        let config_dir = repo_root.join(".quicksky");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}