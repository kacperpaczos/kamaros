# Architektura Portów (Rust Core)

> **Dokumentacja interfejsów warstwy portów w `kamaros-corelib`**

---

## Przegląd

Warstwa portów definiuje **interfejsy (traits)** dla zewnętrznych adapterów. Dzięki temu logika biznesowa (Use Cases) jest niezależna od platformy.

```
                    ┌──────────────────────────────────┐
                    │       Application Layer          │
                    │  (SaveCheckpoint, RestoreVersion)│
                    └──────────────┬───────────────────┘
                                   │ uses
                    ┌──────────────▼───────────────────┐
                    │           Ports Layer            │
                    │  (StoragePort, DiffPort, etc.)   │
                    └──────────────┬───────────────────┘
                                   │ implemented by
                    ┌──────────────▼───────────────────┐
                    │      Infrastructure Layer        │
                    │  (MemoryStorage, SimpleDiff)     │
                    └──────────────────────────────────┘
```

---

## Zdefiniowane Porty

### StoragePort

Abstrakcja operacji I/O systemu plików.

```rust
#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn read(&self, path: &str) -> PortResult<Vec<u8>>;
    async fn write(&self, path: &str, data: &[u8]) -> PortResult<()>;
    async fn delete(&self, path: &str) -> PortResult<()>;
    async fn exists(&self, path: &str) -> PortResult<bool>;
    async fn list(&self, dir: &str) -> PortResult<Vec<String>>;
    async fn size(&self, path: &str) -> PortResult<usize>;
}
```

**Adaptery:**
| Adapter | Użycie | Lokalizacja |
|---------|--------|-------------|
| `MemoryStorage` | Testy jednostkowe | `infrastructure/memory_storage.rs` |
| `NodeFsStorage` | Node.js (via WASM) | Planowane |
| `BrowserStorage` | Browser (IndexedDB) | Planowane |

---

### DiffPort

Generowanie i aplikowanie patchy tekstowych.

```rust
pub trait DiffPort: Send + Sync {
    fn compute_diff(&self, old: &str, new: &str) -> String;
    fn apply_patch(&self, text: &str, patch: &str) -> PortResult<String>;
}
```

**Adaptery:**
| Adapter | Biblioteka | Lokalizacja |
|---------|------------|-------------|
| `SimpleDiff` | `similar` crate | `infrastructure/simple_diff.rs` |

---

### HasherPort

Haszowanie SHA-256 dla Content Addressable Storage.

```rust
pub trait HasherPort: Send + Sync {
    fn hash(&self, data: &[u8]) -> String;
    fn hash_stream(&self, reader: &mut dyn Read) -> PortResult<String>;
}
```

**Adaptery:**
| Adapter | Biblioteka | Lokalizacja |
|---------|------------|-------------|
| `Sha256Hasher` | `sha2` crate | `infrastructure/sha256_hasher.rs` |

---

### CompressorPort

Kompresja/dekompresja ZIP.

```rust
#[async_trait]
pub trait CompressorPort: Send + Sync {
    async fn compress(&self, data: &[u8], level: u32) -> PortResult<Vec<u8>>;
    async fn decompress(&self, data: &[u8]) -> PortResult<Vec<u8>>;
}
```

**Adaptery:** Planowane (wykorzystanie crate `zip`)

---

## Obsługa błędów

Wszystkie porty używają wspólnego typu błędu:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Patch application failed: {0}")]
    PatchFailed(String),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
}

pub type PortResult<T> = Result<T, PortError>;
```

---

## Lokalizacja w kodzie

```
core/src/ports/mod.rs
```

---

[← Powrót do System Overview](01-system-overview.md)
