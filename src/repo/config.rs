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
pub struct UserConfig {
    pub name: String,
    pub token: Option<String>,
}

/// A single remote entry (HTTP or SSH URL)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteEntry {
    pub name: String,
    pub url: String,
}

/// Legacy single-remote field — kept for backwards compat, mapped to remotes[0]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteConfig {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BranchConfig {
    pub main: String,
    pub current: Option<String>,
}

/// A known P2P peer
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerEntry {
    pub name: String,
    pub addr: String, // host:port
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoConfig {
    pub user: UserConfig,
    /// Legacy single remote — still written for backwards compat
    pub remote: RemoteConfig,
    /// All remotes (includes the legacy one as first entry)
    #[serde(default)]
    pub remotes: Vec<RemoteEntry>,
    pub branch: BranchConfig,
    pub smtp: Option<SmtpConfig>,
    #[serde(default)]
    pub peers: Vec<PeerEntry>,
}

impl RepoConfig {
    pub fn new(
        name: String,
        token: Option<String>,
        remote_url: String,
        main_branch: String,
        smtp: Option<SmtpConfig>,
    ) -> Self {
        let origin = RemoteEntry { name: "origin".into(), url: remote_url.clone() };
        Self {
            user: UserConfig { name, token },
            remote: RemoteConfig { url: remote_url, name: "origin".into() },
            remotes: vec![origin],
            branch: BranchConfig { main: main_branch.clone(), current: Some(main_branch) },
            smtp,
            peers: vec![],
        }
    }

    /// Find a remote by name
    pub fn get_remote(&self, name: &str) -> Option<&RemoteEntry> {
        self.remotes.iter().find(|r| r.name == name)
    }

    /// Default remote (first in list, or legacy)
    pub fn default_remote(&self) -> RemoteEntry {
        self.remotes.first().cloned().unwrap_or(RemoteEntry {
            name: self.remote.name.clone(),
            url: self.remote.url.clone(),
        })
    }

    pub fn add_remote(&mut self, name: String, url: String) -> anyhow::Result<()> {
        if self.remotes.iter().any(|r| r.name == name) {
            return Err(anyhow::anyhow!("Remote '{}' already exists", name));
        }
        self.remotes.push(RemoteEntry { name, url });
        Ok(())
    }

    pub fn remove_remote(&mut self, name: &str) -> anyhow::Result<()> {
        let before = self.remotes.len();
        self.remotes.retain(|r| r.name != name);
        if self.remotes.len() == before {
            return Err(anyhow::anyhow!("Remote '{}' not found", name));
        }
        Ok(())
    }

    pub fn add_peer(&mut self, name: String, addr: String) -> anyhow::Result<()> {
        if self.peers.iter().any(|p| p.name == name) {
            return Err(anyhow::anyhow!("Peer '{}' already exists", name));
        }
        self.peers.push(PeerEntry { name, addr });
        Ok(())
    }

    pub fn remove_peer(&mut self, name: &str) -> anyhow::Result<()> {
        let before = self.peers.len();
        self.peers.retain(|p| p.name != name);
        if self.peers.len() == before {
            return Err(anyhow::anyhow!("Peer '{}' not found", name));
        }
        Ok(())
    }

    pub fn load(repo_root: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(repo_root.join(".quicksky/config.toml"))?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, repo_root: &Path) -> anyhow::Result<()> {
        let dir = repo_root.join(".quicksky");
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("config.toml"), toml::to_string_pretty(self)?)?;
        Ok(())
    }
}
