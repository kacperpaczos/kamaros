# 📚 API Reference: JCFManager

## 1. Przegląd

`JCFManager` to główna klasa biblioteki JCF, zapewniająca interfejs do wszystkich operacji na plikach projektowych z wersjonowaniem.

## 2. Import i Inicjalizacja

### 2.1 Import

```typescript
// ES Modules
import { JCFManager, BrowserAdapter, NodeAdapter } from 'jcf-manager';

// CommonJS
const { JCFManager, BrowserAdapter } = require('jcf-manager');
```

### 2.2 Konstruktor

```typescript
constructor(config?: JCFConfig)
```

**Parametry**:
- `config` (opcjonalny): Konfiguracja managera

**Przykład**:
```typescript
const manager = new JCFManager({
  author: 'Jan Kowalski',
  compressionLevel: 6,
  autoGC: true
});
```

### 2.3 Konfiguracja

```typescript
interface JCFConfig {
  /**
   * Domyślny autor commitów
   */
  author?: string;
  
  /**
   * Email autora
   */
  email?: string;
  
  /**
   * Poziom kompresji (0-9)
   * 0 = STORE (bez kompresji)
   * 9 = MAX (najlepsza kompresja)
   * Default: 6
   */
  compressionLevel?: number;
  
  /**
   * Automatyczny garbage collection po każdym N commitów
   * Default: false
   */
  autoGC?: boolean;
  
  /**
   * Interwał snapshotów (co ile wersji tworzyć pełny snapshot)
   * Default: 50
   */
  snapshotInterval?: number;
  
  /**
   * Maksymalny rozmiar pliku (bytes)
   * Default: 500MB w przeglądarce, unlimited w Node.js
   */
  maxFileSize?: number;
  
  /**
   * Czy używać Web Workers
   * Default: true (jeśli dostępne)
   */
  useWorkers?: boolean;
  
  /**
   * Liczba workerów
   * Default: navigator.hardwareConcurrency || 4
   */
  workerCount?: number;
}
```

## 3. Metody Główne

### 3.1 init()

Inicjalizuje managera z adapterem systemu plików.

```typescript
async init(
  adapter: FileSystemAdapter,
  source?: Uint8Array | ReadableStream
): Promise<void>
```

**Parametry**:
- `adapter`: Adapter dla platformy (BrowserAdapter, NodeAdapter, TauriAdapter)
- `source` (opcjonalny): Istniejący plik JCF do otwarcia

**Przykład**:
```typescript
// Nowy projekt
const manager = new JCFManager();
await manager.init(new BrowserAdapter());

// Otwarcie istniejącego
const fileData = await loadFile('project.jcf');
await manager.init(new NodeAdapter(), fileData);
```

**Throws**:
- `AdapterInitError`: Jeśli inicjalizacja adaptera się nie powiodła
- `InvalidJCFError`: Jeśli source nie jest prawidłowym plikiem JCF

---

### 3.2 addFile()

Dodaje lub aktualizuje plik w projekcie.

```typescript
async addFile(
  path: string,
  content: string | Uint8Array | Blob | ReadableStream,
  metadata?: FileMetadata
): Promise<void>
```

**Parametry**:
- `path`: Ścieżka relatywna (Unix-style, np. `src/index.js`)
- `content`: Zawartość pliku
  - `string`: Tekst (automatycznie wykryty jako text)
  - `Uint8Array`: Binarny
  - `Blob`: Browser File/Blob
  - `ReadableStream`: Dla dużych plików
- `metadata` (opcjonalny): Dodatkowe metadane

**Przykład**:
```typescript
// Text file
await manager.addFile('README.md', '# My Project');

// Binary file
await manager.addFile('logo.png', pngData);

// From File input
const file = fileInput.files[0];
await manager.addFile(`assets/${file.name}`, file);

// Streaming large file
const stream = getLargeFileStream();
await manager.addFile('video.mp4', stream);
```

**Throws**:
- `InvalidPathError`: Jeśli ścieżka zawiera nieprawidłowe znaki
- `FileTooLargeError`: Jeśli plik przekracza `maxFileSize`

---

