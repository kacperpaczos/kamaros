//! # Kamaros WASM Bindings
//!
//! WASM exports for kamaros-corelib.
//! Iteration 3: SaveCheckpoint integration with JsStorageAdapter.

use wasm_bindgen::prelude::*;
use kamaros_corelib::domain::manifest::{Manifest, ProjectMetadata};
use std::collections::HashMap;

mod js_adapter;
pub use js_adapter::JsStorageAdapter;

mod error;
pub use error::KamarosError;

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

/// Derive key from passphrase
#[wasm_bindgen]
pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<Vec<u8>, JsValue> {
    use kamaros_corelib::infrastructure::AesGcmEncryptor;
    use kamaros_corelib::ports::EncryptionPort;
    
    let encryptor = AesGcmEncryptor::new();
    encryptor.derive_key(passphrase, salt)
        .map_err(|e| KamarosError::from(e.to_string()).to_js())
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
        .map_err(|e| KamarosError::from(e.to_string()).to_js())
}

/// Parse manifest from JavaScript object
#[wasm_bindgen]
pub fn parse_manifest(js_manifest: JsValue) -> Result<JsValue, JsValue> {
    let manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| KamarosError::from(format!("Parse error: {}", e)).to_js())?;
    
    // Return back to verify round-trip
    serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| KamarosError::from(e.to_string()).to_js())
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
    encryption_key: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    use kamaros_corelib::application::save_checkpoint::{SaveCheckpointUseCase, SaveCheckpointInput};
    use kamaros_corelib::infrastructure::{Sha256Hasher, SimpleDiff, AesGcmEncryptor};

    let mut manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| KamarosError::from(format!("Parse manifest error: {}", e)).to_js())?;
    
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    let diff = SimpleDiff::new();
    let hasher = Sha256Hasher::new();
    let encryptor = AesGcmEncryptor::new();

    let use_case = SaveCheckpointUseCase::new(storage, diff, hasher, encryptor);
    
    let input = SaveCheckpointInput {
        message: message.to_string(),
        author: author.to_string(),
        encryption_key,
    };

    let output = use_case.execute(&mut manifest, input).await
        .map_err(|e| KamarosError::from(e.to_string()).to_js())?;
    
    // Create result object
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"manifest".into(), &serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| KamarosError::from(e.to_string()).to_js())?)?;
    // Fallback: return JSON string to avoid serde_wasm_bindgen Map issues
    let manifest_json = serde_json::to_string(&manifest)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization failed: {}", e)))?;
    js_sys::Reflect::set(&result, &"manifestJson".into(), &JsValue::from_str(&manifest_json))?;

    js_sys::Reflect::set(&result, &"versionId".into(), &output.version_id.into())?;
    js_sys::Reflect::set(&result, &"filesAdded".into(), &(output.files_added as u32).into())?;
    js_sys::Reflect::set(&result, &"filesModified".into(), &(output.files_changed as u32).into())?;
    js_sys::Reflect::set(&result, &"filesDeleted".into(), &(output.files_deleted as u32).into())?;
    
    Ok(result.into())
}

/// Run Garbage Collection
#[wasm_bindgen]
pub async fn gc(js_manifest: JsValue, js_storage: JsStorageAdapter) -> Result<JsValue, JsValue> {
    let manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| KamarosError::from(format!("Parse error: {}", e)).to_js())?;
        
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    
    // Use Arc for shared ownership if needed by GC implementation detail, 
    // but here we can just pass the storage wrapper if GC takes ownership or reference.
    // Core GC takes `S: StoragePort`. JsStorageWrapper implements it.
    
    let gc = kamaros_corelib::application::garbage_collect::GcUseCase::new(storage);
    let result = gc.run(&manifest).await
        .map_err(|e| KamarosError::from(e.to_string()).to_js())?;
        
    let js_result = js_sys::Object::new();
    js_sys::Reflect::set(&js_result, &"blobsChecked".into(), &(result.blobs_checked as u32).into())?;
    js_sys::Reflect::set(&js_result, &"blobsDeleted".into(), &(result.blobs_deleted as u32).into())?;
    js_sys::Reflect::set(&js_result, &"bytesFreed".into(), &(result.bytes_freed as u32).into())?;
    
    Ok(js_result.into())
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
    encryption_key: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    use kamaros_corelib::application::restore_version::{RestoreVersionUseCase, RestoreVersionInput};
    use kamaros_corelib::infrastructure::{SimpleDiff, AesGcmEncryptor};

    let mut manifest: Manifest = serde_wasm_bindgen::from_value(js_manifest)
        .map_err(|e| KamarosError::from(format!("Parse manifest error: {}", e)).to_js())?;
    
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    let diff = SimpleDiff::new();
    let encryptor = AesGcmEncryptor::new();

    let use_case = RestoreVersionUseCase::new(storage, diff, encryptor);
    
    let input = RestoreVersionInput {
        target_version_id: version_id.to_string(),
        force: true, // WASM bindings default to force for now
        encryption_key,
    };

    let output = use_case.execute(&mut manifest, input).await
        .map_err(|e| KamarosError::from(e.to_string()).to_js())?;
        
    // Return result
    let result = js_sys::Object::new();
    let updated_manifest = serde_wasm_bindgen::to_value(&manifest)
        .map_err(|e| KamarosError::from(e.to_string()).to_js())?;
        
    js_sys::Reflect::set(&result, &"manifest".into(), &updated_manifest)?;
    js_sys::Reflect::set(&result, &"restoredVersionId".into(), &output.restored_version_id.into())?;
    js_sys::Reflect::set(&result, &"filesRestored".into(), &(output.files_restored as u32).into())?;
    js_sys::Reflect::set(&result, &"patchesApplied".into(), &(output.patches_applied as u32).into())?;
    
    Ok(result.into())
}


/// Export project as ZIP archive
#[wasm_bindgen]
pub async fn export_zip(js_storage: JsStorageAdapter) -> Result<Vec<u8>, JsValue> {
    use kamaros_corelib::application::export_archive::ExportArchiveUseCase;
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    let use_case = ExportArchiveUseCase::new(storage);
    use_case.execute().await.map_err(|e| KamarosError::from(e.to_string()).to_js())
}

/// Import project from ZIP archive
#[wasm_bindgen]
pub async fn import_zip(js_storage: JsStorageAdapter, archive_data: Vec<u8>) -> Result<JsValue, JsValue> {
    use kamaros_corelib::application::import_archive::{ImportArchiveUseCase, ImportArchiveInput};
    let storage = js_adapter::JsStorageWrapper::new(js_storage);
    let use_case = ImportArchiveUseCase::new(storage);
    let input = ImportArchiveInput { archive_data };
    
    let result = use_case.execute(input).await.map_err(|e| KamarosError::from(e.to_string()).to_js())?;
    
    let js_result = js_sys::Object::new();
    js_sys::Reflect::set(&js_result, &"projectName".into(), &result.project_name.into())?;
    js_sys::Reflect::set(&js_result, &"filesImported".into(), &(result.files_imported as u32).into())?;
    js_sys::Reflect::set(&js_result, &"totalSize".into(), &(result.total_size as u32).into())?;
    
    Ok(js_result.into())
}

// Helper: Get current timestamp using JS Date API
fn current_timestamp() -> String {
    let date = js_sys::Date::new_0();
    date.to_iso_string().as_string().unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
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
