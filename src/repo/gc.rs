use anyhow::Result;
use crate::repo::objects::ObjectStore;
use std::path::Path;

/// Garbage collector for object database
pub struct GarbageCollector {
    object_store: ObjectStore,
}

impl GarbageCollector {
    pub fn new(repo_root: &Path) -> Result<Self> {
        let object_store = ObjectStore::new(repo_root)?;
        Ok(Self { object_store })
    }

    /// Run garbage collection and return stats
    pub fn run(&self) -> Result<GcStats> {
        let objects_before = self.object_store.list_objects()?;
        let size_before = self.object_store.total_size()?;

        // In Phase 1, we do basic verification
        // In Phase 2+, we'll add reachability analysis to remove unreferenced objects
        let (verified, corrupt) = self.object_store.verify()?;

        let objects_after = self.object_store.list_objects()?;
        let size_after = self.object_store.total_size()?;

        Ok(GcStats {
            objects_before: objects_before.len(),
            objects_after: objects_after.len(),
            objects_removed: objects_before.len() - objects_after.len(),
            size_before,
            size_after,
            bytes_freed: size_before.saturating_sub(size_after),
            verified_objects: verified,
            corrupt_objects: corrupt,
        })
    }

    /// Repack objects for better compression (future enhancement)
    pub fn repack(&self) -> Result<RepackStats> {
        // This will be implemented in Phase 2
        // For now, just return basic stats
        let object_count = self.object_store.list_objects()?.len();
        let total_size = self.object_store.total_size()?;

        Ok(RepackStats {
            objects: object_count,
            original_size: total_size,
            repacked_size: total_size, // No change in Phase 1
            compression_ratio: 1.0,
        })
    }
}

/// Garbage collection statistics
#[derive(Debug, Clone)]
pub struct GcStats {
    pub objects_before: usize,
    pub objects_after: usize,
    pub objects_removed: usize,
    pub size_before: u64,
    pub size_after: u64,
    pub bytes_freed: u64,
    pub verified_objects: usize,
    pub corrupt_objects: usize,
}

/// Repack statistics
#[derive(Debug, Clone)]
pub struct RepackStats {
    pub objects: usize,
    pub original_size: u64,
    pub repacked_size: u64,
    pub compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_gc_run() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let gc = GarbageCollector::new(temp_dir.path())?;
        
        // Add some objects first
        let store = ObjectStore::new(temp_dir.path())?;
        store.store_blob(b"test1")?;
        store.store_blob(b"test2")?;

        let stats = gc.run()?;
        assert_eq!(stats.verified_objects, 2);

        Ok(())
    }
}