### 3.3 getFile()

Pobiera zawartość pliku.

```typescript
async getFile(
  path: string,
  versionId?: string
): Promise<Uint8Array>
```

**Parametry**:
- `path`: Ścieżka do pliku
- `versionId` (opcjonalny): ID wersji (domyślnie: HEAD)

**Przykład**:
```typescript
// Bieżąca wersja
const content = await manager.getFile('src/index.js');
const text = new TextDecoder().decode(content);

// Konkretna wersja
const oldContent = await manager.getFile('src/index.js', 'v1');
```

**Throws**:
- `FileNotFoundError`: Jeśli plik nie istnieje
- `VersionNotFoundError`: Jeśli wersja nie istnieje

---

### 3.4 getFileStream()

Pobiera zawartość pliku jako stream (dla dużych plików).

```typescript
getFileStream(
  path: string,
  versionId?: string
): ReadableStream
```

**Parametry**:
- `path`: Ścieżka do pliku
- `versionId` (opcjonalny): ID wersji

**Przykład**:
```typescript
// Stream large file
const stream = manager.getFileStream('video.mp4');

// Pipe to download
const response = new Response(stream);
const blob = await response.blob();
saveAs(blob, 'video.mp4');
```

---

### 3.5 deleteFile()

Usuwa plik z projektu (soft delete - zachowuje w historii).

```typescript
async deleteFile(path: string): Promise<void>
```

**Parametry**:
- `path`: Ścieżka do pliku

**Przykład**:
```typescript
await manager.deleteFile('old-file.txt');
```

**Uwaga**: Plik nie jest fizycznie usunięty do następnego GC. Jest tylko oznaczony jako deleted w manifeście.

---

### 3.6 moveFile()

Zmienia nazwę/przenosi plik (zachowuje historię).

```typescript
async moveFile(
  oldPath: string,
  newPath: string
): Promise<void>
```

**Parametry**:
- `oldPath`: Aktualna ścieżka
- `newPath`: Nowa ścieżka

**Przykład**:
```typescript
// Rename
await manager.moveFile('old-name.js', 'new-name.js');

// Move to different directory
await manager.moveFile('file.js', 'src/file.js');
```

**Uwaga**: Operacja używa systemu inode, więc historia pliku jest zachowana po rename.

---

### 3.7 listFiles()

Zwraca listę wszystkich plików w projekcie.

```typescript
listFiles(versionId?: string): Promise<FileInfo[]>
```

**Parametry**:
- `versionId` (opcjonalny): ID wersji (domyślnie: HEAD)

**Return**:
```typescript
interface FileInfo {
  path: string;
  type: 'text' | 'binary';
  size: number;
  hash?: string; // Dla binary
  modified?: Date;
}
```

**Przykład**:
```typescript
const files = await manager.listFiles();

for (const file of files) {
  console.log(`${file.path} (${formatBytes(file.size)})`);
}

// Filter text files
const textFiles = files.filter(f => f.type === 'text');
```

---

## 4. Wersjonowanie

### 4.1 saveCheckpoint()

Tworzy nowy checkpoint (commit) z bieżącymi zmianami.

```typescript
async saveCheckpoint(
  message: string,
  author?: string
): Promise<string>
```

**Parametry**:
- `message`: Opis zmian
- `author` (opcjonalny): Nadpisuje domyślnego autora

**Return**: ID nowej wersji (UUID v4)

**Przykład**:
```typescript
// Basic usage
const versionId = await manager.saveCheckpoint('Add login feature');

// Custom author
const v2 = await manager.saveCheckpoint(
  'Fix bug',
  'Anna Kowalska <anna@example.com>'
);

console.log(`Saved as version: ${versionId}`);
```

**Throws**:
- `NoChangesError`: Jeśli nie ma żadnych zmian do zapisania
- `StorageError`: Jeśli zapis się nie powiódł

---

### 4.2 restoreVersion()

Przywraca projekt do określonej wersji (time travel).

```typescript
async restoreVersion(versionId: string): Promise<void>
```

**Parametry**:
- `versionId`: ID wersji do przywrócenia

