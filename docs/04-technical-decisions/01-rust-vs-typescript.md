# Rust vs TypeScript - decyzje techniczne

## Matryca decyzyjna

### Analiza komponentów

| Komponent | CPU Load | I/O Load | Złożoność | Shared Logic | Platform | Rekomendacja | Uzasadnienie |
|-----------|----------|----------|-----------|--------------|----------|--------------|--------------|
| SHA-256 Hashing | 🔥🔥🔥 | - | Łatwe | ✅ 100% | Wszystkie | **RUST** | Pure computation, 5x speedup |
| Diff Algorithm (Myers) | 🔥🔥🔥 | - | Średnie | ✅ 100% | Wszystkie | **RUST** | CPU-intensive, 6.7x speedup |
| Patch Apply | 🔥🔥 | - | Średnie | ✅ 100% | Wszystkie | **RUST** | String manipulation, 4.4x speedup |
| ZIP Compression | 🔥🔥🔥 | 🔥 | Trudne | ✅ 100% | Wszystkie | **RUST** | I/O + CPU, 4.7x speedup |
| ZIP Decompression | 🔥🔥 | 🔥🔥 | Trudne | ✅ 100% | Wszystkie | **RUST** | I/O intensive, 4x speedup |
| Manifest Parsing (JSON) | 🔥 | - | Łatwe | ✅ 100% | Wszystkie | **RUST** | Serde super fast, 5x speedup |
| Manifest Validation | 🔥 | - | Średnie | ✅ 100% | Wszystkie | **RUST** | Schema validation |
| Version Graph (DAG) | 🔥 | - | Średnie | ✅ 100% | Wszystkie | **RUST** | BFS/DFS algorithms |
| LRU Cache | 🔥 | - | Łatwe | ✅ 100% | Wszystkie | **RUST** | Memory management |
| Garbage Collection | 🔥🔥 | 🔥 | Średnie | ✅ 100% | Wszystkie | **RUST** | Set operations, mark & sweep |
| Content Addressing | 🔥 | - | Łatwe | ✅ 100% | Wszystkie | **RUST** | Hash-based, pure logic |
| Reverse Delta Logic | 🔥🔥 | - | Trudne | ✅ 100% | Wszystkie | **RUST** | Core algorithm |
| File System API | - | 🔥🔥🔥 | Trudne | ❌ 0% | Specific | **TypeScript/Python** | Platform-specific APIs |
| IndexedDB (Browser) | - | 🔥🔥 | Średnie | ❌ 0% | Browser only | **TypeScript** | Web API |
| Node.js fs | - | 🔥🔥 | Średnie | ❌ 0% | Node.js only | **TypeScript** | Node API |
| Tauri fs | - | 🔥🔥 | Średnie | ❌ 0% | Tauri only | **TypeScript** | Tauri API |
| Python pathlib | - | 🔥🔥 | Średnie | ❌ 0% | Python only | **Python** | Python stdlib |
| Event System | - | - | Łatwe | ❌ 0% | Specific | **TypeScript/Python** | Language-native (EventEmitter) |
| Streaming API | - | 🔥🔥🔥 | Trudne | ⚠️ 50% | Specific | **HYBRID** | ReadableStream (TS), io (Python) |
| Error Handling | - | - | Średnie | ⚠️ 50% | Specific | **HYBRID** | Result<T> (Rust), Exceptions (TS/Py) |
| Progress Tracking | - | - | Łatwe | ❌ 0% | Specific | **TypeScript/Python** | Callbacks, events |

**Legenda**:
- 🔥 = Intensywność (więcej = wyższe obciążenie)
- Złożoność implementacji
- ✅ = Logika wspólna 100% (identyczna dla wszystkich języków)
- ⚠️ = Częściowo wspólna (hybrid approach)
- ❌ = Specyficzna dla platformy (nie da się współdzielić)

### Oczekiwane charakterystyki wydajności

**Zalety Rust Core**:

| Kategoria operacji | Oczekiwana poprawa | Uzasadnienie |
|-------------------|-------------------|--------------|
| SHA-256 Hashing | Znaczna | Native crypto vs JavaScript implementation |
| Diff Computation | Bardzo znaczna | Compiled algorithm vs interpreted |
| Patch Apply | Znaczna | String manipulation in compiled code |
| ZIP Compression | Znaczna | Native flate2 vs JavaScript |
| JSON Parsing | Umiarkowana | serde (compiled) vs JSON.parse |
| GC Operations | Znaczna | Manual memory management vs GC |
| Graph Operations | Umiarkowana | Compiled data structures |

**Klasyfikacja priorytetów**:
- 🟢 High: Częste operacje (save, load)
- 🟡 Medium: Okazjonalne operacje (restore, GC)
- 🔴 Low: Rzadkie operacje (deep history)

### Charakterystyki pamięci

