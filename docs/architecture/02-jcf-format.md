# 📦 Format JCF - Specyfikacja Techniczna

## 1. Wprowadzenie

**JCF (JSON Container Format)** to format pliku oparty na ZIP, zaprojektowany specjalnie dla przechowywania projektów z pełną historią wersji. Format łączy prostotę ZIP z zaawansowanym systemem wersjonowania.

## 2. Podstawowe Założenia

### 2.1 Dlaczego ZIP?

**Zalety**:
- ✅ Uniwersalny - każdy system operacyjny rozumie ZIP
- ✅ Streaming - można tworzyć i odczytywać bez ładowania całości do pamięci
- ✅ Kompresja - wbudowana deflate redukcja rozmiaru
- ✅ Recovery - standardowe narzędzia mogą odzyskać pliki nawet z uszkodzonego archiwum
- ✅ Ecosystem - tysiące bibliotek i narzędzi

**Alternatywy odrzucone**:
- TAR: Brak wbudowanej kompresji, trudniejszy streaming
- Custom binary: Zero tooling, trudny debugging
- SQLite: Overkill, wymaga parsera, nie jest human-readable

### 2.2 Philosophia Designu

1. **Human Recoverable**: W najgorszym przypadku użytkownik może unzipnąć plik zwykłym archiverem
2. **Self-Describing**: `manifest.json` opisuje całą strukturę
3. **Backward Compatible**: Nowe wersje formatu mogą czytać stare pliki
4. **Extensible**: Pole `extra` w manifeście dla custom metadata

## 3. Struktura ZIP

### 3.1 Hierarchia Plików

```
project.jcf (ZIP Archive)
│
├── mimetype                          [UNCOMPRESSED, FIRST]
│   └── "application/x-jcf"
│
├── manifest.json                     [COMPRESSED]
│   └── Metadata + wersje + fileMap
│
├── content/                          [WORKING COPY]
│   ├── src/
│   │   ├── index.js
│   │   ├── utils.js
│   │   └── components/
│   │       └── Button.jsx
│   ├── assets/
│   │   ├── logo.png
│   │   └── icon.svg
│   └── package.json
│
└── .store/                           [HISTORY]
    ├── blobs/                        [Content Addressable Storage]
    │   ├── a3f5e8...2b1c (SHA-256)  [Old/alternate binary files]
    │   ├── 9d4c1e...7f3a
    │   └── ...
    │
    └── deltas/                       [Text Diffs]
        ├── v5_src_index.js.patch     [Reverse patch: v5 → v4]
        ├── v4_src_index.js.patch     [Reverse patch: v4 → v3]
        ├── v3_src_index.js.patch
        └── ...
```

### 3.2 Szczegóły Plików

#### 3.2.1 `/mimetype`

**Wymagania**:
- **MUSI** być pierwszym plikiem w ZIP
- **MUSI** być nieskompresowany (STORE method)
- **MUSI** zawierać dokładnie: `application/x-jcf`
- Brak newline na końcu

**Cel**: Umożliwia szybką identyfikację typu pliku bez parsowania całego ZIP

**Implementacja**:
```typescript
const mimetypeStream = new fflate.ZipPassThrough('mimetype');
zip.add(mimetypeStream);
mimetypeStream.push(
  new TextEncoder().encode('application/x-jcf'),
  true
);
```

**Walidacja**:
```typescript
function validateMimetype(zipData: Uint8Array): boolean {
  // ZIP signature: 50 4B 03 04
  if (zipData[0] !== 0x50 || zipData[1] !== 0x4B) {
    return false;
  }
  
  // Check if first entry is "mimetype" uncompressed
  const filenameLength = zipData[26] | (zipData[27] << 8);
  const filename = new TextDecoder().decode(
    zipData.slice(30, 30 + filenameLength)
  );
  
  return filename === 'mimetype';
}
```

#### 3.2.2 `/manifest.json`

