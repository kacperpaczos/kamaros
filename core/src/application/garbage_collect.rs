//! Garbage Collection Use Case
//!
//! Removes unreferenced blobs from storage to reclaim disk space.
//! Similar to `git gc` - identifies and prunes orphaned content.

use crate::domain::manifest::Manifest;
use crate::ports::{PortResult, StoragePort};
use std::collections::HashSet;

/// Result of garbage collection operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct GcResult {
    /// Number of blobs analyzed
    pub blobs_checked: usize,
    /// Number of unreferenced blobs deleted
    pub blobs_deleted: usize,
    /// Bytes freed
    pub bytes_freed: usize,
}

/// Garbage Collection Use Case
pub struct GcUseCase<S: StoragePort> {
    storage: S,
}

impl<S: StoragePort> GcUseCase<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
    
    /// Run garbage collection
    /// 
    /// 1. Collect all blob hashes referenced in version history
    /// 2. List all blobs in .store/blobs/
    /// 3. Delete blobs not in referenced set
    pub async fn run(&self, manifest: &Manifest) -> PortResult<GcResult> {
        // Step 1: Collect referenced hashes from all versions
        let referenced = self.collect_referenced_hashes(manifest);
        
        // Step 2: List all blobs in storage
        let all_blobs = self.storage.list(".store/blobs/").await.unwrap_or_default();
        
        let mut blobs_deleted = 0;
        let mut bytes_freed = 0;
        
        // Step 3: Delete unreferenced blobs
        for blob_name in &all_blobs {
            if !referenced.contains(blob_name) {
                let blob_path = format!(".store/blobs/{}", blob_name);
                
                // Get size before deletion for reporting
                if let Ok(size) = self.storage.size(&blob_path).await {
                    bytes_freed += size;
                }
                
                if self.storage.delete(&blob_path).await.is_ok() {
                    blobs_deleted += 1;
                }
            }
        }
        
        Ok(GcResult {
            blobs_checked: all_blobs.len(),
            blobs_deleted,
            bytes_freed,
        })
    }
    
    /// Collect all blob hashes referenced across all versions
    fn collect_referenced_hashes(&self, manifest: &Manifest) -> HashSet<String> {
        let mut referenced = HashSet::new();
        
        for version in &manifest.version_history {
            for (_path, state) in &version.file_states {
                // Add hash if present
                if let Some(ref hash) = state.hash {
                    referenced.insert(hash.clone());
                }
                
                // Also check content_ref (e.g., "blobs/abc123")
                if let Some(ref content_ref) = state.content_ref {
                    if content_ref.starts_with("blobs/") {
                        let hash = content_ref.trim_start_matches("blobs/");
                        referenced.insert(hash.to_string());
                    }
                }
            }
        }
        
        referenced
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::manifest::{Manifest, ProjectMetadata};
    use crate::domain::version::{Version, FileState};
    use crate::infrastructure::memory_storage::MemoryStorage;
    use std::collections::HashMap;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_gc_removes_unreferenced_blobs() {
        let storage = Arc::new(MemoryStorage::new());
        
        // Setup: Create blobs (2 referenced, 1 orphan)
        storage.write(".store/blobs/hash1", b"content1").await.unwrap();
        storage.write(".store/blobs/hash2", b"content2").await.unwrap();
        storage.write(".store/blobs/orphan", b"orphan").await.unwrap();
        
        // Create manifest with only hash1 and hash2 referenced
        let manifest = Manifest {
            format_version: "1.0.0".to_string(),
            metadata: ProjectMetadata {
                name: "Test".to_string(),
                description: None,
                created: "2024-01-01".to_string(),
                last_modified: "2024-01-01".to_string(),
                author: None,
            },
            file_map: HashMap::new(),
            version_history: vec![
                Version {
                    id: "v1".to_string(),
                    parent_id: None,
                    timestamp: "2024-01-01".to_string(),
                    message: "Test".to_string(),
                    author: "Test".to_string(),
                    file_states: HashMap::from([
                        ("file1.txt".to_string(), FileState {
                            inode_id: "i1".to_string(),
                            hash: Some("hash1".to_string()),
                            content_ref: Some("blobs/hash1".to_string()),
                            deleted: None,
                            encrypted: None,
                        }),
                        ("file2.txt".to_string(), FileState {
                            inode_id: "i2".to_string(),
                            hash: Some("hash2".to_string()),
                            content_ref: Some("blobs/hash2".to_string()),
                            deleted: None,
                            encrypted: None,
                        }),
                    ]),
                },
            ],
            refs: HashMap::new(),
            rename_log: vec![],
        };
        
        let gc: GcUseCase<Arc<MemoryStorage>> = GcUseCase::new(Arc::clone(&storage));
        let result: GcResult = gc.run(&manifest).await.unwrap();
        
        assert_eq!(result.blobs_deleted, 1);
        assert!(storage.exists(".store/blobs/hash1").await.unwrap());
        assert!(storage.exists(".store/blobs/hash2").await.unwrap());
        assert!(!storage.exists(".store/blobs/orphan").await.unwrap());
    }
}
