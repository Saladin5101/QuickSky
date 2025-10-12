# QuickSky: Lazy Developer-Friendly **Native** Version Control Tool  
A no-bullshit, standalone version control tool—built from scratch (not a Git wrapper!) to fix the stuff that makes version control feel like a chore. No more memorizing convoluted commands, hopping between directories, or fighting legacy workflows. Designed for developers who just want to write code, not their tools.  


## What QuickSky Solves  
Version control shouldn’t require a cheat sheet. QuickSky cuts through the noise with **native, simplified workflows**—no dependency on existing tools, just what you actually use:  
- No multi-step "stage → commit → push" (one command handles it all)  
- No manual `cd` to switch repos (tell it the repo name, it takes you there)  
- Rebasing without cryptic SHA chasing (use date ranges or "rebase all"—no jargon)  
- Conflict resolution that skips messy markers (just pick "remote" or "local"—done)  


## Key Features  
| Feature | What It Does |
|---------|--------------|
| One-Click Upload | `sky upload` natively handles changes tracking, committing, and remote sync—add a comment if you want, no pressure. |
| Repo 0-Hassle Switching | `sky change-to <repo-name>` jumps straight to your repo (no more `cd ~/projects/...`—QuickSky indexes repos for you). |
| Pain-Free Rebasing | Rebase by date range (`sky rebase 2024-10-01 -> 2024-10-05`) or rebase all local changes (`sky rebase --all`)—native logic, no hidden dependencies. |
| Fix Old Commits | `sky reload <commit-sha> "New message"` edits past commits without breaking your history (handles commit trees natively, no workaround). |
| Simple Branching | Create/delete/switch branches with `sky branch -a <name>`, `sky branch -d <name>`, or `sky branch <name>`—all branch logic is built in, no external tools. |


## Installation  
QuickSky ships with native packages for most platforms—no build-from-source hassle (unless you want to).  

### 1. npm Package (Cross-Platform, Node.js Required)  
For Node.js (v14+) users, install globally as a native CLI:  
```bash
npm install -g quicksky-cli
```  

### 2. DEB Package (Debian/Ubuntu/Linux Mint)  
Download the latest `.deb` from the [Releases](https://github.com/Saladin5101/QuickSky/releases) page, then install:  
```bash
sudo dpkg -i quicksky_1.0.0_amd64.deb
# Fix missing dependencies (if any)
sudo apt-get install -f
```  

### 3. macOS Packages (PKG/DMG)  
- **PKG Installer**: Download the `.pkg` from [Releases](https://github.com/Saladin5101/QuickSky/releases) and double-click to run (sets up system-wide PATH automatically).  
- **DMG (Drag-and-Drop)**: Open the `.dmg`, drag the `QuickSky` binary to `/usr/local/bin` (or your preferred PATH directory).  

### 4. Windows (MSI Only—EXE Can Wait)  
Windows EXEs are more hassle than they’re worth, but **MSI is supported**: Grab the latest `.msi` from [Releases](https://github.com/Saladin5101/QuickSky/releases) and run the installer (adds QuickSky to your Windows PATH).  

### Verify Installation  
Check if it’s working (native CLI, no Git required):  
```bash
sky --help
```  


## Quick Start  
First, initialize QuickSky for your project (it asks for basic info *once*—no repeated setup):  
```bash
# Run this in your project directory
sky init
# Follow prompts:
# 1. Your username (for commit attribution)
# 2. PAT/password (for remote repo access)
# 3. Remote repo URL + name (e.g., https://github.com/[YOUR-USERNAME]/my-repo.git origin)
# 4. Main branch name (e.g., "main"—QuickSky uses this for default workflows)
```  

Now use the commands you’ll actually need (all native, no hidden tools):  

### 1. Upload Changes  
```bash
# Upload with a custom comment
sky upload "Fix login bug + add user profile"

# Auto-generate a timestamped comment (lazy mode activated)
sky upload
```  

### 2. Switch Repos  
```bash
# No more cd! Use the repo name you set during init
sky change-to my-other-project
```  

### 3. Rebase Without Panic  
```bash
# Rebase commits from Oct 1 to Oct 5 onto your main branch
sky rebase 2024-10-01 -> 2024-10-05

# Rebase all local changes to match the remote main branch
sky rebase --all

# Messed up the rebase? Undo instantly (no cryptic "abort" commands)
sky rebase fuck-base
```  

### 4. Fix an Old Commit  
```bash
# Get the commit SHA from `sky log` (QuickSky’s native log command), then edit:
sky reload a1b2c3d "Correct function name (fixes #42)"
```  

### 5. Manage Branches  
```bash
# Create + switch to a new branch (native branch tracking)
sky branch -a feature/payment

# Switch to an existing branch
sky branch main

# Delete a branch (local + remote—no extra steps)
sky branch -d old-feature
```  

### Bonus: Check Commit History  
QuickSky has a native log command (simple, no clutter):  
```bash
sky log
# Shows commits with SHA, author, date, and message—no Git-style verbosity
```  


## License  
QuickSky is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**. This means:  
- You can use, modify, and distribute it freely (it’s open-source, after all).  
- If you run QuickSky as a service (e.g., in a cloud code space), you must share your modified source code (keeps it fair for everyone).  

See the [LICENSE](LICENSE) file for full details.  


## Notes for V1  
- QuickSky v1 is **100% native**—no wrapping, no rely on Git or other tools. It’s built from the ground up to be simple.  
- Bugs? Feature requests? Open an issue or PR. This tool’s for *your* workflow—if something feels off, say so.  
- No "best practices" lectures. If it works for how you code, that’s all that matters.  


## Author  
-- Saladin5101 in Chengdu , SC , China. UTC+0800 17:26  
[Optional: "Want an EXE for Windows? Convince me—it’s more work than it sounds."]