**Struktura**:
```typescript
interface Manifest {
  // Wersja formatu (dla backward compatibility)
  formatVersion: '1.0.0';
  
  // Metadata projektu
  metadata: {
    author: string;
    created_at: string;      // ISO 8601: "2025-12-18T10:30:00.000Z"
    last_modified: string;   // ISO 8601
    application: string;     // "MyApp v2.1.0"
    description?: string;
    tags?: string[];
    extra?: Record<string, unknown>; // Custom fields
  };
  
  // Mapa wszystkich plików projektu
  fileMap: Record<string, FileEntry>;
  
  // Historia wersji (commits)
  versionHistory: Version[];
  
  // Wskaźniki (refs)
  refs: {
    head: string;           // ID bieżącej wersji
    [branchName: string]: string; // Opcjonalne: branching
  };
  
  // Log zmian nazw (dla zachowania historii)
  renameLog: RenameEntry[];
  
  // Opcjonalne: Konfiguracja
  config?: {
    autoGC?: boolean;       // Auto garbage collection
    compressionLevel?: number; // 0-9
    maxHistorySize?: number;   // MB
  };
}
```

**FileEntry**:
```typescript
interface FileEntry {
  type: 'text' | 'binary';
  
  // Unikalny ID pliku (jak Unix inode)
  // Nie zmienia się po rename
  inodeId: string;          // UUID v4
  
  // Dla binary
  currentHash?: string;     // SHA-256 hex
  
  // Encoding
  encoding?: string;        // 'utf-8', 'base64', 'binary'
  
  // Metadata
  created_at?: string;
  modified_at?: string;
  size?: number;            // Bytes
  
  // Custom
  mime?: string;            // 'image/png', 'text/javascript'
  extra?: Record<string, unknown>;
}
```

**Version** (Commit):
```typescript
interface Version {
  id: string;               // UUID v4
  timestamp: string;        // ISO 8601
  message: string;          // Commit message
  author: string;
  email?: string;
  parentId: string | null;  // null dla pierwszego commit
  
  // Snapshot stanu wszystkich plików w tej wersji
  fileStates: Record<string, FileState>;
  
  // Opcjonalne
  tags?: string[];          // ['release', 'stable']
  extra?: Record<string, unknown>;
}
```

**FileState**:
```typescript
interface FileState {
  inodeId: string;          // Link do FileEntry
  path: string;             // Ścieżka w tej wersji
  
  // Dla binary
  hash?: string;            // SHA-256
  
  // Referencja do zawartości
  contentRef?: string;      // ".store/blobs/abc..." lub ".store/deltas/v5_..."
  
  size: number;             // Bytes
  deleted?: boolean;        // Soft delete marker
  
  // Change type (dla UI diff)
  changeType?: 'added' | 'modified' | 'deleted' | 'renamed';
}
```

**RenameEntry**:
```typescript
interface RenameEntry {
  inodeId: string;
  fromPath: string;
  toPath: string;
  versionId: string;        // W której wersji nastąpił rename
  timestamp: string;
}
```

**Przykład manifest.json**:
```json
{
  "formatVersion": "1.0.0",
  "metadata": {
    "author": "Jan Kowalski",
    "created_at": "2025-12-01T10:00:00.000Z",
    "last_modified": "2025-12-18T14:30:00.000Z",
    "application": "JCF Manager v1.0.0",
    "description": "My awesome project"
  },
  "fileMap": {
    "src/index.js": {
      "type": "text",
      "inodeId": "550e8400-e29b-41d4-a716-446655440000",
      "encoding": "utf-8"
    },
    "assets/logo.png": {
      "type": "binary",
      "inodeId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "currentHash": "a3f5e8d9c1b2e4f6...",
      "mime": "image/png",
      "size": 15360
    }
  },
  "versionHistory": [
    {
      "id": "v1",
      "timestamp": "2025-12-01T10:00:00.000Z",
      "message": "Initial commit",
      "author": "Jan Kowalski",
      "parentId": null,
      "fileStates": {
        "src/index.js": {
          "inodeId": "550e8400-e29b-41d4-a716-446655440000",
          "path": "src/index.js",
          "size": 256,
          "changeType": "added"
        }
      }
    },
    {
      "id": "v2",
      "timestamp": "2025-12-18T14:30:00.000Z",
      "message": "Add logo",
      "author": "Jan Kowalski",
      "parentId": "v1",
      "fileStates": {
        "src/index.js": {
          "inodeId": "550e8400-e29b-41d4-a716-446655440000",
          "path": "src/index.js",
          "size": 312,
          "contentRef": ".store/deltas/v1_src_index.js.patch",
          "changeType": "modified"
        },
        "assets/logo.png": {
          "inodeId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
          "path": "assets/logo.png",
          "hash": "a3f5e8d9c1b2e4f6...",
          "contentRef": ".store/blobs/a3f5e8d9c1b2e4f6...",
          "size": 15360,
          "changeType": "added"
        }
      }
    }
  ],
  "refs": {
    "head": "v2"
  },
  "renameLog": []
}
```

