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

    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

pub type PortResult<T> = std::result::Result<T, PortError>;

/// Storage Port - abstraction for file system operations
/// 
/// Implementations:
/// - `MemoryStorage` (for tests)
/// - `NodeFsStorage` (via WASM/FFI)
/// - `BrowserStorage` (via IndexedDB/OPFS)
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait StoragePort: Send + Sync {
    /// Read file contents (for small files <50MB)
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
    
    // =========================================================================
    // STREAMING METHODS (for large files >50MB)
    // =========================================================================
    
    /// Read file in chunks (streaming)
    /// 
    /// Returns data as chunks. Suitable for processing large files incrementally.
    /// Default implementation falls back to `read()` for backwards compatibility.
    async fn read_chunked(
        &self, 
        path: &str, 
        chunk_size: usize,
    ) -> PortResult<Vec<Vec<u8>>> {
        let data = self.read(path).await?;
        Ok(data.chunks(chunk_size).map(|c| c.to_vec()).collect())
    }
    
    /// Write file from chunks (streaming)
    /// 
    /// Concatenates chunks and writes to file.
    /// Default implementation buffers all chunks and calls `write()`.
    async fn write_chunked(
        &self, 
        path: &str, 
        chunks: Vec<Vec<u8>>,
    ) -> PortResult<()> {
        let data: Vec<u8> = chunks.into_iter().flatten().collect();
        self.write(path, &data).await
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<S: StoragePort + ?Sized> StoragePort for std::sync::Arc<S> {
    async fn read(&self, path: &str) -> PortResult<Vec<u8>> {
        (**self).read(path).await
    }
    
    async fn write(&self, path: &str, data: &[u8]) -> PortResult<()> {
        (**self).write(path, data).await
    }
    
    async fn delete(&self, path: &str) -> PortResult<()> {
        (**self).delete(path).await
    }
    
    async fn exists(&self, path: &str) -> PortResult<bool> {
        (**self).exists(path).await
    }
    
    async fn list(&self, dir: &str) -> PortResult<Vec<String>> {
        (**self).list(dir).await
    }
    
    async fn size(&self, path: &str) -> PortResult<usize> {
        (**self).size(path).await
    }

    async fn read_chunked(
        &self, 
        path: &str, 
        chunk_size: usize,
    ) -> PortResult<Vec<Vec<u8>>> {
        (**self).read_chunked(path, chunk_size).await
    }

    async fn write_chunked(
        &self, 
        path: &str, 
        chunks: Vec<Vec<u8>>,
    ) -> PortResult<()> {
        (**self).write_chunked(path, chunks).await
    }
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
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CompressorPort: Send + Sync {
    /// Compress data with specified level (0-9)
    async fn compress(&self, data: &[u8], level: u32) -> PortResult<Vec<u8>>;
    
    /// Decompress data
    async fn decompress(&self, data: &[u8]) -> PortResult<Vec<u8>>;
}

/// Encryption Port - encryption/decryption for data at rest
/// 
/// Uses authenticated encryption (AES-GCM) for security.
/// Keys are derived from user-provided passphrases via PBKDF2/Argon2.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait EncryptionPort: Send + Sync {
    /// Encrypt data with the provided key
    /// 
    /// Returns ciphertext with nonce prepended (nonce || ciphertext || tag).
    async fn encrypt(&self, key: &[u8], plaintext: &[u8]) -> PortResult<Vec<u8>>;
    
    /// Decrypt data with the provided key
    /// 
    /// Expects nonce || ciphertext || tag format.
    async fn decrypt(&self, key: &[u8], ciphertext: &[u8]) -> PortResult<Vec<u8>>;
    
    /// Derive key from passphrase using PBKDF2 (or Argon2)
    /// 
    /// Returns 256-bit (32 bytes) key suitable for AES-256-GCM.
    fn derive_key(&self, passphrase: &str, salt: &[u8]) -> PortResult<Vec<u8>>;
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
