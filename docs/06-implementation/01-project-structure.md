# Struktura projektu

## Hierarchia folderów

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

## Modularność

### Rust Core
- **Self-contained**: Zero external dependencies for core logic
- **FFI-ready**: Clean interfaces for language bindings
- **Tested**: Comprehensive unit tests

### Language Bindings
- **Thin wrappers**: Minimal logic, delegate to Rust
- **Platform-specific**: Handle platform differences
- **Type-safe**: Strong typing in each language

### Documentation
- **Comprehensive**: All aspects covered
- **Examples**: Working code samples
- **API reference**: Complete type definitions

### Tests
- **Unit**: Individual components
- **Integration**: Full workflows
- **Performance**: Benchmarks and limits
- **Cross-platform**: All supported platforms

### Tools
- **Build**: Automated compilation and bundling
- **Lint**: Code quality enforcement
- **Release**: Version management and publishing