# Architektura hybrydowa: Rust Core + TypeScript/Python

## Decyzja: Rust Core + Language Bindings

```
User Application (JS/Python)
         ↓
Language Wrapper (TS/Py)
         ↓
FFI Bindings (WASM/PyO3)
         ↓
Rust Core (kamaros-core)
```

## Zalety

1. **Performance**: Compiled Rust for heavy lifting
2. **Multi-language**: Write once, use everywhere
3. **Type Safety**: Rust's strong types + TS types
4. **Memory Efficient**: No GC in core
5. **Future-proof**: Easy to add more languages

## Trade-offs

- **+2 weeks** development time (Rust setup)
- **+600KB** WASM bundle (but faster)
- **Learning curve** (mitigated by AI/docs)

## Implementacja

### Rust Core (kamaros-core)

```rust
// lib.rs
mod jcf;
mod versioning;
mod diff;
mod hash;
mod zip;
mod manifest;
mod cas;

pub use jcf::JCFManager;
pub use versioning::{VersionManager, VersionId};
pub use diff::{DiffEngine, Patch};
pub use hash::HashEngine;
pub use zip::{ZipCompressor, ZipDecompressor};
pub use manifest::{Manifest, FileEntry, Version};
pub use cas::ContentAddressableStorage;
```

### TypeScript Bindings (WASM)

```typescript
// index.ts
import init, { JCFManager as RustJCFManager } from './pkg/kamaros_core';

export class JCFManager {
  private rustManager: RustJCFManager;

  constructor(adapter: FileSystemAdapter) {
    this.adapter = adapter;
  }

  async init(): Promise<void> {
    await init(); // Initialize WASM
    this.rustManager = new RustJCFManager();
  }

  async saveCheckpoint(message: string): Promise<string> {
    // Delegate to Rust
    return this.rustManager.save_checkpoint(message);
  }

  async restoreVersion(versionId: string): Promise<void> {
    // Delegate to Rust
    return this.rustManager.restore_version(versionId);
  }
}
```

### Python Bindings (PyO3)

```python
# __init__.py
from .kamaros_core import JCFManager as RustJCFManager

class JCFManager:
    def __init__(self, adapter):
        self.adapter = adapter
        self.rust_manager = RustJCFManager()

    def save_checkpoint(self, message: str) -> str:
        return self.rust_manager.save_checkpoint(message)

    def restore_version(self, version_id: str) -> None:
        return self.rust_manager.restore_version(version_id)
```

## Komunikacja między językami

### WASM (JavaScript/TypeScript)

- **wasm-bindgen**: Generate JS bindings from Rust
- **wasm-pack**: Bundle WASM + JS
- **Message passing**: Structured clone for complex objects

```rust
// Rust side
#[wasm_bindgen]
pub struct JCFManager {
    // ...
}

#[wasm_bindgen]
impl JCFManager {
    #[wasm_bindgen]
    pub fn save_checkpoint(&self, message: &str) -> Result<String, JsValue> {
        // Implementation
        Ok(version_id)
    }
}
```

### PyO3 (Python)

- **PyO3**: Rust bindings for Python
- **Native modules**: Direct function calls
- **Python objects**: Automatic conversion

```rust
// Rust side
#[pyclass]
pub struct JCFManager {
    // ...
}

#[pymethods]
impl JCFManager {
    fn save_checkpoint(&self, message: String) -> PyResult<String> {
        // Implementation
        Ok(version_id)
    }
}
```

## Testowanie

### Unit Tests
- **Rust**: cargo test (core logic)
- **TypeScript**: Jest (wrappers, adapters)
- **Python**: pytest (wrappers, adapters)

### Integration Tests
- **Cross-language**: Test full workflows through each binding
- **Performance**: Benchmarks for each language
- **Memory**: Leak detection, peak usage monitoring

### E2E Tests
- **Browser**: Puppeteer/Playwright
- **Node.js**: Direct execution
- **Tauri**: Tauri test runner
- **Python**: pytest with fixtures

## Deployment

### JavaScript/TypeScript
```json
{
  "dependencies": {
    "kamaros": "^1.0.0"
  }
}
```

Bundle zawiera:
- JS wrapper (~20KB)
- WASM binary (~600KB)
- Type definitions (.d.ts)

### Python
```python
pip install kamaros
```

Instaluje:
- Python wheel z compiled Rust extension
- Python wrappers (~15KB)
- Compiled shared library (~800KB)

## Migracja i utrzymanie

### Versioning
- **Semantic versioning**: Major.Minor.Patch
- **Language-specific**: Different versions per language if needed
- **Breaking changes**: Coordinated across all bindings

### Updates
- **Rust core**: Single source of truth
- **Regenerate bindings**: Automated in CI/CD
- **Test all languages**: On every change

## Przyszłe języki

Łatwo dodać nowe języki:

1. **Generate bindings**: Rust FFI tools (wasm-bindgen, PyO3, etc.)
2. **Language wrapper**: Thin layer for language-specific patterns
3. **Tests**: Add language-specific test suite
4. **Documentation**: Language-specific examples

Potencjalne języki:
- **Go**: Via CGO or WASM
- **C#**: .NET WASM or native bindings
- **Java**: JNI or GraalVM
- **Swift**: Swift-WASM or native bindings