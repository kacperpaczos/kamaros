# ADR 001: Content Addressable Storage (CAS)

## Status
Accepted

## Context
Kamaros needs to store version history of files efficiently. A naive approach of storing full copies of every file for every version would lead to massive storage amplification. We need a mechanism that:
1. Minimizes storage usage for duplicate content.
2. Ensures data integrity.
3. Allows for efficient synchronization and diffing.

## Decision
We decided to implement a **Content Addressable Storage (CAS)** system for managing file content.

### Implementation Details
- **Hashing Algorithm**: SHA-256. This provides a strong guarantee against collisions and is widely supported natively (SubtleCrypto in browsers, Ring/RustCrypto in Rust).
- **Blob Storage**: All file content is stored in a flat directory `.store/blobs/`, where the filename is the SHA-256 hash of the content.
- **Deduplication**: Since the filename is the hash, identical content (regardless of original filename or location) maps to the same blob. Writing the same content multiple times results in a no-op if the blob already exists.
- **Manifest**: The `Manifest` maps logical file paths (e.g., `src/main.rs`) to their current content hash.

### directory Structure
```
.store/
  ├── manifest.json  # logical path -> hash mapping
  └── blobs/
      ├── 8d23...ac91  # content of file A
      └── e5a1...bd02  # content of file B
```

## Consequences

### Positive
- **Storage Efficiency**: Zero storage cost for duplicate files or reverting to previous versions.
- **Integrity**: Any corruption in a blob is easily detected by re-hashing.
- **Simplicity**: No complex delta chains for binary files (though we use reverse deltas for text history optimization, the CAS blobs are the source of truth for HEAD).

### Negative
- **Indirection**: Reading a file requires two lookups (Manifest -> Hash, Hash -> Blob).
- **Garbage Collection**: Deleting a file from the manifest doesn't delete the blob immediately (it might be used by history). Pruning (GC) is required to remove orphaned blobs.
