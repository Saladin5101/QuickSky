use clap::Parser;
use std::io::Write;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

mod ffi {
    pub mod sys;
}
mod ignore;
mod repo {
    pub mod config;
    pub mod change;
    pub mod commit;
    pub mod branch;
    pub mod patch;
    pub mod object;
}
mod remote {
    pub mod ops;
    pub mod email;
    pub mod ssh;
    pub mod peer;
    pub mod server;
}
mod registry;

#[derive(Parser, Debug)]
#[command(
    name = "sky",
    author = "Saladin5101",
    version = "0.1.0",
    about = "Lazy developer-friendly version control tool",
    long_about = None
)]
enum SkyCmd {
    #[command(about = "Initialize QuickSky repository")]
    Init,

    #[command(about = "Commit and push to remote repository")]
    Upload {
        #[arg(default_value_t = String::new())]
        message: String,
        #[arg(long, help = "Push to a specific remote (default: origin)")]
        remote: Option<String>,
    },

    #[command(about = "View all commit records")]
    Log,

    #[command(about = "Branch management")]
    Branch {
        #[arg(short = 'a', help = "Create and switch to new branch")]
        add: Option<String>,
        #[arg(short = 'd', help = "Delete branch")]
        delete: Option<String>,
        #[arg(help = "Switch to existing branch")]
        name: Option<String>,
    },

    #[command(about = "Switch to different repository")]
    ChangeTo {
        repo_name: String,
    },

    #[command(about = "Rebase commits")]
    Rebase {
        #[arg(long)]
        all: bool,
        #[arg(help = "Date range (YYYY-MM-DD -> YYYY-MM-DD) or 'fuck-base' to undo")]
        range: Option<String>,
    },

    #[command(about = "Edit past commit message")]
    Reload {
        commit_sha: String,
        message: String,
    },

    #[command(about = "Generate patch from commit and send via email")]
    PatchSend {
        #[arg(long)]
        commit: Option<String>,
        #[arg(help = "Patch file path (optional)")]
        patch_file: Option<String>,
        #[arg(long)]
        to: String,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long, help = "SMTP host:port (overrides config)")]
        smtp: Option<String>,
        #[arg(long, help = "Sender email (overrides config)")]
        from: Option<String>,
        #[arg(long, help = "SMTP password (overrides config)")]
        pass: Option<String>,
    },

    #[command(about = "Manage remotes (add / remove / list)")]
    Remote {
        #[arg(long, help = "Add remote: --add <name> <url>", num_args = 2)]
        add: Option<Vec<String>>,
        #[arg(long, help = "Remove remote by name")]
        remove: Option<String>,
        #[arg(long, help = "List all remotes")]
        list: bool,
    },

    #[command(about = "Manage P2P peers (add / remove / list)")]
    Peer {
        #[arg(long, help = "Add peer: --add <name> <host:port>", num_args = 2)]
        add: Option<Vec<String>>,
        #[arg(long, help = "Remove peer by name")]
        remove: Option<String>,
        #[arg(long, help = "List all peers")]
        list: bool,
    },

    #[command(about = "Sync with all known P2P peers")]
    Sync,

    #[command(about = "Start QuickSky P2P server")]
    Serve {
        #[arg(long, default_value_t = 7272)]
        port: u16,
    },
}

fn main() -> anyhow::Result<()> {
    let cmd = SkyCmd::parse();
    match cmd {
        SkyCmd::Init                                                          => init()?,
        SkyCmd::Upload { message, remote }                                    => upload(message, remote)?,
        SkyCmd::Log                                                           => log()?,
        SkyCmd::Branch { add, delete, name }                                  => branch_cmd(add, delete, name)?,
        SkyCmd::ChangeTo { repo_name }                                        => change_to(repo_name)?,
        SkyCmd::Rebase { all, range }                                         => rebase_cmd(all, range)?,
        SkyCmd::Reload { commit_sha, message }                                => reload_commit(commit_sha, message)?,
        SkyCmd::PatchSend { commit, patch_file, to, subject, smtp, from, pass } => patch_send(commit, patch_file, to, subject, smtp, from, pass)?,
        SkyCmd::Remote { add, remove, list }                                  => remote_cmd(add, remove, list)?,
        SkyCmd::Peer { add, remove, list }                                    => peer_cmd(add, remove, list)?,
        SkyCmd::Sync                                                          => sync_cmd()?,
        SkyCmd::Serve { port }                                                => serve_cmd(port)?,
    }
    Ok(())
}

