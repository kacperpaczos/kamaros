# Konwencje nazewnictwa

## Zasady ogólne

- **Consistency**: Ta sama konwencja w całym języku
- **Readability**: Nazwy opisowe i zrozumiałe
- **Tooling**: Compatible z standardowymi narzędziami

## Rust (snake_case)

### Moduły
```rust
mod jcf_format;
mod time_travel_versioning;
mod content_addressable_storage;
```

### Funkcje i metody
```rust
fn save_checkpoint(message: &str) -> Result<VersionId, Error>
fn restore_version(version_id: &VersionId) -> Result<(), Error>
fn compute_diff(old_content: &str, new_content: &str) -> Vec<Patch>
```

### Typy i struktury
```rust
struct JCFManager;
struct VersionHistory;
struct FileEntry;
struct ContentAddressableStorage;
```

### Stałe
```rust
const DEFAULT_COMPRESSION_LEVEL: u32 = 6;
const MANIFEST_FILENAME: &str = "manifest.json";
```

## TypeScript (camelCase)

### Klasy i interfejsy
```typescript
class JCFManager
class VersionManager
class FileManager
class DeltaManager

interface Manifest
interface FileEntry
interface Version
interface FileSystemAdapter
```

### Metody i funkcje
```typescript
saveCheckpoint(message: string): Promise<string>
restoreVersion(versionId: string): Promise<void>
computeDiff(oldContent: string, newContent: string): Patch[]
getFileContent(path: string): Promise<Uint8Array>
```

### Właściwości i zmienne
```typescript
private adapter: FileSystemAdapter;
public versionHistory: Version[];
const defaultCompressionLevel = 6;
```

## Python (snake_case)

### Wszystko snake_case
```python
class jcf_manager:
    def save_checkpoint(self, message: str) -> str:
        pass

    def restore_version(self, version_id: str) -> None:
        pass

def compute_diff(old_content: str, new_content: str) -> list:
    pass
```

## Specyficzne dla domeny

### Wersjonowanie
- `version_id`, `versionId`, `versionId`
- `parent_id`, `parentId`, `parent_id`
- `head_ref`, `headRef`, `head_ref`

### Pliki i ścieżki
- `file_path`, `filePath`, `file_path`
- `file_content`, `fileContent`, `file_content`
- `file_entry`, `fileEntry`, `fileEntry`

### Hash i CAS
- `content_hash`, `contentHash`, `content_hash`
- `blob_id`, `blobId`, `blob_id`
- `cas_store`, `casStore`, `cas_store`

### Diff i patch
- `diff_patch`, `diffPatch`, `diff_patch`
- `reverse_delta`, `reverseDelta`, `reverse_delta`
- `patch_text`, `patchText`, `patch_text`

## TypeScript specific

### Generic types
```typescript
Map<string, FileEntry>
Promise<Version[]>
Result<T, Error>
```

### Enum values
```typescript
enum FileType {
  Text = 'text',
  Binary = 'binary'
}
```

### Event names
```typescript
'checkpoint:start'
'checkpoint:progress'
'checkpoint:complete'
'checkpoint:error'
```

## Rust specific

### Error types
```rust
#[derive(Debug, thiserror::Error)]
pub enum JCFError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid manifest: {reason}")]
    InvalidManifest { reason: String },
}
```

### Macro usage
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    // ...
}
```

## Cross-language consistency

### API mapping
```typescript
// TypeScript
saveCheckpoint(message: string): Promise<string>

// Maps to Rust
fn save_checkpoint(message: &str) -> Result<String, Error>

// Maps to Python
def save_checkpoint(self, message: str) -> str:
```

### Type mapping
- `string` (TS) ↔ `String` (Rust) ↔ `str` (Python)
- `number` (TS) ↔ `f64` (Rust) ↔ `float` (Python)
- `Uint8Array` (TS) ↔ `Vec<u8>` (Rust) ↔ `bytes` (Python)
- `Promise<T>` (TS) ↔ `Result<T, E>` (Rust) ↔ `T | Exception` (Python)