#### 3.2.3 `/content/` - Working Copy

**Cel**: Przechowuje **bieżący stan** (HEAD) projektu w pełnej postaci

**Dlaczego to ważne?**:
1. **Recovery**: Można unzipnąć standardowym narzędziem i odzyskać najnowszą wersję
2. **Performance**: Dostęp do HEAD nie wymaga rekonstrukcji z delta
3. **Compatibility**: Zewnętrzne narzędzia mogą czytać pliki bez znajomości JCF

**Struktura**:
```
content/
├── [zachowuje strukturę katalogów użytkownika]
├── src/
│   └── index.js          <- Pełna zawartość z HEAD
├── assets/
│   └── logo.png          <- Aktualna wersja
└── README.md
```

**Compression**: 
- Text files: DEFLATE (standardowa kompresja ZIP)
- Binary files: STORE lub DEFLATE (zależnie od typu - PNG już jest skompresowane)

**Naming**:
- Ścieżki relatywne do root projektu
- Unix-style separatory (`/`) nawet na Windows
- Case-sensitive (nawet na case-insensitive systemach)

#### 3.2.4 `/.store/blobs/` - Content Addressable Storage

**Cel**: Deduplikacja plików binarnych poprzez content addressing

**Naming Convention**: 
```
Filename = SHA-256(file_content)
```

**Przykład**:
```
.store/blobs/
├── a3f5e8d9c1b2e4f67890abcdef123456...  [64 chars hex]
├── 9d4c1e2b3a4f5e6d7c8b9a0f1e2d3c4b...
└── ...
```

**Jak to działa?**:

1. **Save**: 
   ```typescript
   const hash = await sha256(fileContent);
   const blobPath = `.store/blobs/${hash}`;
   
   // Sprawdź czy już istnieje (deduplikacja)
   if (!await blobExists(blobPath)) {
     await writeToZip(blobPath, fileContent);
   }
   
   // W manifeście zapisz tylko hash
   fileEntry.currentHash = hash;
   ```

2. **Load**:
   ```typescript
   const hash = fileEntry.currentHash;
   const blobPath = `.store/blobs/${hash}`;
   const content = await readFromZip(blobPath);
   ```

**Korzyści**:
- Automatyczna deduplikacja (ten sam plik w wielu wersjach = 1 blob)
- Integrity checking (hash weryfikuje zawartość)
- Efficient dla dużych binary files

**Przykład deduplikacji**:
```
v1: logo.png (hash: abc123...)  → .store/blobs/abc123...
v2: logo.png (modified)         → .store/blobs/def456...
v3: logo.png (reverted to v1)  → reuses abc123... (NO NEW BLOB!)
```

#### 3.2.5 `/.store/deltas/` - Text Diffs

**Cel**: Przechowywanie zmian tekstowych jako patche (diffs)

**Naming Convention**:
```
{source_version_id}_{filepath_hash}.patch
```

**Dlaczego hash filepath?**:
- Unika problemów z długimi ścieżkami
- Unika problemów ze znakami specjalnymi
- Consistent length

**Funkcja hashowania ścieżki**:
```typescript
function hashPath(filepath: string): string {
  // Użyj krótkiego hasha (8 chars wystarczy dla collision resistance w projekcie)
  const fullHash = sha256(filepath);
  return fullHash.substring(0, 16); // 16 hex chars = 64 bits
}
```

**Przykład**:
```
.store/deltas/
├── v5_3a2f1b4e5c6d7e8f.patch    [v5 → v4 dla src/index.js]
├── v4_3a2f1b4e5c6d7e8f.patch    [v4 → v3 dla src/index.js]
├── v5_a1b2c3d4e5f6a7b8.patch    [v5 → v4 dla src/utils.js]
└── ...
```

**Format Patch**: Google `diff-match-patch` text format

**Przykład patch**:
```
@@ -1,3 +1,4 @@
 console.log('Hello
+World
 ');
```

**Serializacja**:
```typescript
import { diff_match_patch } from 'diff-match-patch';

const dmp = new diff_match_patch();

// Create patch (NEW → OLD dla reverse delta)
const patches = dmp.patch_make(newText, oldText);
const patchText = dmp.patch_toText(patches);

// Apply patch
const patches2 = dmp.patch_fromText(patchText);
const [restoredText, success] = dmp.patch_apply(patches2, newText);
```

