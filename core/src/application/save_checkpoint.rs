//! # Save Checkpoint Use Case
//!
//! Creates a new version (checkpoint) of the project state.
//! Implements the Reverse Delta Strategy: HEAD is always full,
//! history is stored as reverse patches.

use crate::domain::manifest::{FileType, Manifest};
use crate::domain::version::{FileState, Version};
use crate::ports::{DiffPort, EncryptionPort, HasherPort, PortResult, StoragePort};
use std::collections::HashMap;

/// Input for SaveCheckpoint use case
#[derive(Debug)]
pub struct SaveCheckpointInput {
    pub message: String,
    pub author: String,
    pub encryption_key: Option<Vec<u8>>,
}

/// Output of SaveCheckpoint use case
#[derive(Debug, serde::Serialize)]
pub struct SaveCheckpointOutput {
    pub version_id: String,
    pub files_changed: usize,
    pub files_added: usize,
    pub files_deleted: usize,
}

/// Represents a detected file change
#[derive(Debug)]
pub enum FileChange {
    Added {
        path: String,
        hash: String,
    },
    Modified {
        path: String,
        old_hash: String,
        new_hash: String,
    },
    Deleted {
        path: String,
    },
}

/// SaveCheckpoint Use Case
/// 
/// Generic over port implementations for testability.
pub struct SaveCheckpointUseCase<S, D, H, E> 
where
    S: StoragePort,
    D: DiffPort,
    H: HasherPort,
    E: EncryptionPort,
{
    storage: S,
    diff: D,
    hasher: H,
    encryptor: E,
}

