//! # Ports Layer
//! 
//! Defines interfaces (traits) for external adapters.
//! These ports abstract away platform-specific I/O operations.

use async_trait::async_trait;

/// Error types for port operations
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Patch application failed: {0}")]
    PatchFailed(String),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
}

pub type PortResult<T> = std::result::Result<T, PortError>;

/// Storage Port - abstraction for file system operations
/// 
/// Implementations:
/// - `MemoryStorage` (for tests)
/// - `NodeFsStorage` (via WASM/FFI)
/// - `BrowserStorage` (via IndexedDB/OPFS)
#[async_trait]
pub trait StoragePort: Send + Sync {
    /// Read file contents
    async fn read(&self, path: &str) -> PortResult<Vec<u8>>;
    
    /// Write data to file (creates parent dirs if needed)
    async fn write(&self, path: &str, data: &[u8]) -> PortResult<()>;
    
    /// Delete a file
    async fn delete(&self, path: &str) -> PortResult<()>;
    
    /// Check if file exists
    async fn exists(&self, path: &str) -> PortResult<bool>;
    
    /// List files in directory
    async fn list(&self, dir: &str) -> PortResult<Vec<String>>;
    
    /// Get file size in bytes
    async fn size(&self, path: &str) -> PortResult<usize>;
}

/// Diff Port - text diffing and patching
/// 
/// Uses unified diff format for compatibility.
/// Implementation uses `similar` crate.
pub trait DiffPort: Send + Sync {
    /// Compute unified diff between old and new text
    /// Returns patch string in unified diff format
    fn compute_diff(&self, old: &str, new: &str) -> String;
    
    /// Apply unified diff patch to text
    /// Returns patched text or error if patch cannot be applied
    fn apply_patch(&self, text: &str, patch: &str) -> PortResult<String>;
}

/// Hasher Port - content hashing
/// 
/// Uses SHA-256 for content-addressable storage.
pub trait HasherPort: Send + Sync {
    /// Compute SHA-256 hash of data
    /// Returns lowercase hex string (64 chars)
    fn hash(&self, data: &[u8]) -> String;
    
    /// Compute hash from stream (for large files)
    fn hash_stream(&self, reader: &mut dyn std::io::Read) -> PortResult<String>;
}

/// Compressor Port - ZIP compression/decompression
/// 
/// For streaming large files.
#[async_trait]
pub trait CompressorPort: Send + Sync {
    /// Compress data with specified level (0-9)
    async fn compress(&self, data: &[u8], level: u32) -> PortResult<Vec<u8>>;
    
    /// Decompress data
    async fn decompress(&self, data: &[u8]) -> PortResult<Vec<u8>>;
}

// Re-exports for convenience
pub use PortError as Error;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test that traits are object-safe
    fn _assert_object_safe<T: StoragePort + ?Sized>() {}
    fn _assert_diff_object_safe<T: DiffPort + ?Sized>() {}
}
