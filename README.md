# Kamaros

**Kamaros** is a high-performance, version-controlled virtual file system designed for modern applications. It brings "Git-like" capabilities—snapshots, history, and restoration—directly into your application's data layer.

Built with **Rust** and **WebAssembly**, Kamaros provides a unified, efficient storage format (`.jcf`, JSON Content Format) that is portable across **Node.js**, **Browsers**, and **Python**.

## Technical Highlights

- **Universal Core**: Business logic implementation in Rust, compiled to WebAssembly for consistent behavior across all platforms.
- **Content-Addressable Storage (CAS)**: Automatic deduplication of file content. Identical files stored in different versions or locations occupy space only once.
- **Security at Rest**: Optional AES-GCM encryption for all content and history. Keys are derived using PBKDF2, ensuring data is secure even if the storage backend is compromised.
- **Interoperability**: Built-in support for exporting/importing projects as standard ZIP archives, enabling easy data portability.
- **Storage Agnostic**: Decoupled storage adapters allow running effectively on:
    - **Browser**: Origin Private File System (OPFS) for high-performance web storage.
    - **Node.js**: Native filesystem access.
    - **Python**: In-memory or filesystem integration.
- **Time Travel**: Instant, atomic snapshots and restoration of the entire project state.

## Installation

### JavaScript (Node.js & Browser)
The project is organized as a monorepo. Packages are available via NPM (local build currently):

```bash
npm install @kamaros/core-wasm @kamaros/node  # For Node.js
npm install @kamaros/core-wasm @kamaros/web   # For Browser
```

### Python
```bash
pip install kamaros
```

## Quick Start

### Node.js (with Encryption & ZIP)
```typescript
import { JCFManager, NodeAdapter } from '@kamaros/node';

// Initialize
const adapter = new NodeAdapter('./my-data-store');
const manager = await JCFManager.create(adapter);

// Create Project
await manager.createProject("MyProject");

// 1. Add files
await manager.addFile("secret.txt", new TextEncoder().encode("Top Secret Data"));

// 2. Derive Key (optional, for encryption)
const salt = crypto.getRandomValues(new Uint8Array(16));
const key = await manager.deriveKey("my-password", salt);

// 3. Save Encrypted Checkpoint
const v1 = await manager.saveCheckpoint("Initial encrypted commit", { 
    encryptionKey: key 
});
console.log(`Saved encrypted version: ${v1}`);

// 4. Export to ZIP
const zipData = await manager.exportZip();
console.log(`Exported ZIP size: ${zipData.length} bytes`);
```

### Browser (OPFS)
```typescript
import { JCFManager, OPFSAdapter } from '@kamaros/web';

const adapter = new OPFSAdapter();
const manager = await JCFManager.create(adapter);
await manager.load(); // Load existing project if any
```

### Python
```python
from kamaros import JCFManager, FileAdapter
import os

manager = JCFManager(FileAdapter("./my-data-store"))
manager.create_project("MyProject")

manager.add_file("data.txt", b"Important Data")

# Save with Encryption
salt = os.urandom(16)
key = manager.derive_key("password123", salt)
version_id = manager.save_checkpoint("Snapshot 1", encryption_key=key)

# Export ZIP
zip_bytes = manager.export_zip()
with open("project.zip", "wb") as f:
    f.write(zip_bytes)
```

## Development Guide

### Prerequisites (Linux)

Ensure you have the following toolchains installed:

```bash
# Rust (Stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (Lateest LTS)
sudo apt install nodejs npm

# Python (3.8+)
sudo apt install python3 python3-pip python3-venv

# wasm-pack (for building the Rust core)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Building from Source

This project uses a **Monorepo** structure. The root `package.json` orchestrates the build process for Rust, WASM, and TypeScript packages.

```bash
# 1. Install dependencies
npm install

# 2. Build everything (Rust Core -> WASM -> TypeScript Packages)
npm run build
```

To build the **Python** bindings:

```bash
# Setup virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install build tool
pip install maturin

# Build and install into current venv
cd python
maturin develop --release
```

### Running Tests

**Core Logic (Rust):**
```bash
cargo test
```

**Python Bindings:**
```bash
# with .venv activated
pip install pytest
pytest python/tests
```

**Browser/Integration:**
Examples in `examples/` allow for manual verification.
```bash
# Node.js Example
cd examples/js/node && npm start

# Browser Example (requires local server)
cd examples/js/browser && npm run dev
```

## Project Structure

- `core/`: Pure Rust implementation of the domain logic.
- `wasm/`: Rust crate exposing the Core via `wasm-bindgen`.
- `packages/core-wasm/`: Low-level TypeScript wrappers around the WASM module.
- `packages/node/`: Node.js specific implementations (File System Adapter).
- `packages/web/`: Browser specific implementations (OPFS, IndexedDB Adapters).
- `python/`: Python bindings using PyO3.
- `examples/`: Reference implementations for all supported platforms.

## License

This project is licensed under the MIT License.
