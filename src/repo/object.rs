use anyhow::{Result, anyhow};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};

/// Root of the object store: .quicksky/objects/
fn objects_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".quicksky/objects")
}

/// Compute SHA-256 of raw bytes, return hex string
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Object path: .quicksky/objects/<first2>/<rest>
fn object_path(repo_root: &Path, hash: &str) -> PathBuf {
    objects_dir(repo_root).join(&hash[..2]).join(&hash[2..])
}

/// Write an object. Returns its hash. Idempotent — skips write if already exists.
pub fn write_object(repo_root: &Path, data: &[u8]) -> Result<String> {
    let hash = hash_bytes(data);
    let path = object_path(repo_root, &hash);
    if path.exists() {
        return Ok(hash);
    }
    fs::create_dir_all(path.parent().unwrap())?;
    let compressed = zstd::encode_all(data, 3)?;
    fs::write(&path, compressed)?;
    Ok(hash)
}

/// Read an object by hash.
pub fn read_object(repo_root: &Path, hash: &str) -> Result<Vec<u8>> {
    let path = object_path(repo_root, hash);
    if !path.exists() {
        return Err(anyhow!("Object not found: {}", hash));
    }
    let compressed = fs::read(&path)?;
    Ok(zstd::decode_all(compressed.as_slice())?)
}

/// Check if an object exists locally.
#[allow(dead_code)]
pub fn has_object(repo_root: &Path, hash: &str) -> bool {
    object_path(repo_root, hash).exists()
}

/// List all object hashes in the store.
pub fn list_objects(repo_root: &Path) -> Result<Vec<String>> {
    let dir = objects_dir(repo_root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut hashes = Vec::new();
    for prefix_entry in fs::read_dir(&dir)? {
        let prefix_entry = prefix_entry?;
        if !prefix_entry.path().is_dir() { continue; }
        let prefix = prefix_entry.file_name().to_string_lossy().to_string();
        for obj_entry in fs::read_dir(prefix_entry.path())? {
            let obj_entry = obj_entry?;
            let suffix = obj_entry.file_name().to_string_lossy().to_string();
            hashes.push(format!("{}{}", prefix, suffix));
        }
    }
    Ok(hashes)
}

/// Store a commit into the object store. Returns the object hash.
pub fn store_commit(repo_root: &Path, commit_bytes: &[u8]) -> Result<String> {
    write_object(repo_root, commit_bytes)
}

/// Load a commit from the object store by its object hash.
pub fn load_commit_object(repo_root: &Path, hash: &str) -> Result<Vec<u8>> {
    read_object(repo_root, hash)
}