## 4. Reverse Delta Strategy

### 4.1 Koncepcja

**Tradycyjny (Forward) Delta**:
```
v1 (full) → [patch v1→v2] → v2 → [patch v2→v3] → v3 (HEAD)
```
Problem: Aby dostać HEAD, musisz zastosować wszystkie patche od v1

**Reverse Delta (JCF)**:
```
v1 ← [patch v2→v1] ← v2 ← [patch v3→v2] ← v3 (FULL, HEAD)
```
Zaleta: HEAD zawsze pełny, starsze wersje rekonstruowane wstecz

### 4.2 Przykład

**Stan początkowy (v1)**:
```javascript
// src/index.js
console.log('Hello');
```

**Zmiana (v2)**:
```javascript
// src/index.js
console.log('Hello World');
```

**Co jest zapisane**:

1. `/content/src/index.js` (v2, pełny):
   ```javascript
   console.log('Hello World');
   ```

2. `/.store/deltas/v1_3a2f1b4e5c6d7e8f.patch`:
   ```
   @@ -1 +1 @@
   -console.log('Hello World');
   +console.log('Hello');
   ```

**Odtworzenie v1**:
```typescript
const currentText = await readFile('content/src/index.js');
// "console.log('Hello World');"

const patch = await readFile('.store/deltas/v1_3a2f1b4e5c6d7e8f.patch');
const [v1Text] = applyPatch(patch, currentText);
// "console.log('Hello');"
```

### 4.3 Multi-Step Restoration

**Historia**:
```
v1 ← v2 ← v3 ← v4 ← v5 (HEAD)
```

**Odtworzenie v2 z v5**:
```typescript
let text = await readFile('content/src/index.js'); // v5 content

// Apply patches backwards
text = applyPatch(await readPatch('v4_...'), text); // v5 → v4
text = applyPatch(await readPatch('v3_...'), text); // v4 → v3
text = applyPatch(await readPatch('v2_...'), text); // v3 → v2

// Now text contains v2!
```

## 5. Kompresja i Optymalizacja

### 5.1 Poziomy Kompresji

```typescript
interface CompressionPolicy {
  // 0 = STORE (no compression)
  // 1-9 = DEFLATE levels (1=fast, 9=best)
  
  manifest: 6;        // Balance: readable vs size
  textFiles: 6;       // Good compression for code
  images: 0;          // Already compressed (PNG/JPG)
  videos: 0;          // Already compressed
  archives: 0;        // ZIP in ZIP = bad
  deltas: 9;          // Small files, maximize compression
}
```

### 5.2 Strategia per File Type

```typescript
function getCompressionLevel(filepath: string, content: Uint8Array): number {
  const ext = getExtension(filepath);
  
  // Already compressed formats
  if (['.png', '.jpg', '.jpeg', '.gif', '.webp', '.mp4', 
       '.zip', '.gz', '.7z', '.wasm'].includes(ext)) {
    return 0; // STORE
  }
  
  // Text files
  if (['.js', '.ts', '.jsx', '.tsx', '.json', '.txt', 
       '.md', '.css', '.html', '.xml'].includes(ext)) {
    return 6; // DEFLATE
  }
  
  // Binary - test compressibility
  if (isCompressible(content)) {
    return 6;
  }
  
  return 0; // STORE by default
}

function isCompressible(data: Uint8Array): boolean {
  // Quick heuristic: compress first 1KB
  const sample = data.slice(0, 1024);
  const compressed = deflate(sample);
  
  // If compression ratio > 0.9, it's not worth it
  return (compressed.length / sample.length) < 0.9;
}
```

## 6. Walidacja i Integrity

### 6.1 Manifest Schema Validation

```typescript
import Ajv from 'ajv';

const manifestSchema = {
  type: 'object',
  required: ['formatVersion', 'metadata', 'fileMap', 'versionHistory', 'refs'],
  properties: {
    formatVersion: {
      type: 'string',
      pattern: '^\\d+\\.\\d+\\.\\d+$'
    },
    metadata: {
      type: 'object',
      required: ['author', 'created_at', 'last_modified', 'application'],
      properties: {
        author: { type: 'string', minLength: 1 },
        created_at: { type: 'string', format: 'date-time' },
        last_modified: { type: 'string', format: 'date-time' },
        application: { type: 'string' }
      }
    },
    // ... more validation
  }
};

function validateManifest(manifest: unknown): Manifest {
  const ajv = new Ajv();
  const validate = ajv.compile(manifestSchema);
  
  if (!validate(manifest)) {
    throw new ManifestValidationError(validate.errors);
  }
  
  return manifest as Manifest;
}
```