**Przykład**:
```typescript
// Save current state
const v1 = await manager.saveCheckpoint('Version 1');

// Make changes
await manager.addFile('new.txt', 'content');
await manager.saveCheckpoint('Version 2');

// Time travel back
await manager.restoreVersion(v1);

// Now we're back at v1 state
const exists = await manager.fileExists('new.txt');
console.log(exists); // false
```

**Throws**:
- `VersionNotFoundError`: Jeśli wersja nie istnieje
- `RestoreError`: Jeśli przywracanie się nie powiodło

**Uwaga**: To jest destructive operation - bieżące niezapisane zmiany zostaną utracone!

---

### 4.3 getVersionHistory()

Zwraca pełną historię wersji.

```typescript
getVersionHistory(): Version[]
```

**Return**:
```typescript
interface Version {
  id: string;
  timestamp: string; // ISO 8601
  message: string;
  author: string;
  parentId: string | null;
  fileStates: Record<string, FileState>;
}
```

**Przykład**:
```typescript
const history = manager.getVersionHistory();

for (const version of history) {
  console.log(`${version.id}: ${version.message}`);
  console.log(`  By ${version.author} at ${version.timestamp}`);
}

// Get latest 10 versions
const recent = history.slice(-10).reverse();
```

---

### 4.4 getFileHistory()

Zwraca historię zmian konkretnego pliku.

```typescript
async getFileHistory(filePath: string): Promise<FileHistoryEntry[]>
```

**Parametry**:
- `filePath`: Ścieżka do pliku

**Return**:
```typescript
interface FileHistoryEntry {
  versionId: string;
  timestamp: string;
  message: string;
  author: string;
  changeType: 'added' | 'modified' | 'deleted' | 'renamed';
  path: string; // Może się różnić jeśli był rename
  size: number;
}
```

**Przykład**:
```typescript
const history = await manager.getFileHistory('src/index.js');

console.log(`History of src/index.js:`);
for (const entry of history) {
  console.log(`  ${entry.versionId}: ${entry.changeType}`);
  console.log(`    ${entry.message}`);
  
  if (entry.changeType === 'renamed') {
    console.log(`    Path was: ${entry.path}`);
  }
}
```

---

### 4.5 compareVersions()

Porównuje dwie wersje i zwraca diff.

```typescript
async compareVersions(
  versionId1: string,
  versionId2: string
): Promise<VersionDiff>
```

**Return**:
```typescript
interface VersionDiff {
  added: string[];      // Nowe pliki
  modified: string[];   // Zmienione pliki
  deleted: string[];    // Usunięte pliki
  renamed: Array<{      // Zmiana nazwy
    from: string;
    to: string;
  }>;
}
```

**Przykład**:
```typescript
const diff = await manager.compareVersions('v1', 'v5');

console.log(`Changes from v1 to v5:`);
console.log(`  Added: ${diff.added.length} files`);
console.log(`  Modified: ${diff.modified.length} files`);
console.log(`  Deleted: ${diff.deleted.length} files`);

// Show details
for (const file of diff.modified) {
  console.log(`  Modified: ${file}`);
}
```

---

## 5. Maintenance

### 5.1 runGC()

Uruchamia garbage collection (usuwa nieużywane bloby).

```typescript
async runGC(options?: GCOptions): Promise<GCReport>
```

**Parametry**:
```typescript
interface GCOptions {
  /**
   * Okres grace (dni) - nie usuwa blobów młodszych niż X dni
   */
  gracePeriodDays?: number;
  
  /**
   * Czy pokazywać progress
   */
  showProgress?: boolean;
}
```

**Return**:
```typescript
interface GCReport {
  blobsRemoved: number;
  deltasRemoved: number;
  spaceFreed: number; // bytes
  duration: number; // ms
}
```

**Przykład**:
```typescript
// Basic GC
const report = await manager.runGC();
console.log(`Freed ${formatBytes(report.spaceFreed)}`);
console.log(`Removed ${report.blobsRemoved} blobs`);

// Safe GC with grace period
const safeReport = await manager.runGC({
  gracePeriodDays: 7
});
```

