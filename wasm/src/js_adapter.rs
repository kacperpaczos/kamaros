//! # JavaScript Storage Adapter
//!
//! Bridge between Rust StoragePort and JavaScript/TypeScript storage adapter.
//! Uses wasm-bindgen to call JS functions for I/O operations.

use wasm_bindgen::prelude::*;
use js_sys::{Uint8Array, Array};
use kamaros_corelib::ports::{StoragePort, PortResult, PortError};


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

    #[wasm_bindgen(method, catch)]
    pub async fn size(this: &JsStorageAdapter, path: &str) -> Result<JsValue, JsValue>;
}

/// Wrapper that provides Rust-friendly interface
pub struct JsStorageWrapper {
    adapter: JsStorageAdapter,
}

// In WASM environment, JS objects are not Send/Sync but they only run on a single thread.
// We must mark them as Send/Sync to satisfy the core traits.
unsafe impl Send for JsStorageWrapper {}
unsafe impl Sync for JsStorageWrapper {}

impl JsStorageWrapper {
    pub fn new(adapter: JsStorageAdapter) -> Self {
        Self { adapter }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl StoragePort for JsStorageWrapper {
    async fn read(&self, path: &str) -> PortResult<Vec<u8>> {
        let result = self.adapter.read(path).await
            .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Read error: {:?}", e))))?;
        
        let uint8_array = Uint8Array::new(&result);
        Ok(uint8_array.to_vec())
    }

    async fn write(&self, path: &str, data: &[u8]) -> PortResult<()> {
        let uint8_array = Uint8Array::from(data);
        self.adapter.write(path, uint8_array).await
            .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Write error: {:?}", e))))
    }

    async fn delete(&self, path: &str) -> PortResult<()> {
        self.adapter.delete_file(path).await
            .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Delete error: {:?}", e))))
    }

    async fn exists(&self, path: &str) -> PortResult<bool> {
        let result = self.adapter.exists(path).await
            .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Exists error: {:?}", e))))?;
        Ok(result.as_bool().unwrap_or(false))
    }

    async fn list(&self, dir: &str) -> PortResult<Vec<String>> {
        let result = self.adapter.list(dir).await
            .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("List error: {:?}", e))))?;
        
        let array = Array::from(&result);
        let mut files = Vec::new();
        
        for i in 0..array.length() {
            if let Some(s) = array.get(i).as_string() {
                files.push(s);
            }
        }
        
        Ok(files)
    }

    async fn size(&self, path: &str) -> PortResult<usize> {
        let result = self.adapter.size(path).await
            .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Size error: {:?}", e))))?;
        Ok(result.as_f64().unwrap_or(0.0) as usize)
    }
}