fn read_line(reader: &mut impl BufRead, prompt: &str) -> anyhow::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn init() -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    if repo_root.join(".quicksky/config.toml").exists() {
        return Err(anyhow::anyhow!("Repository already initialized"));
    }

    let stdin = io::stdin();
    let mut r = stdin.lock();

    let name       = read_line(&mut r, "Enter username: ")?;
    let token_raw  = read_line(&mut r, "Enter PAT/token (leave blank to skip): ")?;
    let token      = if token_raw.is_empty() { None } else { Some(token_raw) };
    let remote_url = read_line(&mut r, "Enter remote repository URL: ")?;
    let branch_raw = read_line(&mut r, "Enter main branch name (default: main): ")?;
    let branch     = if branch_raw.is_empty() { "main".into() } else { branch_raw };

    println!("\nSMTP configuration (for sky patch-send) — leave blank to skip:");
    let smtp_host = read_line(&mut r, "  SMTP host (e.g. smtp.gmail.com): ")?;
    let smtp_cfg = if smtp_host.is_empty() {
        None
    } else {
        let port_raw = read_line(&mut r, "  SMTP port (default: 587): ")?;
        let port     = port_raw.parse::<u16>().unwrap_or(587);
        let from     = read_line(&mut r, "  Sender email: ")?;
        let pass_raw = read_line(&mut r, "  SMTP password (leave blank to skip): ")?;
        let pass     = if pass_raw.is_empty() { None } else { Some(pass_raw) };
        Some(repo::config::SmtpConfig { host: smtp_host, port, from, password: pass })
    };

    let config = repo::config::RepoConfig::new(name.clone(), token, remote_url.clone(), branch.clone(), smtp_cfg);
    config.save(&repo_root)?;
    fs::create_dir_all(repo_root.join(".quicksky/commits"))?;
    fs::create_dir_all(repo_root.join(".quicksky/objects"))?;

    let repo_name = repo_root.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    registry::register_repo(&repo_name, &repo_root)?;

    println!("\n✅ Initialization successful!");
    println!("User: {} | Remote: {} | Branch: {}", name, remote_url, branch);
    println!("Token: {}", if config.user.token.is_some() { "configured" } else { "not set" });
    println!("SMTP:  {}", if config.smtp.is_some() { "configured" } else { "not set" });
    println!("Repository '{}' registered for switching", repo_name);
    Ok(())
}

fn upload(message: String, remote_name: Option<String>) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = repo::config::RepoConfig::load(&repo_root)?;

    let msg = if message.is_empty() {
        format!("Auto-commit: {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))
    } else { message };

    println!("🔍 Detecting changes...");
    let commit = repo::commit::Commit::create(&repo_root, &config, &msg)?;

    let branch = config.branch.current.as_ref().unwrap_or(&config.branch.main).clone();

    println!("📤 Pushing to remote...");
    match remote_name {
        Some(name) => {
            let remote = config.get_remote(&name)
                .ok_or_else(|| anyhow::anyhow!("Remote '{}' not found", name))?
                .clone();
            remote::ops::push_to(&remote, &config, &branch, &commit, &repo_root)?;
        }
        None => remote::ops::push(&config, &branch, &commit, &repo_root)?,
    }

    println!("\n✅ Upload successful!");
    println!("Commit ID: {}", commit.id);
    println!("Message:   {}", commit.message);
    Ok(())
}

fn log() -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let commits = repo::commit::Commit::load_all(&repo_root)?;
    if commits.is_empty() {
        return Err(anyhow::anyhow!("No commit records available"));
    }
    println!("📜 Commit history (latest first):");
    for (i, commit) in commits.iter().enumerate() {
        println!("\n[{i}] ID: {}", commit.id);
        println!("   Author:    {}", commit.author);
        println!("   Timestamp: {}", commit.timestamp);
        println!("   Message:   {}", commit.message);
        println!("   Changes:");
        for (path, status) in &commit.changes {
            let s = match status {
                repo::change::FileStatus::Added    => "Added",
                repo::change::FileStatus::Modified => "Modified",
                repo::change::FileStatus::Deleted  => "Deleted",
            };
            println!("     - {s}: {:?}", path);
        }
    }
    Ok(())
}

fn branch_cmd(add: Option<String>, delete: Option<String>, name: Option<String>) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    if let Some(b) = add {
        repo::branch::create_and_switch(&repo_root, &b)?;
        println!("✅ Created and switched to branch: {}", b);
    } else if let Some(b) = delete {
        repo::branch::delete(&repo_root, &b)?;
        println!("✅ Deleted branch: {}", b);
    } else if let Some(b) = name {
        repo::branch::switch(&repo_root, &b)?;
        println!("✅ Switched to branch: {}", b);
    } else {
        let current  = repo::branch::get_current(&repo_root)?;
        let branches = repo::branch::list_all(&repo_root)?;
        println!("Current branch: {}", current);
        println!("All branches:   {}", branches.join(", "));
    }
    Ok(())
}

fn change_to(repo_name: String) -> anyhow::Result<()> {
    let repo_path = registry::find_repo(&repo_name)?;
    std::env::set_current_dir(&repo_path)?;
    println!("✅ Switched to repository: {} at {}", repo_name, repo_path.display());
    Ok(())
}

