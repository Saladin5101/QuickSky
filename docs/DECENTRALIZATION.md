# QuickSky Decentralization: Git-Like P2P Architecture

## Current Problem

QuickSky currently requires a **central HTTP server** - it sends all commits and files to one location. This creates:
- Single point of failure
- Dependency on server uptime
- No direct peer-to-peer syncing
- Impossible to work offline and sync later

## Git's Model (What We Need)

```
Clone A                 Clone B                 Clone C
  |                       |                       |
  +------- P2P Sync -------+------- P2P Sync -------+
  
Each clone = complete repository with full history
Can sync directly with any other clone
Optional central backup (GitHub) for convenience
```

## Key Changes

### 1. Object Storage (Like Git)
Instead of sending all files, store commits as **immutable objects**:
- Each commit = unique SHA-256 hash
- Only send/pull objects that don't exist locally
- Incremental syncing by default

**Current (wasteful):**
```
Push: Send all files in commit
Pull: Download entire file snapshot
```

**New (efficient):**
```
Push: Send only new commit object (contains delta/full files)
Pull: Ask peer "what commits do you have?", download only missing ones
```

### 2. Multiple Remotes
Support adding multiple peers instead of one central server:
```bash
sky remote add peer1 ssh://user@host1/repo.git
sky remote add peer2 ssh://user@host2/repo.git
sky remote add github https://github.com/user/repo.git

sky push peer1        # Push to peer1 only
sky push --all        # Push to all remotes
sky pull peer2        # Pull from peer2
```

### 3. Fetch-Before-Push
Like Git, first fetch changes before pushing (prevents conflicts):
```bash
sky upload              # Internally:
                        # 1. Fetch latest from all peers
                        # 2. Merge if needed
                        # 3. Push new commits
```

### 4. SSH/Local Transport
Support multiple transport methods:
- **Local**: `file:///path/to/repo/.sky` (same filesystem)
- **SSH**: `ssh://user@host/path/to/repo/.sky`
- **HTTP**: `https://server.com/repo.git` (optional fallback)
- **P2P**: Direct TCP connections between peers

### 5. Ref Management (Branches/Tags)
Treat branch pointers like Git does:
- `.sky/refs/heads/main` = contains commit ID
- `.sky/refs/remotes/peer1/main` = what we know peer1 has
- `.sky/refs/tags/v1.0` = tag pointer

## Implementation Plan

### Phase 1: Object Storage (v1.1)
- [ ] Refactor commits to be stored as objects (content-addressed)
- [ ] Add object database: `.sky/objects/`
- [ ] Implement `sky gc` (garbage collection)

### Phase 2: Multiple Remotes (v1.1)
- [ ] Replace single `remote.url` with `[[remotes]]` config
- [ ] Add `sky remote add/remove/list` commands
- [ ] Update `push/pull` to handle multiple remotes

### Phase 3: SSH Transport (v1.2)
- [ ] Add SSH client support (via `ssh2` crate)
- [ ] Implement SSH remote operations
- [ ] Support SSH key authentication

### Phase 4: Smart Sync (v1.2)
- [ ] Implement commit discovery (list what we have)
- [ ] Implement incremental pull (only download missing)
- [ ] Add conflict detection before push

### Phase 5: Optional - P2P Discovery (v2.0)
- [ ] Local network peer discovery (mDNS)
- [ ] Direct peer connections for faster sync
- [ ] Fallback to SSH if P2P unavailable

## Configuration Example

**Old (centralized):**
```toml
[remote]
url = "https://central-server.com/my-repo"
token = "pat_xxx"
```

**New (decentralized):**
```toml
[[remotes]]
name = "origin"
url = "file:///home/user/projects/my-repo"
fetch = true
push = true

[[remotes]]
name = "peer1"
url = "ssh://alice@192.168.1.100:22/home/alice/repos/my-repo"
fetch = true
push = true

[[remotes]]
name = "backup"
url = "https://github.com/user/my-repo.git"
fetch = true
push = true
```

## File Structure Changes

```
.sky/
├── config              # Repo config (now with [[remotes]])
├── HEAD                # Current branch pointer
├── objects/            # NEW: Commit objects (content-addressed)
│   ├── abc123def...    # Commit objects (compressed)
│   └── ...
├── refs/
│   ├── heads/
│   │   ├── main
│   │   └── feature-x
│   ├── remotes/        # NEW: Track what each remote has
│   │   ├── origin/main
│   │   ├── peer1/main
│   │   └── backup/main
│   └── tags/
└── logs/               # Reflog for undo operations
```

## Backward Compatibility

- Old single-remote configs still work (auto-converted to new format)
- `sky init` asks for remote URL but doesn't require it
- Works completely offline - no sync needed until explicit `push/pull`
- Eventually phase out HTTP-only in favor of SSH/local

## Commands Update

```bash
# New/updated commands
sky remote add <name> <url>       # Add remote
sky remote list                   # List all remotes
sky remote remove <name>          # Remove remote
sky fetch [remote]                # Fetch from remote(s)
sky push [remote]                 # Push to specific remote (or all)
sky pull [remote]                 # Pull from specific remote
sky gc                            # Compress objects, clean up
sky status                        # Show what's unpushed/unpulled

# Existing commands (now work offline)
sky upload [message]              # Local commit only
sky upload --push [message]       # Commit + push to all remotes
sky log                           # Show local commits
sky branch                        # Manage branches (local only)
```

## Benefits

✅ **Fully decentralized** - works without any server  
✅ **Efficient** - only transfer what's needed  
✅ **Flexible** - choose which remotes to sync with  
✅ **Offline-first** - work offline, sync whenever  
✅ **Git-compatible** - similar mental model to Git  
✅ **Fallback support** - can still use central servers optionally  