**Uwaga**: GC może być czasochłonne dla dużych projektów (>1000 wersji).

---

### 5.2 verify()

Weryfikuje integralność pliku JCF.

```typescript
async verify(): Promise<VerificationReport>
```

**Return**:
```typescript
interface VerificationReport {
  valid: boolean;
  errors: VerificationError[];
  warnings: VerificationWarning[];
}

interface VerificationError {
  type: 'missing_blob' | 'corrupt_blob' | 'invalid_manifest' | 'broken_chain';
  message: string;
  details?: any;
}
```

**Przykład**:
```typescript
const report = await manager.verify();

if (!report.valid) {
  console.error('⚠️  JCF file has errors:');
  for (const error of report.errors) {
    console.error(`  - ${error.type}: ${error.message}`);
  }
} else {
  console.log('✅ JCF file is valid');
}

if (report.warnings.length > 0) {
  console.warn('Warnings:');
  for (const warning of report.warnings) {
    console.warn(`  - ${warning.message}`);
  }
}
```

---

### 5.3 getStats()

Zwraca statystyki projektu.

```typescript
async getStats(): Promise<ProjectStats>
```

**Return**:
```typescript
interface ProjectStats {
  // General
  totalVersions: number;
  totalFiles: number;
  totalSize: number;
  
  // Storage breakdown
  contentSize: number;
  blobsSize: number;
  deltasSize: number;
  manifestSize: number;
  
  // Deduplication
  uniqueBlobs: number;
  blobReferences: number;
  deduplicationRatio: number;
  
  // History
  oldestVersion: {
    id: string;
    timestamp: string;
  };
  newestVersion: {
    id: string;
    timestamp: string;
  };
  
  // Files
  filesByType: {
    text: number;
    binary: number;
  };
  largestFile: {
    path: string;
    size: number;
  };
}
```

**Przykład**:
```typescript
const stats = await manager.getStats();

console.log('📊 Project Statistics');
console.log(`Total versions: ${stats.totalVersions}`);
console.log(`Total files: ${stats.totalFiles}`);
console.log(`Total size: ${formatBytes(stats.totalSize)}`);
console.log(`Deduplication ratio: ${stats.deduplicationRatio.toFixed(2)}x`);
console.log(`Space saved: ${formatBytes(stats.totalSize - stats.blobsSize)}`);
```

---

## 6. Export/Import

### 6.1 export()

Eksportuje projekt jako stream (do zapisu lub przesłania).

```typescript
async export(): Promise<ReadableStream>
```

**Przykład**:
```typescript
// Save to file (Browser)
const stream = await manager.export();
const response = new Response(stream);
const blob = await response.blob();
saveAs(blob, 'project.jcf');

// Save to file (Node.js)
const stream = await manager.export();
const writeStream = createWriteStream('project.jcf');
Readable.fromWeb(stream).pipe(writeStream);

// Upload to server
const stream = await manager.export();
await fetch('/api/upload', {
  method: 'POST',
  body: stream,
  headers: {
    'Content-Type': 'application/x-jcf'
  }
});
```

---

### 6.2 exportSnapshot()

Eksportuje snapshot konkretnej wersji (bez historii).

```typescript
async exportSnapshot(
  versionId?: string,
  format?: 'zip' | 'tar'
): Promise<ReadableStream>
```

**Parametry**:
- `versionId` (opcjonalny): Wersja do eksportu (domyślnie: HEAD)
- `format` (opcjonalny): Format archiwum (domyślnie: 'zip')

**Przykład**:
```typescript
// Export current state as clean ZIP
const snapshot = await manager.exportSnapshot();
saveAs(await new Response(snapshot).blob(), 'project-snapshot.zip');

// Export specific version
const v1Snapshot = await manager.exportSnapshot('v1');
```

**Uwaga**: Snapshot nie zawiera historii - tylko pliki z określonej wersji.

---

## 7. Events

### 7.1 Obserwowanie Zmian

