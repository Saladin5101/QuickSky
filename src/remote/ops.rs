use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::repo::change::FileStatus;
use crate::repo::commit::Commit;
use crate::repo::config::RepoConfig;

/// A file entry sent over the wire: relative path + raw bytes
#[derive(Serialize, Deserialize)]
pub(crate) struct RemoteFile {
    path: PathBuf,
    content: Vec<u8>,
}

/// Push payload: commit metadata + full file contents for added/modified files
#[derive(Serialize)]
struct PushPayload<'a> {
    branch: &'a str,
    commit_id: &'a str,
    author: &'a str,
    timestamp: &'a str,
    message: &'a str,
    /// Deleted file paths
    deleted: Vec<PathBuf>,
    /// Added/modified files with content
    files: Vec<RemoteFile>,
}

/// Pull response: snapshot of all files on the remote branch
#[derive(Deserialize)]
pub struct PullPayload {
    pub files: Vec<RemoteFile>,
}

fn build_client(token: Option<&str>) -> Client {
    use reqwest::header::{self, HeaderMap, HeaderValue};
    let mut headers = HeaderMap::new();
    if let Some(t) = token {
        if let Ok(v) = HeaderValue::from_str(&format!("Bearer {}", t)) {
            headers.insert(header::AUTHORIZATION, v);
        }
    }
    headers.insert(header::USER_AGENT, HeaderValue::from_static("quicksky-cli"));
    Client::builder().default_headers(headers).build().unwrap()
}

/// Push a commit to a QuickSky-compatible remote server.
///
/// Remote URL format: `http(s)://host[:port]/repo-name`
/// Endpoint used: `POST {url}/push`
pub fn push(config: &RepoConfig, branch: &str, commit: &Commit, repo_root: &Path) -> Result<()> {
    let client = build_client(config.user.token.as_deref());

    let mut deleted = Vec::new();
    let mut files = Vec::new();

    for (rel_path, status) in &commit.changes {
        match status {
            FileStatus::Deleted => deleted.push(rel_path.clone()),
            FileStatus::Added | FileStatus::Modified => {
                let full = repo_root.join(rel_path);
                let content = fs::read(&full)
                    .map_err(|e| anyhow!("Cannot read {}: {}", full.display(), e))?;
                files.push(RemoteFile { path: rel_path.clone(), content });
            }
        }
    }

    let payload = PushPayload {
        branch,
        commit_id: &commit.id,
        author: &commit.author,
        timestamp: &commit.timestamp,
        message: &commit.message,
        deleted,
        files,
    };

    let url = format!("{}/push", config.remote.url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| anyhow!("Push request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Push rejected by remote ({}): {}", resp.status(), resp.text()?));
    }
    Ok(())
}

/// Pull the latest file snapshot from a QuickSky-compatible remote server.
///
/// Endpoint used: `GET {url}/pull?branch={branch}`
/// Writes all received files into `repo_root`.
pub fn pull(config: &RepoConfig, branch: &str, repo_root: &Path) -> Result<()> {
    let client = build_client(config.user.token.as_deref());

    let url = format!("{}/pull", config.remote.url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .query(&[("branch", branch)])
        .send()
        .map_err(|e| anyhow!("Pull request failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow!("Remote rejected pull: {}", e))?;

    let payload: PullPayload = resp.json()?;

    for file in payload.files {
        let local_path = repo_root.join(&file.path);
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&local_path, &file.content)?;
    }
    Ok(())
}

/// Fetch only the list of remote commit IDs (lightweight check before full pull).
///
/// Endpoint used: `GET {url}/commits?branch={branch}`
#[allow(dead_code)]
pub fn fetch_commit_ids(config: &RepoConfig, branch: &str) -> Result<Vec<String>> {
    let client = build_client(config.user.token.as_deref());
    let url = format!("{}/commits", config.remote.url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .query(&[("branch", branch)])
        .send()
        .map_err(|e| anyhow!("Fetch failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow!("Remote rejected fetch: {}", e))?;

    let ids: Vec<String> = resp.json()?;
    Ok(ids)
}

/// Check whether the remote is reachable and the repo exists.
///
/// Endpoint used: `GET {url}/ping`
#[allow(dead_code)]
pub fn ping(config: &RepoConfig) -> Result<()> {
    let client = build_client(config.user.token.as_deref());
    let url = format!("{}/ping", config.remote.url.trim_end_matches('/'));
    client
        .get(&url)
        .send()
        .map_err(|e| anyhow!("Cannot reach remote '{}': {}", config.remote.url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("Remote ping failed: {}", e))?;
    Ok(())
}

/// Resolve conflicts: map of path -> chosen side ("local" | "remote")
#[derive(Serialize)]
#[allow(dead_code)]
pub struct ConflictResolution {
    pub branch: String,
    pub choices: HashMap<PathBuf, String>,
}
