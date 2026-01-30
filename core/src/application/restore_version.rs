//! # Restore Version Use Case
//!
//! Restores the working directory to a specific version from history.
//! Uses reverse delta strategy: applies patches backwards from HEAD.

use crate::domain::manifest::{FileType, Manifest};
use crate::ports::{DiffPort, EncryptionPort, PortResult, StoragePort, PortError};
use std::collections::{HashMap, HashSet, VecDeque};

/// Input for RestoreVersion use case
#[derive(Debug)]
pub struct RestoreVersionInput {
    pub target_version_id: String,
    pub force: bool, // Skip dirty check
    pub encryption_key: Option<Vec<u8>>,
}

/// Output of RestoreVersion use case
#[derive(Debug, serde::Serialize)]
pub struct RestoreVersionOutput {
    pub restored_version_id: String,
    pub files_restored: usize,
    pub patches_applied: usize,
}

/// RestoreVersion Use Case
pub struct RestoreVersionUseCase<S, D, E>
where
    S: StoragePort,
    D: DiffPort,
    E: EncryptionPort,
{
    storage: S,
    diff: D,
    encryptor: E,
}

impl<S, D, E> RestoreVersionUseCase<S, D, E>
where
    S: StoragePort,
    D: DiffPort,
    E: EncryptionPort,
{
    pub fn new(storage: S, diff: D, encryptor: E) -> Self {
        Self { storage, diff, encryptor }
    }

    /// Execute the restore version algorithm
    pub async fn execute(
        &self,
        manifest: &mut Manifest,
        input: RestoreVersionInput,
    ) -> PortResult<RestoreVersionOutput> {
        let head_id = manifest.refs.get("head").cloned()
            .ok_or_else(|| PortError::NotFound("HEAD not found".to_string()))?;
        
        let target_id = &input.target_version_id;

        // Step 1: Verify no dirty state (unless force)
        if !input.force {
            // In real implementation, compare working copy with HEAD
            // For now, we skip this check
        }

        // Step 2: Find version path from HEAD to Target
        let path = self.find_version_path(manifest, &head_id, target_id)?;
        
        if path.is_empty() {
            return Err(PortError::NotFound(
                format!("No path from {} to {}", head_id, target_id)
            ));
        }

        // Step 3: Get target version's file states
        let target_version = manifest.version_history
            .iter()
            .find(|v| &v.id == target_id)
            .ok_or_else(|| PortError::NotFound(format!("Version {} not found", target_id)))?;

        let head_version = manifest.version_history
            .iter()
            .find(|v| v.id == head_id)
            .cloned();

        // Step 4: Calculate differences and restore
        let mut files_restored = 0;
        let mut patches_applied = 0;

        // Determine which files need to be modified/added/deleted
        let target_files = &target_version.file_states;
        let head_files = head_version.as_ref()
            .map(|v| &v.file_states)
            .cloned()
            .unwrap_or_default();

        // Files to restore
        for (path, target_state) in target_files {
            if target_state.deleted.unwrap_or(false) {
                // File should not exist in target version
                if self.storage.exists(&format!("content/{}", path)).await? {
                    self.storage.delete(&format!("content/{}", path)).await?;
                    files_restored += 1;
                }
                continue;
            }

            // Restore file content
            let file_entry = manifest.file_map.get(path);
            let is_text = file_entry.map(|e| matches!(e.file_type, FileType::Text)).unwrap_or(false);

            if is_text {
                // Try reading from blob first (Fast Path / Full History enabled)
                if let Some(hash) = &target_state.hash {
                    let mut blob_content = self.storage.read(&format!(".store/blobs/{}", hash)).await?;
                    
                    if target_state.encrypted.unwrap_or(false) {
                         let key = input.encryption_key.as_ref()
                            .ok_or_else(|| PortError::EncryptionError("Key required for encrypted content".into()))?;
                         blob_content = self.encryptor.decrypt(key, &blob_content).await?;
                    }
                    
                    self.storage.write(&format!("content/{}", path), &blob_content).await?;
                    files_restored += 1;
                } else {
                     // Fallback to patches if no blob (Legacy/Optimization)
                     let content = self.restore_text_file(manifest, path, &path, target_id, &input.encryption_key).await?;
                     self.storage.write(&format!("content/{}", path), content.as_bytes()).await?;
                     patches_applied += 1;
                }
            } else {
                // For binary files: fetch from CAS
                if let Some(hash) = &target_state.hash {
                    let mut blob_content = self.storage.read(&format!(".store/blobs/{}", hash)).await?;
                    
                    if target_state.encrypted.unwrap_or(false) {
                        let key = input.encryption_key.as_ref()
                            .ok_or_else(|| PortError::EncryptionError("Key required for encrypted content".into()))?;
                        blob_content = self.encryptor.decrypt(key, &blob_content).await?;
                    }
                    
                    self.storage.write(&format!("content/{}", path), &blob_content).await?;
                }
            }
            files_restored += 1;
        }

        // Delete files that exist in HEAD but not in Target
        for (path, _) in &head_files {
            if !target_files.contains_key(path) {
                if self.storage.exists(&format!("content/{}", path)).await? {
                    self.storage.delete(&format!("content/{}", path)).await?;
                    files_restored += 1;
                }
            }
        }

        // Step 5: Update HEAD reference and file entry flags in manifest
        manifest.refs.insert("head".to_string(), target_id.clone());
        for (path, state) in &target_version.file_states {
            if let Some(entry) = manifest.file_map.get_mut(path) {
                entry.current_hash = state.hash.clone();
                entry.encrypted = state.encrypted;
            }
        }

        Ok(RestoreVersionOutput {
            restored_version_id: target_id.clone(),
            files_restored,
            patches_applied,
        })
    }

    /// Find path between two versions in the DAG (BFS)
    fn find_version_path(
        &self,
        manifest: &Manifest,
        from_id: &str,
        to_id: &str,
    ) -> PortResult<Vec<String>> {
        // Build version graph
        let versions: HashMap<_, _> = manifest.version_history
            .iter()
            .map(|v| (v.id.clone(), v))
            .collect();

        // BFS from 'from' backwards to find path to 'to'
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        queue.push_back(from_id.to_string());
        visited.insert(from_id.to_string());

        while let Some(current_id) = queue.pop_front() {
            if current_id == to_id {
                // Found! Reconstruct path
                let mut path = vec![current_id.clone()];
                let mut curr = current_id;
                while let Some(parent) = parent_map.get(&curr) {
                    path.push(parent.clone());
                    curr = parent.clone();
                }
                path.reverse();
                return Ok(path);
            }

            if let Some(version) = versions.get(&current_id) {
                // Go to parent
                if let Some(ref parent_id) = version.parent_id {
                    if !visited.contains(parent_id) {
                        visited.insert(parent_id.clone());
                        parent_map.insert(parent_id.clone(), current_id.clone());
                        queue.push_back(parent_id.clone());
                    }
                }
            }
        }

        // Also try going forward (for branching scenarios - future)
        Ok(vec![])
    }

    /// Restore text file content by reconstructing from patches
    async fn restore_text_file(
        &self,
        manifest: &Manifest,
        file_path: &str,
        _current_path: &str,
        target_version_id: &str,
        encryption_key: &Option<Vec<u8>>,
    ) -> PortResult<String> {
        // Collect patch chain: Target -> Parent -> ... -> Base
        let mut patches = Vec::new();
        let mut current_id = Some(target_version_id.to_string());

        while let Some(id) = current_id {
            let version = manifest.version_history.iter().find(|v| v.id == id);
            
            if let Some(version) = version {
                if let Some(file_state) = version.file_states.get(file_path) {
                    // Check if file is deleted in this version (breaks chain)
                    if file_state.deleted.unwrap_or(false) {
                        break;
                    }

                    if let Some(ref patch_ref) = file_state.content_ref {
                        // Store patch info
                        patches.push((patch_ref.clone(), file_state.encrypted.unwrap_or(false)));
                    } else {
                        // If no content_ref (and not deleted), maybe it's full content or glitch?
                        // In SaveCheckpoint text files always get content_ref (patch).
                        // If it's binary treated as text?
                        // Assume patch is mandatory for text.
                    }
                    
                    // Move to parent
                    current_id = version.parent_id.clone();
                } else {
                    // File not in this version (added in child?)
                    // Stop chain
                    break;
                }
            } else {
                break;
            }
        }

        // Apply patches from Base (end of list) to Target (start of list)
        let mut content = String::new();
        
        for (patch_ref, encrypted) in patches.iter().rev() {
            if self.storage.exists(patch_ref).await? {
                let mut patch_data = self.storage.read(patch_ref).await?;
                
                if *encrypted {
                    let key = encryption_key.as_ref()
                        .ok_or_else(|| PortError::EncryptionError("Key required for encrypted patch".into()))?;
                    patch_data = self.encryptor.decrypt(key, &patch_data).await?;
                }
                
                let patch_str = String::from_utf8_lossy(&patch_data);
                content = self.diff.apply_patch(&content, &patch_str)?;
            }
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests would use mock implementations
}