```typescript
manager.on('change', (event: ChangeEvent) => {
  console.log(`File changed: ${event.path}`);
});

manager.on('checkpoint', (event: CheckpointEvent) => {
  console.log(`Checkpoint created: ${event.versionId}`);
});

manager.on('restore', (event: RestoreEvent) => {
  console.log(`Restored to: ${event.versionId}`);
});

manager.on('error', (error: Error) => {
  console.error('Error:', error);
});
```

**Typy Eventów**:
```typescript
interface ChangeEvent {
  path: string;
  type: 'added' | 'modified' | 'deleted';
  timestamp: string;
}

interface CheckpointEvent {
  versionId: string;
  message: string;
  filesChanged: number;
  timestamp: string;
}

interface RestoreEvent {
  versionId: string;
  previousVersionId: string;
  timestamp: string;
}
```

---

## 8. Utility Methods

### 8.1 fileExists()

```typescript
async fileExists(path: string, versionId?: string): Promise<boolean>
```

### 8.2 getManifest()

```typescript
getManifest(): Manifest
```

### 8.3 dispose()

```typescript
async dispose(): Promise<void>
```

**Przykład**:
```typescript
// Cleanup
await manager.dispose();
```

**Uwaga**: Zawsze wywołaj `dispose()` przed zakończeniem aplikacji (zamyka połączenia, workery, etc.)

---

## 9. Error Handling

### 9.1 Typy Błędów

```typescript
class JCFError extends Error {}

class InvalidPathError extends JCFError {}
class FileNotFoundError extends JCFError {}
class VersionNotFoundError extends JCFError {}
class FileTooLargeError extends JCFError {}
class StorageError extends JCFError {}
class ManifestCorruptionError extends JCFError {}
class BlobCorruptionError extends JCFError {}
```

### 9.2 Przykład

```typescript
try {
  await manager.addFile('test.txt', content);
} catch (error) {
  if (error instanceof FileTooLargeError) {
    console.error('File too large!');
  } else if (error instanceof StorageError) {
    console.error('Storage error:', error.message);
  } else {
    console.error('Unknown error:', error);
  }
}
```

---

## 10. TypeScript Types

### 10.1 Import Types

```typescript
import type {
  JCFConfig,
  FileInfo,
  Version,
  VersionDiff,
  ProjectStats,
  GCReport,
  FileMetadata
} from 'jcf-manager';
```

---

## 11. Best Practices

### 11.1 Do's ✅

1. **Zawsze wywołuj `init()`** przed użyciem
2. **Używaj `try-catch`** dla async operacji
3. **Call `dispose()`** przed exit
4. **Używaj streaming** dla plików >50MB
5. **Regularnie uruchamiaj GC**

### 11.2 Don'ts ❌

1. **Nie modify manifestu** bezpośrednio
2. **Nie używaj długich ścieżek** (>255 chars)
3. **Nie load wszystkich wersji** naraz
4. **Nie ignore errors**
5. **Nie używaj synchronicznych operacji**

---

## 12. Przykłady Użycia

### 12.1 Complete Workflow

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

async function main() {
  // 1. Inicjalizacja
  const manager = new JCFManager({
    author: 'Jan Kowalski',
    compressionLevel: 6
  });
  await manager.init(new BrowserAdapter());
  
  // 2. Dodaj pliki
  await manager.addFile('README.md', '# My Project');
  await manager.addFile('src/index.js', 'console.log("Hello");');
  
  // 3. Pierwszy commit
  const v1 = await manager.saveCheckpoint('Initial commit');
  console.log(`Created v1: ${v1}`);
  
  // 4. Edycja
  await manager.addFile('src/index.js', 'console.log("Hello World");');
  await manager.addFile('src/utils.js', 'export const add = (a,b) => a+b;');
  
  // 5. Drugi commit
  const v2 = await manager.saveCheckpoint('Add utils');
  
  // 6. Historia
  const history = manager.getVersionHistory();
  console.log(`Total versions: ${history.length}`);
  
  // 7. Time travel
  await manager.restoreVersion(v1);
  console.log('Restored to v1');
  
  // 8. Export
  const stream = await manager.export();
  saveAs(await new Response(stream).blob(), 'project.jcf');
  
  // 9. Cleanup
  await manager.dispose();
}

main().catch(console.error);
```

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

