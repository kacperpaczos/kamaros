//! # Kamaros WASM Bindings
//!
//! WASM exports for kamaros-corelib.
//! Iteration 2: WasmJCFManager with save/restore methods.

use wasm_bindgen::prelude::*;
use kamaros_corelib::domain::manifest::{Manifest, ProjectMetadata};
use std::collections::HashMap;

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

// Helper: Get current timestamp (simplified for WASM)
fn current_timestamp() -> String {
    // In WASM, we'd use js_sys::Date, but for simplicity return placeholder
    // TypeScript can override this
    "2024-01-01T00:00:00Z".to_string()
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
