# Organizacja modułów

## Rust Core

### Struktura modułów
```
src/
├── lib.rs              # Public API exports
├── jcf.rs              # Main JCF logic
├── versioning.rs       # Version management
├── diff.rs             # Diff algorithms
├── hash.rs             # Hash functions
├── zip.rs              # ZIP handling
├── manifest.rs         # Manifest serialization
├── cas.rs              # Content addressing
├── errors.rs           # Error types
└── utils.rs            # Utilities
```

### Zależności między modułami
- `jcf.rs` → uses all other modules
- `versioning.rs` → uses `manifest.rs`, `diff.rs`
- `cas.rs` → uses `hash.rs`, `zip.rs`
- Independent modules: `errors.rs`, `utils.rs`

## TypeScript Bindings

### Struktura
```
src/
├── index.ts            # Main exports
├── JCFManager.ts       # Public API
├── adapters/           # Platform adapters
│   ├── BrowserAdapter.ts
│   ├── NodeAdapter.ts
│   └── TauriAdapter.ts
├── core/               # Core logic wrappers
│   ├── VersionManager.ts
│   ├── FileManager.ts
│   └── DeltaManager.ts
├── types.ts            # Type definitions
└── utils.ts            # Utilities
```

### Architektura
- **Thin wrappers**: Minimal logic, delegate to Rust
- **Type safety**: Full TypeScript types
- **Error handling**: Convert Rust errors to JS exceptions
- **Async/await**: All APIs return Promises

## Python Bindings (Future)

### Struktura
```
kamaros/
├── __init__.py         # Main exports
├── manager.py          # JCFManager wrapper
├── adapters/           # Platform adapters
├── core/               # Core logic
└── types.py            # Type stubs
```

### Architektura
- **PyO3 bindings**: Direct Rust FFI
- **Pythonic API**: Follow Python conventions
- **Context managers**: Resource management
- **Type hints**: Full type annotations

## Separation of Concerns

### Core Logic (Rust)
- Algorithm implementation
- Data structures
- File format handling
- Performance-critical code

### Language Bindings
- Platform-specific code
- Language idioms
- Error conversion
- API ergonomics

### Adapters
- File system abstraction
- Platform differences
- Storage optimization
- Error mapping

## Cross-Language Types

### Type Mapping
```
Rust String    → TypeScript string    → Python str
Rust Vec<u8>   → TypeScript Uint8Array → Python bytes
Rust Result<T> → TypeScript Promise<T> → Python T | Exception
Rust Option<T> → TypeScript T | null   → Python T | None
```

### Error Handling
- **Rust**: `Result<T, Error>` with custom error types
- **TypeScript**: `Promise<T>` that rejects with Error objects
- **Python**: Raises custom exceptions

### Memory Management
- **Rust**: Compile-time guarantees
- **TypeScript**: GC with manual buffer management
- **Python**: GC with context managers for resources