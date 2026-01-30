# ADR 002: Encryption at Rest

## Status
Accepted

## Context
Applications dealing with sensitive user data (e.g., in medical or financial domains) require that data is stored securely. Since Kamaros often runs in untrusted environments (local device storage, public cloud buckets), we need a robust encryption mechanism that prevents unauthorized access to the content and version history.

## Decision
We implemented a client-side encryption strategy using **AES-GCM** and **PBKDF2**.

### Algorithms
- **Cipher**: AES-256-GCM (Galois/Counter Mode). ensuring both confidentiality and authenticity.
- **Key Derivation**: PBKDF2-HMAC-SHA256.
  - Iterations: 600,000 (OWASP recommendation).
  - Salt: 16-byte random salt.
  - Key Length: 32 bytes (256 bits).

### Scope
- **Blob Encryption**: Individual files (blobs) are encrypted before writing to persistent storage.
- **Path Obfuscation**: (Future Consideration) Currently filenames in the manifest are plain text, but content is encrypted.
- **Manifest**: The `manifest.json` itself relies on the storage layer's access control, but `SaveCheckpoint` supports encrypting the individual blobs it references.

### Workflow
1. **Derivation**: User provides a password. System generates a salt and derives the Master Key.
2. **Encryption**: 
   - A unique 96-bit Nonce (IV) is generated for each blob.
   - Content is encrypted: `Ciphertext = AES_GCM(Key, Nonce, Plaintext)`.
   - Stored format: `[Nonce (12 bytes)] + [Ciphertext] + [Tag (16 bytes)]`.
3. **Decryption**:
   - Read 12-byte Nonce.
   - Decrypt rest of the file using the Key and Nonce.

## Consequences

### Positive
- **Strong Security**: Industry-standard authenticated encryption.
- **Granularity**: Encryption happens at the blob level, allowing mixed content (some encrypted, some public) if needed in the future.
- **Portability**: All crypto primitives are standard and available in WebCrypto (JS), RustCrypto (WASM), and Cryptography (Python).

### Negative
- **Performance**: Encryption/Decryption adds CPU overhead.
- **Key Management**: The User is responsible for managing the encryption key/password. Losing the key means data loss.
