use anyhow::{Result, anyhow};
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use crate::repo::commit::Commit;
use crate::repo::config::{RepoConfig, RemoteEntry};
use crate::repo::object;
use std::fs;

/// Parse ssh://user@host:port/repo-path  or  user@host:repo-path
fn parse_ssh_url(url: &str) -> Result<(String, String, u16, String)> {
    let url = url.strip_prefix("ssh://").unwrap_or(url);

    // user@host:port/path  or  user@host/path  or  user@host:path
    let (userhost, path) = if let Some(pos) = url.find('/') {
        (&url[..pos], &url[pos + 1..])
    } else if let Some(pos) = url.rfind(':') {
        (&url[..pos], &url[pos + 1..])
    } else {
        return Err(anyhow!("Cannot parse SSH URL: {}", url));
    };

    let (user, hostport) = if let Some(pos) = userhost.find('@') {
        (&userhost[..pos], &userhost[pos + 1..])
    } else {
        ("git", userhost)
    };

    let (host, port) = if let Some(pos) = hostport.rfind(':') {
        let port = hostport[pos + 1..].parse::<u16>().unwrap_or(22);
        (&hostport[..pos], port)
    } else {
        (hostport, 22u16)
    };

    Ok((user.to_string(), host.to_string(), port, path.to_string()))
}

fn connect(remote: &RemoteEntry, config: &RepoConfig) -> Result<Session> {
    let (user, host, port, _) = parse_ssh_url(&remote.url)?;
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| anyhow!("SSH connect to {}:{} failed: {}", host, port, e))?;

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Try key auth first, then password
    let home = std::env::var("HOME").unwrap_or_default();
    let key_paths = [
        PathBuf::from(&home).join(".ssh/id_ed25519"),
        PathBuf::from(&home).join(".ssh/id_rsa"),
    ];

    let mut authed = false;
    for key in &key_paths {
        if key.exists() {
            if sess.userauth_pubkey_file(&user, None, key, None).is_ok() {
                authed = true;
                break;
            }
        }
    }

    if !authed {
        if let Some(pass) = &config.user.token {
            sess.userauth_password(&user, pass)
                .map_err(|e| anyhow!("SSH password auth failed: {}", e))?;
        } else {
            return Err(anyhow!(
                "SSH auth failed: no key found and no token/password configured"
            ));
        }
    }

    Ok(sess)
}

/// Run a remote command over SSH and return stdout bytes
fn run_remote(sess: &Session, cmd: &str) -> Result<Vec<u8>> {
    let mut channel = sess.channel_session()?;
    channel.exec(cmd)?;
    let mut out = Vec::new();
    channel.read_to_end(&mut out)?;
    channel.wait_close()?;
    let exit = channel.exit_status()?;
    if exit != 0 {
        return Err(anyhow!("Remote command failed (exit {}): {}", exit, cmd));
    }
    Ok(out)
}

/// Send bytes to remote stdin, return stdout
fn run_remote_with_stdin(sess: &Session, cmd: &str, input: &[u8]) -> Result<Vec<u8>> {
    let mut channel = sess.channel_session()?;
    channel.exec(cmd)?;
    channel.write_all(input)?;
    channel.send_eof()?;
    let mut out = Vec::new();
    channel.read_to_end(&mut out)?;
    channel.wait_close()?;
    Ok(out)
}

#[derive(Serialize, Deserialize)]
struct SshPushPayload {
    branch: String,
    commit_ids: Vec<String>,
    objects: Vec<SshObject>,
}

#[derive(Serialize, Deserialize)]
struct SshObject {
    hash: String,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SshHas {
    commit_ids: Vec<String>,
}

pub fn push(remote: &RemoteEntry, config: &RepoConfig, branch: &str, _commit: &Commit, repo_root: &Path) -> Result<()> {
    let (_, _, _, repo_path) = parse_ssh_url(&remote.url)?;
    let sess = connect(remote, config)?;

    // Ask remote what it has
    let has_raw = run_remote(&sess, &format!("sky-server has {} {}", repo_path, branch))?;
    let has: SshHas = serde_json::from_slice(&has_raw).unwrap_or(SshHas { commit_ids: vec![] });
    let known: std::collections::HashSet<String> = has.commit_ids.into_iter().collect();

    let all_commits = Commit::load_all(repo_root)?;
    let new_commits: Vec<&Commit> = all_commits.iter().take_while(|c| !known.contains(&c.id)).collect();

    if new_commits.is_empty() {
        println!("   Remote is already up to date.");
        return Ok(());
    }

    let mut objects = Vec::new();
    let mut commit_ids = Vec::new();
    for c in new_commits.iter().rev() {
        let bytes = bincode::serialize(c)?;
        let hash = object::store_commit(repo_root, &bytes)?;
        let raw = fs::read(repo_root.join(".quicksky/objects").join(&hash[..2]).join(&hash[2..]))?;
        objects.push(SshObject { hash, data: raw });
        commit_ids.push(c.id.clone());
    }

    let payload = SshPushPayload { branch: branch.to_string(), commit_ids, objects };
    let payload_bytes = serde_json::to_vec(&payload)?;

    run_remote_with_stdin(&sess, &format!("sky-server push {}", repo_path), &payload_bytes)?;
    Ok(())
}

pub fn pull(remote: &RemoteEntry, config: &RepoConfig, branch: &str, repo_root: &Path) -> Result<()> {
    let (_, _, _, repo_path) = parse_ssh_url(&remote.url)?;
    let sess = connect(remote, config)?;

    let local_ids = object::list_objects(repo_root)?;
    let have_json = serde_json::to_vec(&local_ids)?;

    let raw = run_remote_with_stdin(
        &sess,
        &format!("sky-server pull {} {}", repo_path, branch),
        &have_json,
    )?;

    #[derive(Deserialize)]
    struct SshPullPayload {
        objects: Vec<SshObject>,
        files: Vec<crate::remote::ops::RemoteFile>,
    }

    let payload: SshPullPayload = serde_json::from_slice(&raw)?;

    for obj in &payload.objects {
        let path = repo_root.join(".quicksky/objects").join(&obj.hash[..2]).join(&obj.hash[2..]);
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &obj.data)?;
        }
    }
    for file in &payload.files {
        let local_path = repo_root.join(&file.path);
        if let Some(parent) = local_path.parent() { fs::create_dir_all(parent)?; }
        fs::write(&local_path, &file.content)?;
    }
    Ok(())
}
