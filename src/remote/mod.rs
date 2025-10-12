use anyhow::{Result, anyhow};
use serde::Serialize;
use reqwest::blocking::Client;
use crate::repo::commit::Commit;
use crate::repo::config::RemoteConfig;

#[derive(Serialize)]
struct PushRequest {
    remote_name: String,
    branch: String,
    commit: Commit,
}

/// Push to a remote repository
pub fn push(remote: &RemoteConfig, branch: &str, commit: &Commit) -> Result<(), anyhow::Error> {
    let client = Client::new();
    let url = format!("{}/push", remote.url.trim_end_matches(".git")); // Simplified URL handling

    let response = client
        .post(&url)
        .json(&PushRequest {
            remote_name: remote.name.clone(),
            branch: branch.to_string(),
            commit: commit.clone(),
        })
        .send()?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Push failed: {}", response.text()?))
    }
}