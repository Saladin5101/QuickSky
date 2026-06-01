# Phase 1 Implementation: Object Storage

## Overview

Phase 1 introduces the **foundation for decentralization** by implementing a content-addressed object storage system similar to Git. This is the critical first step that enables all future phases.

## What Changed

### 1. New Modules

#### `src/repo/objects.rs` - Object Store
Implements content-addressed storage for commits, trees, and blobs:
- **SHA-256 hashing** for content addressing
- **Compression** with zstd for efficient storage
- **Object types**: Commit, Tree, Blob
- **API**: `store_commit()`, `read_commit()`, `store_blob()`, `read_blob()`, `verify()`, `list_objects()`

Example:
```rust
let store = ObjectStore::new(repo_root)?;
let hash = store.store_blob(b"file content")?;
let content = store.read_blob(&hash)?;
```

**Benefits:**
- ✅ Incremental sync (only transfer missing objects)
- ✅ Deduplication (same content = same hash)
- ✅ Integrity verification (verify all objects match their hashes)

#### `src/repo/refs.rs` - Reference Manager
Manages branch pointers, tags, and HEAD:
- **Branch pointers**: `refs/heads/{branch}` → commit hash
- **Remote tracking**: `refs/remotes/{remote}/{branch}` → commit hash
- **Tags**: `refs/tags/{tag}` → commit hash
- **API**: `write_branch()`, `read_branch()`, `list_branches()`, `write_tag()`, `read_tag()`

Example:
```rust
let refs = RefManager::new(repo_root)?;
refs.write_branch("main", "abc123def...")?;
let commit_hash = refs.read_branch("main")?;
```

**Benefits:**
- ✅ Branch tracking without storing full histories
- ✅ Support for remote-tracking branches (know what peers have)
- ✅ Fast branch switching (just update a pointer)

#### `src/repo/gc.rs` - Garbage Collection
Cleans up and optimizes the object database:
- **Verify integrity**: Check all objects match their hashes
- **Future phases**: Reachability analysis, object repacking
- **API**: `run()`, `repack()`, `GcStats`

Example:
```bash
sky gc --verbose --verify
```

Output:
```
📊 Garbage Collection Results:
  Objects before: 150
  Objects after: 150
  Objects removed: 0
  Size before: 2560000 bytes
  Size after: 2560000 bytes
  Bytes freed: 0 bytes
  Verified objects: 150
  Corrupt objects: 0
✅ Garbage collection complete!
```

### 2. Updated Dependencies (Cargo.toml)

```toml
sha2 = "0.10"      # SHA-256 hashing
zstd = "0.13"      # Compression
tempfile = "3.8"   # Testing support
```

### 3. New CLI Commands

#### `sky gc` - Garbage Collection
```bash
# Run garbage collection with statistics
sky gc

# Verbose output
sky gc --verbose

# Verify object integrity
sky gc --verify

# Full diagnostics
sky gc --verbose --verify
```

#### `sky remote` - Remote Management (Phase 2 placeholder)
```bash
# Placeholder for Phase 2
sky remote list
sky remote add origin /path/to/repo
sky remote remove origin
```

### 4. File System Structure

New directories created on `sky init`:
```
.sky/
├── objects/              # NEW: Content-addressed objects
│   ├── ab/              # 2-char prefix directory
│   │   └── cdef123...   # Remaining hash (compressed)
│   ├── cd/
│   │   └── ef456...
│   └── ...
├── refs/
│   ├── heads/           # Branch pointers
│   │   ├── main
│   │   └── develop
│   ├── remotes/         # NEW: Remote tracking branches
│   │   ├── origin/
│   │   │   ├── main
│   │   │   └── develop
│   │   └── peer1/
│   │       └── main
│   └── tags/            # Tags
│       ├── v1.0.0
│       └── v1.1.0
└── HEAD                 # Current branch pointer
```

## Architecture Diagram

```
┌─────────────────────────────────────────────┐
│          Reference Manager (refs/)          │
│  - Tracks branch pointers (main, develop)   │
│  - Tracks remote branches (origin/main)     │
│  - Manages tags (v1.0.0)                   │
└────────────────┬────────────────────────────┘
                 │
                 ▼ Points to
┌─────────────────────────────────────────────┐
│        Object Store (.sky/objects/)         │
│  - Content-addressed blobs                  │
│  - Commit objects                           │
│  - Tree objects                             │
│  - Compressed with zstd                     │
└─────────────────────────────────────────────┘
```

## How It Works

### Example: Creating a Commit with Objects

**Old way (Phase 0):**
```
1. Create commit with full file contents
2. Store entire commit in .quicksky/commits/
3. Push sends ALL files to server
4. Server stores files as-is
```

**New way (Phase 1):**
```
1. Create commit with file hashes (blobs)
2. Store blobs in .sky/objects/ (compressed)
3. Store commit object referencing blob hashes
4. Push sends only commit object + missing blobs
5. Peer reconstructs files from blobs
```

### Benefits of This Architecture

| Aspect | Old | New |
|--------|-----|-----|
| Storage | Full files per commit | Deduplicated blobs |
| Transfer | All files each time | Only new objects |
| Disk usage | Grows linearly | Grows with unique content |
| Verify integrity | Manual hash checks | `sky gc --verify` |
| Incremental sync | ❌ Not supported | ✅ Supported (Phase 2) |
| Deduplication | ❌ None | ✅ Automatic |

## Testing

All modules include comprehensive tests:

```bash
# Run tests
cargo test --lib repo::objects
cargo test --lib repo::refs
cargo test --lib repo::gc

# Run all Phase 1 tests
cargo test --lib repo::
```

Example test:
```bash
$ cargo test --lib repo::objects::tests
running 3 tests
test repo::objects::tests::test_object_store_blob ... ok
test repo::objects::tests::test_object_store_commit ... ok
test repo::objects::tests::test_list_objects ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

## Migration Path

### For Existing Repositories

**Option 1: Fresh start** (recommended for new work)
```bash
sky init                    # Creates new repo with Phase 1
```

**Option 2: Migrate existing** (to be implemented in Phase 2)
```bash
# Future command
sky migrate --from-v1.0 --to-phase-1
```

Current version still works as before - Phase 1 is foundational.

## Performance Impact

### Storage Efficiency
- Compression ratio: ~2-3x for text files
- Deduplication: Identical files stored once
- Example: 100MB of similar source code → ~30-50MB

### Query Performance
- Branch switch: O(1) - just update HEAD pointer
- Commit lookup: O(1) - direct hash lookup
- List all commits: O(n) - where n = number of unique objects

## Known Limitations (Phase 1)

- ⚠️ Commit objects still stored in old `.quicksky/commits/` format (to be migrated in Phase 2)
- ⚠️ No incremental push/pull yet (Phase 2)
- ⚠️ No remote-tracking branches in use yet (Phase 2)
- ⚠️ Garbage collection only verifies, doesn't remove unused objects (Phase 2+)

## Next: Phase 2 - Multiple Remotes

Phase 2 will build on Phase 1's object storage to enable:
- Multiple remotes support (Phase 2)
- Incremental push/pull using object database
- Remote-tracking branches
- Migration of existing commits to object storage

## Running Phase 1

```bash
# Build and test
cargo build --release
cargo test --lib repo::

# Initialize a repo (automatically uses Phase 1)
sky init

# Try garbage collection
sky gc --verbose --verify
```