impl<S, D, H, E> SaveCheckpointUseCase<S, D, H, E>
where
    S: StoragePort,
    D: DiffPort,
    H: HasherPort,
    E: EncryptionPort,
{
    pub fn new(storage: S, diff: D, hasher: H, encryptor: E) -> Self {
        Self { storage, diff, hasher, encryptor }
    }

    /// Execute the save checkpoint algorithm
    pub async fn execute(
        &self,
        manifest: &mut Manifest,
        input: SaveCheckpointInput,
    ) -> PortResult<SaveCheckpointOutput> {
        // Step 0: Generate version ID
        let version_id = uuid::Uuid::new_v4().to_string();
        let current_version_id = manifest.refs.get("head").cloned();

        // Step 1: Identify changed files
        let changes = self.identify_changes(manifest).await?;
        
        if changes.is_empty() {
            return Err(crate::ports::PortError::PatchFailed(
                "No changes to commit".to_string()
            ));
        }

        // Step 2: Process text files modifications (generate reverse patches for history)
        // Must run BEFORE process_full_files updates the manifest
        self.process_text_modifications(manifest, &changes, &version_id, &input.encryption_key).await?;

        // Step 3: Identify files needing full content (all added and modified files)
        self.process_full_files(manifest, &changes, &input.encryption_key).await?;

        // Step 4: Create new Version object
        let version = self.create_version(
            &version_id,
            current_version_id,
            &input,
            manifest,
            &changes,
        );

        // Step 5: Update references
        manifest.refs.insert("head".to_string(), version_id.clone());
        manifest.metadata.last_modified = chrono_now();

        // Add version to history
        manifest.version_history.push(version);

        // Count changes
        let (added, modified, deleted) = count_changes(&changes);

        Ok(SaveCheckpointOutput {
            version_id,
            files_changed: modified,
            files_added: added,
            files_deleted: deleted,
        })
    }

    /// Step 1: Identify files that changed since last checkpoint
    async fn identify_changes(&self, manifest: &Manifest) -> PortResult<Vec<FileChange>> {
        let mut changes = Vec::new();
        
        // Get current files in working directory
        let current_files = self.storage.list("content/").await?;
        let current_files_set: std::collections::HashSet<_> = 
            current_files.iter().collect();

        // Check for added/modified files
        for file_path in &current_files {
            let content = self.storage.read(&format!("content/{}", file_path)).await?;
            let current_hash = self.hasher.hash(&content);

            if let Some(file_entry) = manifest.file_map.get(file_path) {
                // File exists in manifest
                if let Some(old_hash) = &file_entry.current_hash {
                    // Check if modified
                    if old_hash != &current_hash {
                        changes.push(FileChange::Modified {
                            path: file_path.clone(),
                            old_hash: old_hash.clone(),
                            new_hash: current_hash,
                        });
                    }
                } else {
                    // First time saving this file (even if in manifest) -> Added
                    changes.push(FileChange::Added {
                        path: file_path.clone(),
                        hash: current_hash,
                    });
                }
            } else {
                // New file not in manifest
                changes.push(FileChange::Added {
                    path: file_path.clone(),
                    hash: current_hash,
                });
            }
        }

        // Check for deleted files
        for (path, _entry) in &manifest.file_map {
            if !current_files_set.contains(path) {
                changes.push(FileChange::Deleted { path: path.clone() });
            }
        }

        Ok(changes)
    }

    /// Step 3: Process ALL added and modified files as full blobs (CAS)
    /// This ensures HEAD always contains full files.
    async fn process_full_files(
        &self,
        manifest: &mut Manifest,
        changes: &[FileChange],
        encryption_key: &Option<Vec<u8>>,
    ) -> PortResult<()> {
        for change in changes {
            let (path, hash) = match change {
                FileChange::Added { path, hash } | FileChange::Modified { path, new_hash: hash, .. } => {
                    (path, hash)
                }
                _ => continue,
            };

            // Check if blob already exists (deduplication!)
            let blob_path = format!(".store/blobs/{}", hash);
            if !self.storage.exists(&blob_path).await? {
                // Read content and save new blob
                let mut content = self.storage.read(&format!("content/{}", path)).await?;
                
                if let Some(key) = encryption_key {
                    content = self.encryptor.encrypt(key, &content).await?;
                }
                
                self.storage.write(&blob_path, &content).await?;
            }
            
            // Update file entry hash and encrypted flag
            if let Some(entry) = manifest.file_map.get_mut(path) {
                entry.current_hash = Some(hash.clone());
                entry.modified = chrono_now();
                entry.encrypted = Some(encryption_key.is_some());
            }
        }

        Ok(())
    }

    /// Step 2: Generate reverse patches for text file modifications
    async fn process_text_modifications(
        &self,
        manifest: &Manifest,
        changes: &[FileChange],
        version_id: &str,
        encryption_key: &Option<Vec<u8>>,
    ) -> PortResult<()> {
        for change in changes {
            let (path, old_hash) = match change {
                FileChange::Modified { path, old_hash, .. } => {
                    let entry = manifest.file_map.get(path);
                    let is_text = entry.map(|e| matches!(e.file_type, FileType::Text)).unwrap_or(false);
                    if !is_text { continue; }
                    (path, old_hash)
                }
                _ => continue,
            };

            // Read NEW content (working copy)
            let new_content = self.storage.read(&format!("content/{}", path)).await?;
            let new_text = String::from_utf8_lossy(&new_content);

            // Read OLD content (from blob store using OLD hash)
            // HEAD always points to full blobs
            let old_blob_path = format!(".store/blobs/{}", old_hash);
            let mut old_content = self.storage.read(&old_blob_path).await?;
            
            // Decrypt OLD content if needed
            let old_entry = manifest.file_map.get(path);
            let was_encrypted = old_entry.and_then(|e| e.encrypted).unwrap_or(false);
            
            if was_encrypted {
                let key = encryption_key.as_ref()
                    .ok_or_else(|| crate::ports::PortError::EncryptionError("Key required for encrypted history".into()))?;
                 old_content = self.encryptor.decrypt(key, &old_content).await?;
            }
            
            let old_text = String::from_utf8_lossy(&old_content);

            // Compute REVERSE patch: NEW -> OLD
            let reverse_patch = self.diff.compute_diff(&new_text, &old_text);

            // Save patch to .store/deltas/{version_id}_{path_hash}.patch
            let path_hash = self.hasher.hash(path.as_bytes());
            let patch_path = format!(".store/deltas/{}_{}.patch", version_id, &path_hash[..16]);
            
            let mut patch_data = reverse_patch.into_bytes();
            if let Some(key) = encryption_key {
                patch_data = self.encryptor.encrypt(key, &patch_data).await?;
            }
            
            self.storage.write(&patch_path, &patch_data).await?;
        }

        Ok(())
    }

    /// Create Version object from changes
    fn create_version(
        &self,
        version_id: &str,
        parent_id: Option<String>,
        input: &SaveCheckpointInput,
        manifest: &Manifest,
        changes: &[FileChange],
    ) -> Version {
        // Copy parent's file states
        let mut file_states: HashMap<String, FileState> = 
            if let Some(ref parent) = parent_id {
                manifest.version_history
                    .iter()
                    .find(|v| &v.id == parent)
                    .map(|v| v.file_states.clone())
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };

        // Apply changes to file states
        for change in changes {
            match change {
                FileChange::Added { path, hash } => {
                    let entry = manifest.file_map.get(path);
                    file_states.insert(path.clone(), FileState {
                        inode_id: entry.map(|e| e.inode_id.clone()).unwrap_or_default(),
                        hash: Some(hash.clone()),
                        content_ref: None,
                        deleted: None,
                        encrypted: Some(input.encryption_key.is_some()),
                    });
                }
                FileChange::Modified { path, new_hash, .. } => {
                    let entry = manifest.file_map.get(path);
                    let state = file_states.entry(path.clone()).or_insert_with(|| FileState {
                        inode_id: entry.map(|e| e.inode_id.clone()).unwrap_or_default(),
                        hash: Some(new_hash.clone()),
                        content_ref: None,
                        deleted: None,
                        encrypted: Some(input.encryption_key.is_some()),
                    });
                    
                    state.hash = Some(new_hash.clone());
                    state.encrypted = Some(input.encryption_key.is_some());
                    
                    // For text files, add content_ref to the patch
                    let is_text = entry.map(|e| matches!(e.file_type, FileType::Text)).unwrap_or(false);
                    if is_text {
                        let path_hash = self.hasher.hash(path.as_bytes());
                        state.content_ref = Some(format!(
                            ".store/deltas/{}_{}.patch",
                            version_id, &path_hash[..16]
                        ));
                    }
                }
                FileChange::Deleted { path } => {
                    if let Some(state) = file_states.get_mut(path) {
                        state.deleted = Some(true);
                    }
                }
            }
        }

        Version {
            id: version_id.to_string(),
            parent_id,
            timestamp: chrono_now(),
            message: input.message.clone(),
            author: input.author.clone(),
            file_states,
        }
    }
}

// Helper functions
fn chrono_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn count_changes(changes: &[FileChange]) -> (usize, usize, usize) {
    let mut added = 0;
    let mut modified = 0;
    let mut deleted = 0;
    
    for change in changes {
        match change {
            FileChange::Added { .. } => added += 1,
            FileChange::Modified { .. } => modified += 1,
            FileChange::Deleted { .. } => deleted += 1,
        }
    }
    
    (added, modified, deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would use mock implementations of ports
    // See infrastructure/memory_storage.rs for in-memory adapter
}
