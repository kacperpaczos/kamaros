//! AES-GCM Encryption Implementation
//!
//! Uses `aes-gcm` crate for authenticated encryption and `pbkdf2` for key derivation.

use crate::ports::{EncryptionPort, PortError, PortResult};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use async_trait::async_trait;
use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha256;
use rand::RngCore;

/// Implementation of EncryptionPort using AES-256-GCM
pub struct AesGcmEncryptor;

impl AesGcmEncryptor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AesGcmEncryptor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl EncryptionPort for AesGcmEncryptor {
    async fn encrypt(&self, key: &[u8], plaintext: &[u8]) -> PortResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(PortError::EncryptionError("Key must be 32 bytes for AES-256".into()));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        // Generate random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| PortError::EncryptionError(format!("Encryption failed: {}", e)))?;

        // Format: Nonce (12 bytes) || Ciphertext (including Tag)
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    async fn decrypt(&self, key: &[u8], data: &[u8]) -> PortResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(PortError::EncryptionError("Key must be 32 bytes for AES-256".into()));
        }
        
        if data.len() < 12 {
            return Err(PortError::EncryptionError("Data too short to contain nonce".into()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| PortError::EncryptionError("Decryption failed (invalid key or tag)".into()))?;

        Ok(plaintext)
    }

    fn derive_key(&self, passphrase: &str, salt: &[u8]) -> PortResult<Vec<u8>> {
        let mut key = [0u8; 32]; // 256 bits
        
        // PBKDF2-HMAC-SHA256
        // Iterations: 600,000 (OWASP recommendation for PBKDF2-HMAC-SHA256 is 600,000)
        let iterations = 600_000;
        
        pbkdf2::<Hmac<Sha256>>(
            passphrase.as_bytes(),
            salt,
            iterations,
            &mut key
        ).map_err(|e| PortError::EncryptionError(format!("Key derivation failed: {}", e)))?;
        
        Ok(key.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let encryptor = AesGcmEncryptor::new();
        let key = encryptor.derive_key("password123", b"salt123").unwrap();
        
        let original_data = b"Secret Message";
        
        // Encrypt
        let encrypted = encryptor.encrypt(&key, original_data).await.unwrap();
        assert_ne!(encrypted, original_data);
        assert!(encrypted.len() > original_data.len()); // Nonce + Tag overhead
        
        // Decrypt
        let decrypted = encryptor.decrypt(&key, &encrypted).await.unwrap();
        assert_eq!(decrypted, original_data);
    }
    
    #[tokio::test]
    async fn test_different_nonces_produce_different_ciphertexts() {
        let encryptor = AesGcmEncryptor::new();
        let key = encryptor.derive_key("password", b"salt").unwrap();
        let data = b"Same Content";
        
        let enc1 = encryptor.encrypt(&key, data).await.unwrap();
        let enc2 = encryptor.encrypt(&key, data).await.unwrap();
        
        assert_ne!(enc1, enc2); // Same data + same key = diff ciphertext (random nonce)
    }
}
