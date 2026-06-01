use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Object types in the repository
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
}

/// A serializable commit object stored in the object database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitObject {
    pub author: String,
    pub timestamp: String,
    pub message: String,
    pub tree_hash: String,           // Reference to tree object
    pub parent_hash: Option<String>, // Previous commit (or None for initial)
    pub changes: HashMap<PathBuf, String>, // file_path -> blob_hash
}

/// A tree object representing directory structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreeObject {
    pub entries: HashMap<String, String>, // filename -> blob_hash
}

/// Content-addressed object database
pub struct ObjectStore {
    objects_dir: PathBuf,
}

impl ObjectStore {
    /// Create a new object store at `.sky/objects/`
    pub fn new(repo_root: &Path) -> Result<Self> {
        let objects_dir = repo_root.join(".sky/objects");
        fs::create_dir_all(&objects_dir)?;
        Ok(Self { objects_dir })
    }

    /// Compute SHA-256 hash of data
    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Store an object and return its hash
    pub fn store_commit(&self, commit: &CommitObject) -> Result<String> {
        let data = bincode::serialize(commit)?;
        let hash = Self::compute_hash(&data);
        self.write_object(&hash, &data)?;
        Ok(hash)
    }

    /// Store a tree object and return its hash
    pub fn store_tree(&self, tree: &TreeObject) -> Result<String> {
        let data = bincode::serialize(tree)?;
        let hash = Self::compute_hash(&data);
        self.write_object(&hash, &data)?;
        Ok(hash)
    }

    /// Store raw blob data and return its hash
    pub fn store_blob(&self, data: &[u8]) -> Result<String> {
        let hash = Self::compute_hash(data);
        self.write_object(&hash, data)?;
        Ok(hash)
    }

    /// Write object to disk with compression
    fn write_object(&self, hash: &str, data: &[u8]) -> Result<()> {
        let obj_path = self.get_object_path(hash);
        
        // Create directory structure: objects/ab/cdef...
        if let Some(parent) = obj_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Skip if already exists
        if obj_path.exists() {
            return Ok(());
        }

        // Compress with zstd before storing
        let compressed = zstd::encode_all(data, 3)?;
        fs::write(&obj_path, compressed)?;
        Ok(())
    }

    /// Retrieve a commit object by hash
    pub fn read_commit(&self, hash: &str) -> Result<CommitObject> {
        let data = self.read_object(hash)?;
        Ok(bincode::deserialize(&data)?)
    }

    /// Retrieve a tree object by hash
    pub fn read_tree(&self, hash: &str) -> Result<TreeObject> {
        let data = self.read_object(hash)?;
        Ok(bincode::deserialize(&data)?)
    }

    /// Retrieve raw blob data by hash
    pub fn read_blob(&self, hash: &str) -> Result<Vec<u8>> {
        self.read_object(hash)
    }

    /// Read an object from disk
    fn read_object(&self, hash: &str) -> Result<Vec<u8>> {
        let obj_path = self.get_object_path(hash);
        
        if !obj_path.exists() {
            return Err(anyhow!("Object not found: {}", hash));
        }

        let compressed = fs::read(&obj_path)?;
        let data = zstd::decode_all(compressed.as_slice())?;
        Ok(data)
    }

    /// Check if an object exists
    pub fn has_object(&self, hash: &str) -> bool {
        self.get_object_path(hash).exists()
    }

    /// Get object file path: objects/ab/cdef...
    fn get_object_path(&self, hash: &str) -> PathBuf {
        if hash.len() < 2 {
            return self.objects_dir.join(hash);
        }
        self.objects_dir
            .join(&hash[..2])
            .join(&hash[2..])
    }

    /// List all object hashes
    pub fn list_objects(&self) -> Result<Vec<String>> {
        let mut objects = Vec::new();
        
        if !self.objects_dir.exists() {
            return Ok(objects);
        }

        for entry in fs::read_dir(&self.objects_dir)? {
            let entry = entry?;
            let dir_name = entry.file_name();
            let dir_str = dir_name.to_string_lossy();
            
            if let Ok(subdir) = fs::read_dir(entry.path()) {
                for sub_entry in subdir {
                    let sub_entry = sub_entry?;
                    let file_name = sub_entry.file_name();
                    let file_str = file_name.to_string_lossy();
                    objects.push(format!("{}{}", dir_str, file_str));
                }
            }
        }

        Ok(objects)
    }

    /// Get total object storage size (in bytes)
    pub fn total_size(&self) -> Result<u64> {
        let mut total = 0;
        for entry in fs::read_dir(&self.objects_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total += metadata.len();
            } else if metadata.is_dir() {
                for sub_entry in fs::read_dir(entry.path())? {
                    let sub_entry = sub_entry?;
                    total += sub_entry.metadata()?.len();
                }
            }
        }
        Ok(total)
    }

    /// Verify object integrity by recomputing hashes
    pub fn verify(&self) -> Result<(usize, usize)> {
        let mut verified = 0;
        let mut corrupt = 0;

        for hash in self.list_objects()? {
            if let Ok(data) = self.read_object(&hash) {
                let computed_hash = Self::compute_hash(&data);
                if computed_hash == hash {
                    verified += 1;
                } else {
                    corrupt += 1;
                }
            }
        }

        Ok((verified, corrupt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_object_store_commit() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = ObjectStore::new(temp_dir.path())?;

        let commit = CommitObject {
            author: "Test User".to_string(),
            timestamp: "2024-01-01 12:00:00".to_string(),
            message: "Initial commit".to_string(),
            tree_hash: "tree123".to_string(),
            parent_hash: None,
            changes: HashMap::new(),
        };

        let hash = store.store_commit(&commit)?;
        assert!(!hash.is_empty());
        assert!(store.has_object(&hash));

        let retrieved = store.read_commit(&hash)?;
        assert_eq!(retrieved.author, commit.author);

        Ok(())
    }

    #[test]
    fn test_object_store_blob() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = ObjectStore::new(temp_dir.path())?;

        let data = b"hello world";
        let hash = store.store_blob(data)?;

        let retrieved = store.read_blob(&hash)?;
        assert_eq!(retrieved, data);

        Ok(())
    }

    #[test]
    fn test_list_objects() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = ObjectStore::new(temp_dir.path())?;

        store.store_blob(b"data1")?;
        store.store_blob(b"data2")?;

        let objects = store.list_objects()?;
        assert_eq!(objects.len(), 2);

        Ok(())
    }
}
