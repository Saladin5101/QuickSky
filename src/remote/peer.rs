use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::repo::commit::Commit;
use crate::repo::config::{PeerEntry, RepoConfig};
use crate::repo::object;

fn peer_client() -> Client {
    use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
    let mut h = HeaderMap::new();
    h.insert(USER_AGENT, HeaderValue::from_static("quicksky-peer"));
    Client::builder().default_headers(h).timeout(std::time::Duration::from_secs(10)).build().unwrap()
}

fn peer_base(peer: &PeerEntry) -> String {
    format!("http://{}", peer.addr.trim_end_matches('/'))
}

#[derive(Serialize, Deserialize)]
struct ObjectEntry {
    hash: String,
    data: Vec<u8>,
}

#[derive(Serialize)]
struct SyncPush {
    objects: Vec<ObjectEntry>,
    commit_ids: Vec<String>,
}

#[derive(Deserialize)]
struct SyncPull {
    objects: Vec<ObjectEntry>,
}

/// Push all local objects a peer doesn't have yet
pub fn push_to_peer(peer: &PeerEntry, repo_root: &Path) -> Result<()> {
    let client = peer_client();
    let base = peer_base(peer);

    // Ask peer which objects it has
    let resp = client.get(format!("{}/objects", base)).send()
        .map_err(|e| anyhow!("Cannot reach peer '{}': {}", peer.name, e))?;
    let peer_hashes: Vec<String> = resp.json().unwrap_or_default();
    let peer_set: std::collections::HashSet<String> = peer_hashes.into_iter().collect();

    let local_hashes = object::list_objects(repo_root)?;
    let mut objects = Vec::new();
    let mut commit_ids = Vec::new();

    for hash in &local_hashes {
        if !peer_set.contains(hash) {
            let path = repo_root.join(".quicksky/objects").join(&hash[..2]).join(&hash[2..]);
            let data = fs::read(&path)?;
            // Try to deserialise as commit to collect its ID
            if let Ok(raw) = object::load_commit_object(repo_root, hash) {
                if let Ok(c) = bincode::deserialize::<Commit>(&raw) {
                    commit_ids.push(c.id);
                }
            }
            objects.push(ObjectEntry { hash: hash.clone(), data });
        }
    }

    if objects.is_empty() {
        return Ok(());
    }

    let payload = SyncPush { objects, commit_ids };
    client.post(format!("{}/sync/push", base)).json(&payload).send()
        .map_err(|e| anyhow!("Sync push to '{}' failed: {}", peer.name, e))?
        .error_for_status()
        .map_err(|e| anyhow!("Peer '{}' rejected sync: {}", peer.name, e))?;

    Ok(())
}

/// Pull objects from a peer that we don't have
pub fn pull_from_peer(peer: &PeerEntry, repo_root: &Path) -> Result<usize> {
    let client = peer_client();
    let base = peer_base(peer);

    let local_hashes = object::list_objects(repo_root)?;

    let resp = client
        .post(format!("{}/sync/pull", base))
        .json(&local_hashes)
        .send()
        .map_err(|e| anyhow!("Cannot reach peer '{}': {}", peer.name, e))?
        .error_for_status()
        .map_err(|e| anyhow!("Peer '{}' rejected pull: {}", peer.name, e))?;

    let payload: SyncPull = resp.json()?;
    let count = payload.objects.len();

    for obj in payload.objects {
        let path = repo_root.join(".quicksky/objects").join(&obj.hash[..2]).join(&obj.hash[2..]);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &obj.data)?;
        }
    }

    Ok(count)
}

/// Sync with all configured peers (pull then push)
pub fn sync_all(config: &RepoConfig, repo_root: &Path) -> Result<()> {
    if config.peers.is_empty() {
        return Err(anyhow!("No peers configured. Use `sky peer add <name> <host:port>`"));
    }

    for peer in &config.peers {
        print!("  ↔ {} ({})  ", peer.name, peer.addr);

        match pull_from_peer(peer, repo_root) {
            Ok(n) => print!("pulled {} objects  ", n),
            Err(e) => { println!("pull failed: {}", e); continue; }
        }

        match push_to_peer(peer, repo_root) {
            Ok(_)  => println!("pushed ✓"),
            Err(e) => println!("push failed: {}", e),
        }
    }
    Ok(())
}
