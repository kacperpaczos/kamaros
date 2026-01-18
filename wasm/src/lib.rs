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
            content_ref: None,
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
