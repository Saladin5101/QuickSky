use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use crate::repo::change::FileStatus;
use crate::repo::commit::Commit;
use crate::repo::config::{RepoConfig, RemoteEntry};
use crate::repo::object;

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct RemoteFile {
    pub path: PathBuf,
    pub content: Vec<u8>,
}

/// Incremental push: only send commits the remote doesn't have yet
#[derive(Serialize)]
#[allow(dead_code)]
struct PushPayload {
    branch: String,
    /// New commit objects (bincode-serialised, zstd-compressed — already stored in object DB)
    objects: Vec<ObjectEntry>,
    /// Ordered list of commit IDs being pushed (newest last)
    commit_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ObjectEntry {
    pub hash: String,
    pub data: Vec<u8>, // raw compressed bytes straight from object store
}

/// Remote tells us which commit IDs it already has
#[derive(Deserialize)]
struct RemoteHas {
    commit_ids: Vec<String>,
}

/// Pull response: objects the client asked for
#[derive(Deserialize)]
struct PullPayload {
    objects: Vec<ObjectEntry>,
    files: Vec<RemoteFile>,
}

// ── HTTP client ───────────────────────────────────────────────────────────────

fn http_client(token: Option<&str>) -> Client {
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

// ── Transport dispatch ────────────────────────────────────────────────────────

fn is_ssh(url: &str) -> bool {
    url.starts_with("ssh://") || url.starts_with("git@")
}

// ── Push ──────────────────────────────────────────────────────────────────────

/// Push to a named remote (defaults to "origin")
pub fn push(config: &RepoConfig, branch: &str, commit: &Commit, repo_root: &Path) -> Result<()> {
    let remote = config.default_remote();
    push_to(&remote, config, branch, commit, repo_root)
}

pub fn push_to(remote: &RemoteEntry, config: &RepoConfig, branch: &str, commit: &Commit, repo_root: &Path) -> Result<()> {
    if is_ssh(&remote.url) {
        return crate::remote::ssh::push(remote, config, branch, commit, repo_root);
    }
    push_http(remote, config, branch, commit, repo_root)
}

fn push_http(remote: &RemoteEntry, config: &RepoConfig, branch: &str, _commit: &Commit, repo_root: &Path) -> Result<()> {
    let client = http_client(config.user.token.as_deref());
    let base = remote.url.trim_end_matches('/');

    // 1. Ask remote which commits it already has
    let has: RemoteHas = client
        .get(format!("{}/commits?branch={}", base, branch))
        .send()
        .map_err(|e| anyhow!("Cannot reach remote: {}", e))?
        .json()
        .unwrap_or(RemoteHas { commit_ids: vec![] });

    let known: std::collections::HashSet<String> = has.commit_ids.into_iter().collect();

    // 2. Collect only new commits (walk back from HEAD until we hit a known one)
    let all_commits = Commit::load_all(repo_root)?;
    let new_commits: Vec<&Commit> = all_commits
        .iter()
        .take_while(|c| !known.contains(&c.id))
        .collect();

    if new_commits.is_empty() {
        println!("   Remote is already up to date.");
        return Ok(());
    }

    // 3. Store each new commit in object DB and collect objects to send
    let mut objects = Vec::new();
    let mut commit_ids = Vec::new();

    for c in new_commits.iter().rev() {
        let bytes = bincode::serialize(c)?;
        let hash = object::store_commit(repo_root, &bytes)?;
        let raw = fs::read(repo_root.join(".quicksky/objects")
            .join(&hash[..2]).join(&hash[2..]))?;
        objects.push(ObjectEntry { hash, data: raw });
        commit_ids.push(c.id.clone());
    }

    // 4. Also send file contents for the tip commit
    let tip = new_commits[0]; // newest
    let mut files = Vec::new();
    for (rel_path, status) in &tip.changes {
        if matches!(status, FileStatus::Added | FileStatus::Modified) {
            let full = repo_root.join(rel_path);
            if let Ok(content) = fs::read(&full) {
                files.push(RemoteFile { path: rel_path.clone(), content });
            }
        }
    }

    // 5. Send
    let payload = serde_json::json!({
        "branch": branch,
        "objects": objects,
        "commit_ids": commit_ids,
        "files": files,
    });

    let resp = client.post(format!("{}/push", base)).json(&payload).send()
        .map_err(|e| anyhow!("Push failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Remote rejected push ({}): {}", resp.status(), resp.text()?));
    }
    Ok(())
}

// ── Pull ──────────────────────────────────────────────────────────────────────

pub fn pull(config: &RepoConfig, branch: &str, repo_root: &Path) -> Result<()> {
    let remote = config.default_remote();
    pull_from(&remote, config, branch, repo_root)
}

pub fn pull_from(remote: &RemoteEntry, config: &RepoConfig, branch: &str, repo_root: &Path) -> Result<()> {
    if is_ssh(&remote.url) {
        return crate::remote::ssh::pull(remote, config, branch, repo_root);
    }
    pull_http(remote, config, branch, repo_root)
}

fn pull_http(remote: &RemoteEntry, config: &RepoConfig, branch: &str, repo_root: &Path) -> Result<()> {
    let client = http_client(config.user.token.as_deref());
    let base = remote.url.trim_end_matches('/');

    // 1. Tell remote which commits we already have
    let local_ids = object::list_objects(repo_root)?;
    let resp = client
        .post(format!("{}/pull", base))
        .json(&serde_json::json!({ "branch": branch, "have": local_ids }))
        .send()
        .map_err(|e| anyhow!("Pull failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow!("Remote rejected pull: {}", e))?;

    let payload: PullPayload = resp.json()?;

    // 2. Store received objects
    for obj in &payload.objects {
        let path = repo_root.join(".quicksky/objects")
            .join(&obj.hash[..2]).join(&obj.hash[2..]);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &obj.data)?;
        }
    }

    // 3. Write files to working directory
    for file in &payload.files {
        let local_path = repo_root.join(&file.path);
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&local_path, &file.content)?;
    }

    Ok(())
}

// ── Misc ──────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn fetch_commit_ids(config: &RepoConfig, branch: &str) -> Result<Vec<String>> {
    let client = http_client(config.user.token.as_deref());
    let base = config.default_remote().url;
    let resp = client
        .get(format!("{}/commits?branch={}", base.trim_end_matches('/'), branch))
        .send()
        .map_err(|e| anyhow!("Fetch failed: {}", e))?
        .error_for_status()?;
    let has: RemoteHas = resp.json()?;
    Ok(has.commit_ids)
}

#[allow(dead_code)]
pub fn ping(config: &RepoConfig) -> Result<()> {
    let client = http_client(config.user.token.as_deref());
    let base = config.default_remote().url;
    client
        .get(format!("{}/ping", base.trim_end_matches('/')))
        .send()
        .map_err(|e| anyhow!("Cannot reach remote: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow!("Ping failed: {}", e))?;
    Ok(())
}
