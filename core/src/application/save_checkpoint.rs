//! # Save Checkpoint Use Case
//!
//! Creates a new version (checkpoint) of the project state.
//! Implements the Reverse Delta Strategy: HEAD is always full,
//! history is stored as reverse patches.

use crate::domain::manifest::{FileEntry, FileType, Manifest};
use crate::domain::version::{FileState, Version};
use crate::ports::{DiffPort, HasherPort, PortResult, StoragePort};
use std::collections::HashMap;

/// Input for SaveCheckpoint use case
#[derive(Debug)]
pub struct SaveCheckpointInput {
    pub message: String,
    pub author: String,
}

/// Output of SaveCheckpoint use case
#[derive(Debug)]
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
pub struct SaveCheckpointUseCase<S, D, H> 
where
    S: StoragePort,
    D: DiffPort,
    H: HasherPort,
{
    storage: S,
    diff: D,
    hasher: H,
}

impl<S, D, H> SaveCheckpointUseCase<S, D, H>
where
    S: StoragePort,
    D: DiffPort,
    H: HasherPort,
{
    pub fn new(storage: S, diff: D, hasher: H) -> Self {
        Self { storage, diff, hasher }
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

        // Step 2: Process text files (generate reverse patches)
        self.process_text_files(manifest, &changes, &version_id).await?;

        // Step 3: Process binary files (CAS)
        self.process_binary_files(manifest, &changes).await?;

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
                // File exists in manifest - check if modified
                if file_entry.current_hash.as_ref() != Some(&current_hash) {
                    changes.push(FileChange::Modified {
                        path: file_path.clone(),
                        old_hash: file_entry.current_hash.clone().unwrap_or_default(),
                        new_hash: current_hash,
                    });
                }
            } else {
                // New file
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

    /// Step 2: Generate reverse patches for text files
    async fn process_text_files(
        &self,
        manifest: &Manifest,
        changes: &[FileChange],
        version_id: &str,
    ) -> PortResult<()> {
        for change in changes {
            let (path, is_text) = match change {
                FileChange::Modified { path, .. } => {
                    let entry = manifest.file_map.get(path);
                    let is_text = entry.map(|e| matches!(e.file_type, FileType::Text)).unwrap_or(false);
                    (path, is_text)
                }
                _ => continue,
            };

            if !is_text {
                continue;
            }

            // Read NEW content (working copy)
            let new_content = self.storage.read(&format!("content/{}", path)).await?;
            let new_text = String::from_utf8_lossy(&new_content);

            // Read OLD content (from HEAD version)
            let old_content = self.get_file_content_from_head(manifest, path).await?;
            let old_text = String::from_utf8_lossy(&old_content);

            // Compute REVERSE patch: NEW -> OLD
            // This allows restoring the OLD version from NEW
            let reverse_patch = self.diff.compute_diff(&new_text, &old_text);

            // Save patch to .store/deltas/{version_id}_{path_hash}.patch
            let path_hash = self.hasher.hash(path.as_bytes());
            let patch_path = format!(".store/deltas/{}_{}.patch", version_id, &path_hash[..16]);
            self.storage.write(&patch_path, reverse_patch.as_bytes()).await?;
        }

        Ok(())
    }

    /// Step 3: Process binary files using Content Addressable Storage
    async fn process_binary_files(
        &self,
        manifest: &mut Manifest,
        changes: &[FileChange],
    ) -> PortResult<()> {
        for change in changes {
            let (path, hash) = match change {
                FileChange::Added { path, hash } | FileChange::Modified { path, new_hash: hash, .. } => {
                    let entry = manifest.file_map.get(path);
                    let is_binary = entry.map(|e| matches!(e.file_type, FileType::Binary)).unwrap_or(true);
                    if !is_binary { continue; }
                    (path, hash)
                }
                _ => continue,
            };

            // Check if blob already exists (deduplication!)
            let blob_path = format!(".store/blobs/{}", hash);
            if !self.storage.exists(&blob_path).await? {
                // Read content and save new blob
                let content = self.storage.read(&format!("content/{}", path)).await?;
                self.storage.write(&blob_path, &content).await?;
            }

            // Update file entry hash
            if let Some(entry) = manifest.file_map.get_mut(path) {
                entry.current_hash = Some(hash.clone());
                entry.modified = chrono_now();
            }
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
                    });
                }
                FileChange::Modified { path, new_hash, .. } => {
                    if let Some(state) = file_states.get_mut(path) {
                        state.hash = Some(new_hash.clone());
                        // For text files, add content_ref to the patch
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

    /// Helper: Get file content from HEAD version
    async fn get_file_content_from_head(
        &self,
        manifest: &Manifest,
        path: &str,
    ) -> PortResult<Vec<u8>> {
        // For HEAD, content is in /content/ directory
        self.storage.read(&format!("content/{}", path)).await
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