### 6.2 Checksum Verification

**Level 1: ZIP CRC** (built-in)
```typescript
// fflate automatycznie weryfikuje CRC32 przy odczycie
```

**Level 2: SHA-256 dla blobów**
```typescript
async function verifyBlob(blobPath: string): Promise<boolean> {
  const expectedHash = extractHashFromPath(blobPath);
  const content = await readFromZip(blobPath);
  const actualHash = await sha256(content);
  
  if (expectedHash !== actualHash) {
    console.error(`Blob corruption detected: ${blobPath}`);
    return false;
  }
  
  return true;
}
```

**Level 3: Manifest Checksum** (opcjonalny)
```typescript
interface Manifest {
  // ...
  _checksum?: string; // SHA-256 of manifest without this field
}

async function verifyManifest(manifest: Manifest): Promise<boolean> {
  const checksum = manifest._checksum;
  delete manifest._checksum;
  
  const json = JSON.stringify(manifest);
  const expectedHash = await sha256(new TextEncoder().encode(json));
  
  return checksum === expectedHash;
}
```

## 7. Migracje Formatu

### 7.1 Versioning Strategy

**Semantic Versioning** dla `formatVersion`:
- **Major**: Breaking changes (old readers can't read new format)
- **Minor**: Additive changes (old readers can ignore new fields)
- **Patch**: Bug fixes, clarifications

**Przykład**:
```typescript
interface FormatMigration {
  from: string;
  to: string;
  migrate: (manifest: any) => Manifest;
}

const migrations: FormatMigration[] = [
  {
    from: '1.0.0',
    to: '1.1.0',
    migrate: (old) => ({
      ...old,
      config: {
        autoGC: false,
        compressionLevel: 6
      }
    })
  }
];

function migrateManifest(manifest: any): Manifest {
  let current = manifest;
  
  for (const migration of migrations) {
    if (current.formatVersion === migration.from) {
      current = migration.migrate(current);
      current.formatVersion = migration.to;
    }
  }
  
  return current;
}
```

## 8. Best Practices

### 8.1 Do's ✅

1. **Zawsze waliduj manifest po odczycie**
2. **Używaj streaming dla plików >10MB**
3. **Uruchamiaj GC regularnie** (co 50 commitów lub 1GB)
4. **Twórz backup przed saveCheckpoint**
5. **Atomic writes** (temp file + rename)

### 8.2 Don'ts ❌

1. **Nie modyfikuj manifestu ręcznie** (używaj API)
2. **Nie kompresuj już skompresowanych plików**
3. **Nie ładuj całego ZIP do RAM** (streaming!)
4. **Nie używaj długich commit messages** (>1000 chars)
5. **Nie przechowuj secrets** w manifestcie

## 9. Przykłady Użycia

### 9.1 Tworzenie nowego JCF

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

const manager = new JCFManager();
await manager.init(new BrowserAdapter());

// Add files
await manager.addFile('README.md', '# My Project');
await manager.addFile('src/index.js', 'console.log("Hello");');

// First commit
await manager.saveCheckpoint('Initial commit');

// Export
const zipStream = await manager.export();
await saveFile('project.jcf', zipStream);
```

### 9.2 Otwieranie istniejącego JCF

```typescript
const fileData = await loadFile('project.jcf');

const manager = new JCFManager();
await manager.init(new NodeAdapter(), fileData);

// Read manifest
const manifest = manager.getManifest();
console.log(manifest.metadata.author);

// List files
const files = manager.listFiles();
console.log(files);
```

## 10. Podsumowanie

**Format JCF** to:
- ✅ ZIP container (universal compatibility)
- ✅ Self-describing (manifest.json)
- ✅ Reverse delta strategy (fast HEAD access)
- ✅ Content addressable storage (deduplication)
- ✅ Human recoverable (standard unzip)
- ✅ Extensible (extra fields, plugins)

**Następne kroki**:
1. Przeczytaj [Reverse Delta Strategy](./03-reverse-delta.md)
2. Zobacz [CAS Blobs](./04-cas-blobs.md)
3. Sprawdź [API Reference](../api/JCFManager.md)

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

