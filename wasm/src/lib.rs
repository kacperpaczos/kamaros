//! # Kamaros WASM Bindings
//!
//! WASM exports for kamaros-corelib.
//! Iteration 3: SaveCheckpoint integration with JsStorageAdapter.

use wasm_bindgen::prelude::*;
use kamaros_corelib::domain::manifest::{Manifest, ProjectMetadata, FileEntry, FileType};
use kamaros_corelib::domain::version::{Version, FileState};
use std::collections::HashMap;

mod js_adapter;
pub use js_adapter::JsStorageAdapter;

// Better panic messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Get library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Simple test function
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from Kamaros WASM, {}!", name)
}

/// Create empty manifest
#[wasm_bindgen]
pub fn create_empty_manifest(project_name: &str) -> Result<JsValue, JsValue> {
    let manifest = Manifest {
        format_version: "1.0.0".to_string(),
        metadata: ProjectMetadata {
            name: project_name.to_string(),
            description: None,
            created: current_timestamp(),
            last_modified: current_timestamp(),
            author: None,
        },
        file_map: HashMap::new(),
        version_history: vec![],
        refs: HashMap::from([("head".to_string(), "".to_string())]),
        rename_log: vec![],
    };
    
    serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Parse manifest from JavaScript object
#[wasm_bindgen]
pub fn parse_manifest(js_manifest: JsValue) -> Result<JsValue, JsValue> {
    let manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    // Return back to verify round-trip
    serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get manifest info (project name, version count)
#[wasm_bindgen]
pub fn get_manifest_info(js_manifest: JsValue) -> Result<JsValue, JsValue> {
    let manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    // Return simple info object
    let info = js_sys::Object::new();
    js_sys::Reflect::set(&info, &"name".into(), &manifest.metadata.name.into())?;
    js_sys::Reflect::set(&info, &"versionCount".into(), &(manifest.version_history.len() as u32).into())?;
    js_sys::Reflect::set(&info, &"fileCount".into(), &(manifest.file_map.len() as u32).into())?;
    
    Ok(info.into())
}

/// Save checkpoint - create a new version
/// 
/// Returns updated manifest with new version in history.
/// 
/// @param js_manifest - current manifest object
/// @param js_storage - StorageAdapter with read/write/exists/list methods
/// @param message - commit message
/// @param author - author name
#[wasm_bindgen]
pub async fn save_checkpoint(
    js_manifest: JsValue,
    js_storage: JsStorageAdapter,
    message: &str,
    author: &str,
) -> Result<JsValue, JsValue> {
    let mut manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| JsValue::from_str(&format!("Parse manifest error: {}", e)))?;
    
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    
    // Generate new version ID
    let version_id = uuid::Uuid::new_v4().to_string();
    let parent_id = manifest.refs.get("head").cloned().filter(|s| !s.is_empty());
    
    // Get current files from storage
    let current_files = storage.list("content/").await
        .map_err(|e| JsValue::from_str(&e))?;
    
    // Detect changes by comparing with manifest's file_map
    let mut file_states: HashMap<String, FileState> = HashMap::new();
    let mut files_added = 0u32;
    let mut files_modified = 0u32;
    let mut files_deleted = 0u32;
    
    // Check current files
    for file_path in &current_files {
        let content = storage.read(&format!("content/{}", file_path)).await
            .map_err(|e| JsValue::from_str(&e))?;
        let hash = compute_sha256(&content);
        
        // CAS Strategy: Store content in blobs if not exists
        let blob_path = format!(".store/blobs/{}", hash);
        if !storage.exists(&blob_path).await.map_err(|e| JsValue::from_str(&e))? {
            // Write blob
            storage.write(&blob_path, &content).await
                .map_err(|e| JsValue::from_str(&e))?;
        }
        
        let inode_id = if let Some(entry) = manifest.file_map.get(file_path) {
            // Check if modified
            if entry.current_hash.as_ref() != Some(&hash) {
                files_modified += 1;
            }
            entry.inode_id.clone()
        } else {
            // New file
            files_added += 1;
            uuid::Uuid::new_v4().to_string()
        };
        
        file_states.insert(file_path.clone(), FileState {
            inode_id: inode_id.clone(),
            hash: Some(hash.clone()),
            content_ref: Some(format!("blobs/{}", hash)), // Store reference
            deleted: None,
        });
        
        // Update file_map
        let now = current_timestamp();
        manifest.file_map.entry(file_path.clone())
            .and_modify(|e| {
                e.current_hash = Some(hash.clone());
                e.modified = now.clone();
            })
            .or_insert_with(|| FileEntry {
                inode_id,
                file_type: infer_file_type(file_path),
                created: now.clone(),
                modified: now,
                current_hash: Some(hash),
            });
    }
    
    // Check for deleted files
    let current_set: std::collections::HashSet<_> = current_files.iter().collect();
    let deleted_paths: Vec<_> = manifest.file_map.keys()
        .filter(|p| !current_set.contains(p))
        .cloned()
        .collect();
    
    for path in deleted_paths {
        files_deleted += 1;
        if let Some(entry) = manifest.file_map.get(&path) {
            file_states.insert(path.clone(), FileState {
                inode_id: entry.inode_id.clone(),
                hash: None,
                content_ref: None,
                deleted: Some(true),
            });
        }
    }
    
    // Create version
    let version = Version {
        id: version_id.clone(),
        parent_id,
        timestamp: current_timestamp(),
        message: message.to_string(),
        author: author.to_string(),
        file_states,
    };
    
    // Update manifest
    manifest.version_history.push(version);
    manifest.refs.insert("head".to_string(), version_id.clone());
    manifest.metadata.last_modified = current_timestamp();
    
    // Create result object
    let result = js_sys::Object::new();
    let updated_manifest = serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    js_sys::Reflect::set(&result, &"manifest".into(), &updated_manifest)?;
    js_sys::Reflect::set(&result, &"versionId".into(), &version_id.into())?;
    js_sys::Reflect::set(&result, &"filesAdded".into(), &files_added.into())?;
    js_sys::Reflect::set(&result, &"filesModified".into(), &files_modified.into())?;
    js_sys::Reflect::set(&result, &"filesDeleted".into(), &files_deleted.into())?;
    
    Ok(result.into())
}

/// Restore version - checkout specific version
/// 
/// Restores working directory to state at version_id.
/// 
/// @param js_manifest - current manifest object
/// @param js_storage - StorageAdapter
/// @param version_id - target version ID
#[wasm_bindgen]
pub async fn restore_version(
    js_manifest: JsValue,
    js_storage: JsStorageAdapter,
    version_id: &str,
) -> Result<JsValue, JsValue> {
    let mut manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| JsValue::from_str(&format!("Parse manifest error: {}", e)))?;
    
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    
    // Find target version
    let target_version = manifest.version_history.iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| JsValue::from_str(&format!("Version {} not found", version_id)))?;
        
    let final_files = &target_version.file_states;
    let mut files_restored = 0u32;
    let mut files_deleted = 0u32;
    
    // 1. Restore files from version state
    for (path, state) in final_files {
        if state.deleted.unwrap_or(false) {
            // Ensure deleted
            if storage.exists(&format!("content/{}", path)).await.map_err(|e| JsValue::from_str(&e))? {
                storage.delete(&format!("content/{}", path)).await.map_err(|e| JsValue::from_str(&e))?;
                files_deleted += 1;
            }
            continue;
        }
        
        // Restore content
        // Try content_ref first, then hash
        let blob_path = if let Some(ref r) = state.content_ref {
             format!(".store/{}", r)
        } else if let Some(ref h) = state.hash {
             format!(".store/blobs/{}", h)
        } else {
            // No content ref or hash? Skip or error?
            // Existing file might be kept if no change, but here we assume full restore
             continue; 
        };
        
        if storage.exists(&blob_path).await.map_err(|e| JsValue::from_str(&e))? {
             let content = storage.read(&blob_path).await.map_err(|e| JsValue::from_str(&e))?;
             storage.write(&format!("content/{}", path), &content).await.map_err(|e| JsValue::from_str(&e))?;
             files_restored += 1;
        } else {
             return Err(JsValue::from_str(&format!("Missing blob for file {}: {}", path, blob_path)));
        }
    }
    
    // 2. Cleanup: Delete files in working dir that are NOT in target version
    // (This matches 'git checkout' behavior of removing untracked/extra files if we want exact state match,
    // actually git checkout usually keeps untracked files, but restores tracked ones.
    // Kamaros Core RestoreVersion deletes files that exist in HEAD but not in Target.
    // Here we should probably check what's in current directory vs target.)
    
    let current_files = storage.list("content/").await.map_err(|e| JsValue::from_str(&e))?;
    for file in current_files {
        if !final_files.contains_key(&file) {
             // File exists in workspace but not in target version
             // Is it tracked? If it was in Manifest.file_map (HEAD), it should be deleted.
             // If it's untracked, maybe keep it?
             // Core logic: "Delete files that exist in HEAD but not in Target"
             if manifest.file_map.contains_key(&file) {
                 storage.delete(&format!("content/{}", file)).await.map_err(|e| JsValue::from_str(&e))?;
                 files_deleted += 1;
             }
        }
    }
    
    // Update refs
    manifest.refs.insert("head".to_string(), version_id.to_string());
    
    // Return result
    let result = js_sys::Object::new();
    let updated_manifest = serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
    js_sys::Reflect::set(&result, &"manifest".into(), &updated_manifest)?;
    js_sys::Reflect::set(&result, &"restoredVersionId".into(), &version_id.into())?;
    js_sys::Reflect::set(&result, &"filesRestored".into(), &files_restored.into())?;
    js_sys::Reflect::set(&result, &"filesDeleted".into(), &files_deleted.into())?;
    
    Ok(result.into())
}


// Helper: Get current timestamp using JS Date API
fn current_timestamp() -> String {
    let date = js_sys::Date::new_0();
    date.to_iso_string().as_string().unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

// Helper: Compute SHA-256 hash (simplified version for WASM)
fn compute_sha256(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// Helper: Infer file type from extension
fn infer_file_type(path: &str) -> FileType {
    let text_extensions = [".txt", ".md", ".json", ".js", ".ts", ".css", ".html", ".xml", ".yaml", ".yml", ".rs", ".py"];
    if text_extensions.iter().any(|ext| path.to_lowercase().ends_with(ext)) {
        FileType::Text
    } else {
        FileType::Binary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello from Kamaros WASM, World!");
    }
    
    #[wasm_bindgen_test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