fn rebase_cmd(all: bool, range: Option<String>) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    if let Some(range_str) = range {
        if range_str == "fuck-base" {
            repo::branch::undo_rebase(&repo_root)?;
            println!("✅ Rebase undone successfully");
        } else if range_str.contains(" -> ") {
            let dates: Vec<&str> = range_str.split(" -> ").collect();
            if dates.len() == 2 {
                repo::branch::rebase_date_range(&repo_root, dates[0], dates[1])?;
                println!("✅ Rebased commits from {} to {}", dates[0], dates[1]);
            } else {
                return Err(anyhow::anyhow!("Invalid date range. Use: YYYY-MM-DD -> YYYY-MM-DD"));
            }
        }
    } else if all {
        let config = repo::config::RepoConfig::load(&repo_root)?;
        let branch = config.branch.current.as_ref().unwrap_or(&config.branch.main).clone();
        println!("📥 Pulling from remote...");
        remote::ops::pull(&config, &branch, &repo_root)?;
        repo::branch::rebase_all(&repo_root)?;
        println!("✅ Pulled remote changes and rebased local commits");
    } else {
        return Err(anyhow::anyhow!("Please specify --all or a date range"));
    }
    Ok(())
}

fn reload_commit(commit_sha: String, message: String) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    repo::commit::edit_message(&repo_root, &commit_sha, &message)?;
    println!("✅ Updated commit {} with new message: {}", commit_sha, message);
    Ok(())
}

fn patch_send(
    commit_sha: Option<String>,
    patch_file: Option<String>,
    to: String,
    subject: Option<String>,
    smtp_override: Option<String>,
    from_override: Option<String>,
    pass_override: Option<String>,
) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = repo::config::RepoConfig::load(&repo_root)?;

    let commit = match commit_sha {
        Some(sha) => repo::commit::Commit::load_all(&repo_root)?
            .into_iter()
            .find(|c| c.id.starts_with(&sha))
            .ok_or_else(|| anyhow::anyhow!("Commit '{}' not found", sha))?,
        None => repo::patch::head_commit(&repo_root)?,
    };

    let patch_path = PathBuf::from(patch_file.unwrap_or_else(|| format!("{}.patch", &commit.id[..8])));

    if !patch_path.exists() {
        println!("📝 Generating patch...");
        repo::patch::format_patch(&repo_root, &commit, &patch_path)?;
        println!("   Written to {}", patch_path.display());
    } else {
        println!("📎 Using existing patch: {}", patch_path.display());
    }

    let mut smtp = config.smtp.clone()
        .ok_or_else(|| anyhow::anyhow!("No SMTP config. Run `sky init` or pass --smtp/--from/--pass"))?;

    if let Some(s) = smtp_override {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        smtp.host = parts[0].to_string();
        if let Some(p) = parts.get(1) { smtp.port = p.parse().unwrap_or(587); }
    }
    if let Some(f) = from_override { smtp.from = f; }
    if let Some(p) = pass_override { smtp.password = Some(p); }

    let subject = subject.unwrap_or_else(|| commit.message.clone());
    println!("📧 Sending patch to {}...", to);
    remote::email::send_patch(&smtp, &to, &subject, &patch_path)?;
    println!("✅ Patch sent successfully!");
    Ok(())
}

fn remote_cmd(add: Option<Vec<String>>, remove: Option<String>, list: bool) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let mut config = repo::config::RepoConfig::load(&repo_root)?;

    if let Some(parts) = add {
        config.add_remote(parts[0].clone(), parts[1].clone())?;
        config.save(&repo_root)?;
        println!("✅ Added remote '{}' → {}", parts[0], parts[1]);
    } else if let Some(name) = remove {
        config.remove_remote(&name)?;
        config.save(&repo_root)?;
        println!("✅ Removed remote '{}'", name);
    } else if list || (!list && add.is_none() && remove.is_none()) {
        if config.remotes.is_empty() {
            println!("No remotes configured.");
        } else {
            println!("Remotes:");
            for r in &config.remotes {
                println!("  {} → {}", r.name, r.url);
            }
        }
    }
    Ok(())
}

fn peer_cmd(add: Option<Vec<String>>, remove: Option<String>, list: bool) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let mut config = repo::config::RepoConfig::load(&repo_root)?;

    if let Some(parts) = add {
        config.add_peer(parts[0].clone(), parts[1].clone())?;
        config.save(&repo_root)?;
        println!("✅ Added peer '{}' at {}", parts[0], parts[1]);
    } else if let Some(name) = remove {
        config.remove_peer(&name)?;
        config.save(&repo_root)?;
        println!("✅ Removed peer '{}'", name);
    } else if list || (!list && add.is_none() && remove.is_none()) {
        if config.peers.is_empty() {
            println!("No peers configured.");
        } else {
            println!("Peers:");
            for p in &config.peers {
                println!("  {} → {}", p.name, p.addr);
            }
        }
    }
    Ok(())
}

fn sync_cmd() -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = repo::config::RepoConfig::load(&repo_root)?;
    println!("🔄 Syncing with {} peer(s)...", config.peers.len());
    remote::peer::sync_all(&config, &repo_root)?;
    println!("✅ Sync complete");
    Ok(())
}

fn serve_cmd(port: u16) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    remote::server::serve(&repo_root, port)
}
