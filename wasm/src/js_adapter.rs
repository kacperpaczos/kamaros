//! # JavaScript Storage Adapter
//!
//! Bridge between Rust StoragePort and JavaScript/TypeScript storage adapter.
//! Uses wasm-bindgen to call JS functions for I/O operations.

use wasm_bindgen::prelude::*;
use js_sys::{Uint8Array, Array};

/// JavaScript storage adapter interface
/// 
/// TypeScript must implement this interface:
/// ```typescript
/// interface JsStorageAdapter {
///     read(path: string): Promise<Uint8Array>;
///     write(path: string, data: Uint8Array): Promise<void>;
///     delete(path: string): Promise<void>;
///     exists(path: string): Promise<boolean>;
///     list(dir: string): Promise<string[]>;
/// }
/// ```
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "JsStorageAdapter")]
    pub type JsStorageAdapter;

    #[wasm_bindgen(method, catch)]
    pub async fn read(this: &JsStorageAdapter, path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn write(this: &JsStorageAdapter, path: &str, data: Uint8Array) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "delete")]
    pub async fn delete_file(this: &JsStorageAdapter, path: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn exists(this: &JsStorageAdapter, path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn list(this: &JsStorageAdapter, dir: &str) -> Result<JsValue, JsValue>;
}

/// Wrapper that provides Rust-friendly interface
pub struct JsStorageWrapper {
    adapter: JsStorageAdapter,
}

impl JsStorageWrapper {
    pub fn new(adapter: JsStorageAdapter) -> Self {
        Self { adapter }
    }

    pub async fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        let result = self.adapter.read(path).await
            .map_err(|e| format!("Read error: {:?}", e))?;
        
        let uint8_array = Uint8Array::new(&result);
        Ok(uint8_array.to_vec())
    }

    pub async fn write(&self, path: &str, data: &[u8]) -> Result<(), String> {
        let uint8_array = Uint8Array::from(data);
        self.adapter.write(path, uint8_array).await
            .map_err(|e| format!("Write error: {:?}", e))
    }

    pub async fn delete(&self, path: &str) -> Result<(), String> {
        self.adapter.delete_file(path).await
            .map_err(|e| format!("Delete error: {:?}", e))
    }

    pub async fn exists(&self, path: &str) -> Result<bool, String> {
        let result = self.adapter.exists(path).await
            .map_err(|e| format!("Exists error: {:?}", e))?;
        Ok(result.as_bool().unwrap_or(false))
    }

    pub async fn list(&self, dir: &str) -> Result<Vec<String>, String> {
        let result = self.adapter.list(dir).await
            .map_err(|e| format!("List error: {:?}", e))?;
        
        let array = Array::from(&result);
        let mut files = Vec::new();
        
        for i in 0..array.length() {
            if let Some(s) = array.get(i).as_string() {
                files.push(s);
            }
        }
        
        Ok(files)
    }
}