**Zalety Rust Core**:
- **No GC Overhead**: Manual memory management, predictable allocations
- **Stack Allocation**: More efficient for temporary data
- **Zero-Copy Operations**: Reduce memory copies where possible
- **Efficient Layout**: Struct packing, cache-friendly data structures

**Oczekiwane korzyści**:
- Niższe bazowe zużycie pamięci
- No GC pauses during operations
- Predictable memory patterns
- Better for large files (>100MB)

### Rozważania rozmiaru bundle

**Trade-offs**:

| Aspect | Pure TypeScript | Rust Core + WASM |
|--------|----------------|------------------|
| Core Logic | Smaller (TypeScript) | Minimal (compiled) |
| WASM Binary | None | ~500-800 KB (estimated) |
| Dependencies | Multiple JS libs | Fewer (built into WASM) |
| **Trade-off** | Smaller w/o WASM | Larger initially, fewer deps |

**Note**: WASM adds initial size but may reduce overall bundle with dependencies included.

### Architektura projektu

```
kamaros/
├── core/                    # Rust core (kamaros-core)
│   ├── src/
│   │   ├── lib.rs           # Main library exports
│   │   ├── jcf.rs           # JCF format implementation
│   │   ├── versioning.rs    # Time-travel versioning
│   │   ├── diff.rs          # Myers diff algorithm
│   │   ├── hash.rs          # SHA-256 implementation
│   │   ├── zip.rs           # Compression/decompression
│   │   ├── manifest.rs      # JSON manifest handling
│   │   └── cas.rs           # Content addressing
│   ├── Cargo.toml
│   └── build.rs             # WASM build script
│
├── js/                      # JavaScript/TypeScript bindings
│   ├── src/
│   │   ├── index.ts         # Main exports
│   │   ├── JCFManager.ts    # Public API
│   │   ├── adapters/        # Platform adapters
│   │   │   ├── BrowserAdapter.ts
│   │   │   ├── NodeAdapter.ts
│   │   │   └── TauriAdapter.ts
│   │   ├── core/            # Core logic wrappers
│   │   │   ├── VersionManager.ts
│   │   │   ├── FileManager.ts
│   │   │   └── DeltaManager.ts
│   │   └── types.ts         # TypeScript definitions
│   ├── package.json
│   ├── tsconfig.json
│   └── webpack.config.js    # WASM bundling
│
├── python/                  # Python bindings (future)
│   ├── src/
│   │   ├── __init__.py
│   │   └── kamaros/
│   ├── setup.py
│   └── requirements.txt
│
├── docs/                    # Documentation
│   ├── api/                 # API reference
│   ├── examples/            # Usage examples
│   └── guides/              # User guides
│
├── tests/                   # Test suite
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   └── performance/         # Performance benchmarks
│
├── tools/                   # Development tools
│   ├── build/               # Build scripts
│   ├── lint/                # Linting configs
│   └── release/             # Release automation
│
├── package.json             # Root package.json
├── Cargo.toml               # Root Cargo.toml (workspace)
├── README.md
└── LICENSE
```

### Konwencje nazewnictwa

#### Rust (snake_case)
```rust
// Modules
mod jcf_format;
mod time_travel_versioning;
mod content_addressable_storage;

// Functions
fn save_checkpoint(message: &str) -> Result<VersionId, Error>
fn restore_version(version_id: &VersionId) -> Result<(), Error>

// Types
struct JCFManager;
struct VersionHistory;
struct FileEntry;
```

#### TypeScript (camelCase)
```typescript
// Classes
class JCFManager
class VersionManager
class FileManager

// Methods
saveCheckpoint(message: string): Promise<string>
restoreVersion(versionId: string): Promise<void>

// Interfaces
interface Manifest
interface FileEntry
interface Version
```

### Zasady kodowania

#### Rust Standards
- `rustfmt` for formatting
- `clippy` for linting
- Comprehensive error handling with `thiserror`
- Zero unsafe code (except FFI bindings)
- Full test coverage (>90%)

#### TypeScript Standards
- ESLint + Prettier
- Strict TypeScript config
- Comprehensive error handling
- Full test coverage (>90%)

### Build Pipeline

```
Source Code
     ↓
Rust Compiler (cargo build --target wasm32-unknown-unknown)
     ↓
WASM Binary + JS Bindings
     ↓
Webpack/Rollup (bundle WASM + JS)
     ↓
Final Bundle (.js + .wasm)
```

### Performance Benchmarks

**Targets**:
- Save checkpoint: <500ms for 100 files
- Load project: <200ms for 1000 files
- Restore version: <1s for 50 commits back
- Hash 100MB file: <2s
- Compress 50MB: <3s

**Memory Limits**:
- Browser: <100MB peak for 500MB project
- Node.js: <200MB peak for 1GB project
- Tauri: <150MB peak for 1GB project