//! # Memory Storage Adapter
//!
//! In-memory implementation of StoragePort for testing.

use crate::ports::{PortError, PortResult, StoragePort};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory storage for testing
pub struct MemoryStorage {
    files: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Create with initial files
    pub fn with_files(files: HashMap<String, Vec<u8>>) -> Self {
        Self {
            files: RwLock::new(files),
        }
    }

    /// Get all stored files (for testing)
    pub fn get_all_files(&self) -> HashMap<String, Vec<u8>> {
        self.files.read().unwrap().clone()
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl StoragePort for MemoryStorage {
    async fn read(&self, path: &str) -> PortResult<Vec<u8>> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| PortError::NotFound(path.to_string()))
    }

    async fn write(&self, path: &str, data: &[u8]) -> PortResult<()> {
        let mut files = self.files.write().unwrap();
        files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn delete(&self, path: &str) -> PortResult<()> {
        let mut files = self.files.write().unwrap();
        files.remove(path);
        Ok(())
    }

    async fn exists(&self, path: &str) -> PortResult<bool> {
        let files = self.files.read().unwrap();
        Ok(files.contains_key(path))
    }

    async fn list(&self, dir: &str) -> PortResult<Vec<String>> {
        let files = self.files.read().unwrap();
        let prefix = if dir.ends_with('/') { dir.to_string() } else { format!("{}/", dir) };
        
        let result: Vec<String> = files
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .map(|k| k.strip_prefix(&prefix).unwrap_or(k).to_string())
            .filter(|k| !k.contains('/')) // Only direct children
            .collect();
        
        Ok(result)
    }

    async fn size(&self, path: &str) -> PortResult<usize> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .map(|d| d.len())
            .ok_or_else(|| PortError::NotFound(path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_read_write() {
        let storage = MemoryStorage::new();
        
        storage.write("test.txt", b"hello world").await.unwrap();
        let content = storage.read("test.txt").await.unwrap();
        
        assert_eq!(content, b"hello world");
    }

    #[tokio::test]
    async fn test_memory_storage_not_found() {
        let storage = MemoryStorage::new();
        
        let result = storage.read("nonexistent.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_storage_list() {
        let storage = MemoryStorage::new();
        
        storage.write("content/file1.txt", b"a").await.unwrap();
        storage.write("content/file2.txt", b"b").await.unwrap();
        storage.write("content/subdir/file3.txt", b"c").await.unwrap();
        
        let files = storage.list("content").await.unwrap();
        
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));
    }
}
