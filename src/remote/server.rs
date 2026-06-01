use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tiny_http::{Method, Response, Server};
use crate::repo::commit::Commit;
use crate::repo::object;

#[derive(Serialize, Deserialize)]
struct ObjectEntry {
    hash: String,
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct PushPayload {
    branch: String,
    #[allow(dead_code)]
    commit_ids: Vec<String>,
    objects: Vec<ObjectEntry>,
    #[serde(default)]
    files: Vec<FileEntry>,
}

#[derive(Serialize, Deserialize)]
struct FileEntry {
    path: PathBuf,
    content: Vec<u8>,
}

#[derive(Deserialize)]
struct PullRequest {
    #[allow(dead_code)]
    branch: String,
    #[allow(dead_code)]
    have: Vec<String>,
}

fn read_body(req: &mut tiny_http::Request) -> Vec<u8> {
    let mut buf = Vec::new();
    req.as_reader().read_to_end(&mut buf).unwrap_or(0);
    buf
}

fn json_response(body: &impl Serialize) -> Response<std::io::Cursor<Vec<u8>>> {
    let bytes = serde_json::to_vec(body).unwrap_or_default();
    Response::from_data(bytes)
        .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap())
}

fn ok() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(b"{\"ok\":true}".to_vec())
        .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap())
}

fn err_response(msg: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(format!("{{\"error\":\"{}\"}}", msg).into_bytes())
        .with_status_code(400)
        .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap())
}

/// Start the QuickSky P2P server. Blocks until Ctrl-C.
pub fn serve(repo_root: &Path, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow!("Cannot bind to {}: {}", addr, e))?;

    println!("🌐 QuickSky server listening on {} (repo: {})", addr, repo_root.display());
    println!("   Press Ctrl-C to stop.\n");

    for mut req in server.incoming_requests() {
        let method = req.method().clone();
        let url = req.url().to_string();
        let path = url.split('?').next().unwrap_or("").to_string();

        let response = match (method, path.as_str()) {
            // Health check
            (Method::Get, "/ping") => ok(),

            // List all local object hashes
            (Method::Get, "/objects") => {
                match object::list_objects(repo_root) {
                    Ok(hashes) => json_response(&hashes),
                    Err(e) => err_response(&e.to_string()),
                }
            }

            // List commit IDs for a branch
            (Method::Get, p) if p.starts_with("/commits") => {
                let branch = url.split("branch=").nth(1).unwrap_or("main");
                match commits_for_branch(repo_root, branch) {
                    Ok(ids) => json_response(&serde_json::json!({ "commit_ids": ids })),
                    Err(e) => err_response(&e.to_string()),
                }
            }

            // Receive pushed commits + objects
            (Method::Post, "/push") => {
                let body = read_body(&mut req);
                match handle_push(repo_root, &body) {
                    Ok(_)  => ok(),
                    Err(e) => err_response(&e.to_string()),
                }
            }

            // Send objects the client doesn't have
            (Method::Post, "/pull") => {
                let body = read_body(&mut req);
                match handle_pull(repo_root, &body) {
                    Ok(resp) => json_response(&resp),
                    Err(e)   => err_response(&e.to_string()),
                }
            }

            // P2P: receive objects from a peer
            (Method::Post, "/sync/push") => {
                let body = read_body(&mut req);
                match handle_sync_push(repo_root, &body) {
                    Ok(_)  => ok(),
                    Err(e) => err_response(&e.to_string()),
                }
            }

            // P2P: send objects a peer doesn't have
            (Method::Post, "/sync/pull") => {
                let body = read_body(&mut req);
                match handle_sync_pull(repo_root, &body) {
                    Ok(resp) => json_response(&resp),
                    Err(e)   => err_response(&e.to_string()),
                }
            }

            _ => Response::from_data(b"not found".to_vec()).with_status_code(404),
        };

        let _ = req.respond(response);
    }
    Ok(())
}

fn commits_for_branch(repo_root: &Path, _branch: &str) -> Result<Vec<String>> {
    Ok(Commit::load_all(repo_root)?.into_iter().map(|c| c.id).collect())
}

fn handle_push(repo_root: &Path, body: &[u8]) -> Result<()> {
    let payload: PushPayload = serde_json::from_slice(body)?;

    // Store objects
    for obj in &payload.objects {
        let path = repo_root.join(".quicksky/objects").join(&obj.hash[..2]).join(&obj.hash[2..]);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &obj.data)?;
        }
    }

    // Write files to working directory
    for file in &payload.files {
        let local = repo_root.join(&file.path);
        if let Some(p) = local.parent() { fs::create_dir_all(p)?; }
        fs::write(&local, &file.content)?;
    }

    println!("  ← push: {} new objects on branch '{}'", payload.objects.len(), payload.branch);
    Ok(())
}

#[derive(Serialize)]
struct PullResponse {
    objects: Vec<ObjectEntry>,
    files: Vec<FileEntry>,
}

fn handle_pull(repo_root: &Path, body: &[u8]) -> Result<PullResponse> {
    let req: PullRequest = serde_json::from_slice(body)?;
    let have: std::collections::HashSet<String> = req.have.into_iter().collect();

    let all = object::list_objects(repo_root)?;
    let mut objects = Vec::new();
    for hash in all {
        if !have.contains(&hash) {
            let path = repo_root.join(".quicksky/objects").join(&hash[..2]).join(&hash[2..]);
            let data = fs::read(&path)?;
            objects.push(ObjectEntry { hash, data });
        }
    }

    // Send working-tree files for the latest commit
    let mut files = Vec::new();
    if let Ok(commits) = Commit::load_all(repo_root) {
        if let Some(tip) = commits.first() {
            for (rel_path, status) in &tip.changes {
                use crate::repo::change::FileStatus;
                if matches!(status, FileStatus::Added | FileStatus::Modified) {
                    let full = repo_root.join(rel_path);
                    if let Ok(content) = fs::read(&full) {
                        files.push(FileEntry { path: rel_path.clone(), content });
                    }
                }
            }
        }
    }

    Ok(PullResponse { objects, files })
}

fn handle_sync_push(repo_root: &Path, body: &[u8]) -> Result<()> {
    #[derive(Deserialize)]
    struct SyncPush { objects: Vec<ObjectEntry> }
    let payload: SyncPush = serde_json::from_slice(body)?;
    for obj in payload.objects {
        let path = repo_root.join(".quicksky/objects").join(&obj.hash[..2]).join(&obj.hash[2..]);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &obj.data)?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct SyncPullResponse { objects: Vec<ObjectEntry> }

fn handle_sync_pull(repo_root: &Path, body: &[u8]) -> Result<SyncPullResponse> {
    let have: Vec<String> = serde_json::from_slice(body).unwrap_or_default();
    let have_set: std::collections::HashSet<String> = have.into_iter().collect();
    let all = object::list_objects(repo_root)?;
    let mut objects = Vec::new();
    for hash in all {
        if !have_set.contains(&hash) {
            let path = repo_root.join(".quicksky/objects").join(&hash[..2]).join(&hash[2..]);
            let data = fs::read(&path)?;
            objects.push(ObjectEntry { hash, data });
        }
    }
    Ok(SyncPullResponse { objects })
}
