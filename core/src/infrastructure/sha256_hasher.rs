//! # SHA-256 Hasher Adapter
//!
//! Implementation of HasherPort using the `sha2` crate.

use crate::ports::{HasherPort, PortResult};
use sha2::{Digest, Sha256};
use std::io::Read;

/// SHA-256 hasher implementation
pub struct Sha256Hasher;

impl Sha256Hasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl HasherPort for Sha256Hasher {
    /// Compute SHA-256 hash of data
    /// Returns lowercase hex string (64 chars)
    fn hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex_encode(&result)
    }

    /// Compute hash from stream (for large files)
    fn hash_stream(&self, reader: &mut dyn Read) -> PortResult<String> {
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let result = hasher.finalize();
        Ok(hex_encode(&result))
    }
}

/// Convert bytes to lowercase hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_empty() {
        let hasher = Sha256Hasher::new();
        let hash = hasher.hash(b"");
        
        // SHA-256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_hello_world() {
        let hasher = Sha256Hasher::new();
        let hash = hasher.hash(b"hello world");
        
        // SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_deterministic() {
        let hasher = Sha256Hasher::new();
        let data = b"test data 123";
        
        let hash1 = hasher.hash(data);
        let hash2 = hasher.hash(data);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_length() {
        let hasher = Sha256Hasher::new();
        let hash = hasher.hash(b"anything");
        
        // SHA-256 produces 64 character hex string
        assert_eq!(hash.len(), 64);
    }
}
