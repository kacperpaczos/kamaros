# ADR 003: ZIP Archive Format

## Status
Accepted

## Context
Kamaros projects are stored in storage backends that may not be easily portable (e.g., Browser OPFS, IndexedDB, S3 Buckets). To allow users to move projects between environments (e.g., Browser -> Desktop -> Cloud), back up their work, or share projects with others, we need a standard, portable archive format.

## Decision
We utilize the **ZIP** file format as the standard container for Export/Import operations.

### Structure
The ZIP archive mirrors the internal JCF storage layout:

```
my_project.zip
├── .store/
│   ├── manifest.json       # The source of truth for history
│   ├── blobs/              # CAS content
│   │   ├── abc12...
│   │   └── ...
│   └── deltas/             # Reverse diffs (if any)
└── content/                # The "Working Directory"
    ├── src/
    │   └── main.rs
    └── README.md
```

### Rationale
1.  **Ubiquity**: ZIP is supported natively by all major operating systems and via standard libraries in almost every programming language (Rust `zip` crate, JS `JSZip` / `fflate`, Python `zipfile`).
2.  **Compression**: Provides built-in compression (Deflate) reducing the size of text-heavy source code and JSON metadata.
3.  **Inspectability**: Users can open the archive with standard system tools to inspect `content/` without needing special Kamaros tooling.

### Constraints
- **Self-Contained**: The ZIP must contain the full `.store/` to preserve version history.
- **Paths**: All paths within the ZIP use forward slashes (`/`) as separators for cross-platform compatibility.

## Consequences

### Positive
- **Portability**: Seamless transfer between Browser (WASM) and Server (Node/Python).
- **Backup**: Easy manual backup solution.
- **Verification**: Integrity of the archive can be checked using standard checksums (CRC32) inherent to ZIP.

### Negative
- **Size**: For large projects with extensive history, the ZIP file can become large.
- **Memory Usage**: Current implementation constructs the ZIP in-memory (WASM constraint). Future optimization might require streaming (supported in Node/Python but harder in browser WASM context).
