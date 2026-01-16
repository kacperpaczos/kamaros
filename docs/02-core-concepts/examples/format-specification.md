# JCF Format Specification Examples

## Overview

This document provides concrete examples of JCF (JSON Content Format) file structure and specification files.

## File Structure Example

```
project.jcf (ZIP Archive)
│
├── mimetype                          [UNCOMPRESSED, FIRST]
│   └── "application/x-jcf"
│
├── manifest.json                     [COMPRESSED]
│   └── See manifest-example.json
│
├── content/                          [WORKING COPY - Current State]
│   ├── src/
│   │   └── main.js                  [Current version - full content]
│   ├── assets/
│   │   └── logo.png                 [Current version - full content]
│   └── config.json                  [Current version - full content]
│
└── .store/                           [VERSIONING STORAGE]
    ├── blobs/                        [Content Addressable Storage]
    │   └── a3f5e8b2c1d4e6f7...      [Binary files by SHA-256 hash]
    │
    └── deltas/                       [Reverse Delta Patches]
        ├── v3_src_main.js.patch      [v3 → v2 reverse patch]
        ├── v2_config.json.patch     [v2 → v1 reverse patch]
        └── v1_src_main.js.patch      [v1 → null (initial)]
```

## Manifest.json Structure

See `manifest-example.json` for a complete example.

### Key Fields:

- **formatVersion**: Version of JCF format specification (currently "1.0.0")
- **metadata**: Project metadata (author, dates, application info)
- **fileMap**: Map of all files in the project with their metadata
- **versionHistory**: Array of all versions (commits) with file states
- **refs**: References to important versions (HEAD, branches)
- **renameLog**: History of file renames (for tracking files across renames)
- **config**: JCF configuration (autoGC, compression, etc.)

## Delta Patch Format

Delta patches use the standard unified diff format. See `delta-example.patch` for an example.

### Reverse Delta Strategy:

- **HEAD (current version)**: Stored in full in `content/`
- **Previous versions**: Reconstructed by applying reverse patches backwards
- **Patch naming**: `{versionId}_{filepath_hash}.patch`

Example:
- `v3_src_main.js.patch` - patch to go from v3 → v2
- `v2_src_main.js.patch` - patch to go from v2 → v1
- To get v1: Start with v3 (HEAD), apply v3 patch → v2, apply v2 patch → v1

## Blob Storage (CAS)

Binary files are stored in `.store/blobs/` using their SHA-256 hash as filename.

Example:
- File: `assets/logo.png` with hash `a3f5e8b2c1d4e6f7...`
- Stored as: `.store/blobs/a3f5e8b2c1d4e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1`
- If same file appears in multiple versions → same blob reused (deduplication)

## Content Directory

The `content/` directory contains the current working state (HEAD version) in full.

- All files are stored as-is (no patches needed)
- Fast access - no reconstruction required
- Mirrors the actual project structure

## Validation Rules

### Manifest.json Validation:

1. **formatVersion** must be valid semantic version
2. **metadata** must contain required fields (author, created_at, etc.)
3. **fileMap** entries must have valid inodeId (UUID v4)
4. **versionHistory** must form valid chain (parentId references)
5. **refs.head** must reference valid version ID
6. All **contentRef** paths must exist in ZIP
7. All **hash** values must be valid SHA-256 (64 hex chars)

### ZIP Structure Validation:

1. **mimetype** must be first entry, uncompressed, contain "application/x-jcf"
2. All paths must be valid (no "..", no absolute paths)
3. All referenced blobs must exist
4. All referenced deltas must exist
5. Content directory structure must match fileMap

## Example Usage

### Creating a JCF file:

```typescript
const manager = new JCFManager();
await manager.init(new FileAdapter('./project'));

// Add files
await manager.addFile('src/main.js', 'function hello() { return "Hello"; }');
await manager.addFile('assets/logo.png', imageData);

// Create version
const v1 = await manager.saveCheckpoint('Initial commit');

// Result: project.jcf created with manifest.json, content/, .store/
```

### Reading a JCF file:

```typescript
const manager = new JCFManager();
await manager.init(new FileAdapter('./project'));

// Load existing JCF
await manager.import(fs.readFileSync('project.jcf'));

// Access current files
const mainJs = await manager.getFileContent('src/main.js');
const logo = await manager.getFileContent('assets/logo.png');

// Access history
const history = await manager.getHistory();
const v1 = await manager.getVersion('v1-abc123def456');
```

## File Type Detection

The library detects file types (text vs binary) **only for versioning strategy**:

- **Text files** (`.js`, `.ts`, `.json`, `.txt`, `.md`, etc.) → Reverse delta patches
- **Binary files** (`.png`, `.jpg`, `.mp4`, `.zip`, etc.) → Content Addressable Storage

This is **not content interpretation** - the library doesn't know what a `.js` file contains, only that it should use delta strategy for versioning.

## Compression Strategy

- **Already compressed** (`.png`, `.jpg`, `.mp4`, `.zip`) → STORE (no compression)
- **Text files** → DEFLATE (compression level 6 by default)
- **Manifest.json** → DEFLATE (compression level 6)
- **Deltas** → DEFLATE (compression level 6)

## Recovery

If a JCF file is corrupted:

1. **Standard ZIP tools** can extract what's possible
2. **Manifest.json** can be manually edited if needed
3. **Content directory** can be extracted directly (current state)
4. **Blobs** can be recovered individually by hash
5. **Deltas** can be recovered if patches are intact

## Extension and MIME Type

- **File extension**: `.jcf`
- **MIME type**: `application/x-jcf`
- **Magic bytes**: ZIP header (PK\x03\x04) + mimetype entry

## See Also

- [manifest-example.json](./manifest-example.json) - Complete manifest example
- [delta-example.patch](./delta-example.patch) - Delta patch example
- [JCF Format Specification](../01-jcf-format.md) - Full specification
