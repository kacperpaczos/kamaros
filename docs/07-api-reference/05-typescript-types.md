# 📘 TypeScript Types & Interfaces - Dokumentacja Typów

## 1. Przegląd

Ten dokument zawiera wszystkie publiczne typy i interfejsy używane w JCF Manager.

## 2. Core Types

### 2.1 Manifest

```typescript
/**
 * Główna struktura metadanych projektu JCF
 */
interface Manifest {
  /**
   * Wersja formatu JCF (Semantic Versioning)
   */
  formatVersion: string; // np. "1.0.0"
  
  /**
   * Metadata projektu
   */
  metadata: ProjectMetadata;
  
  /**
   * Mapa wszystkich plików w projekcie
   */
  fileMap: Record<string, FileEntry>;
  
  /**
   * Pełna historia wersji (commits)
   */
  versionHistory: Version[];
  
  /**
   * Wskaźniki do ważnych wersji
   */
  refs: {
    head: string; // ID bieżącej wersji
    [branchName: string]: string; // Opcjonalne: inne branche
  };
  
  /**
   * Log zmian nazw plików (tracking rename)
   */
  renameLog: RenameEntry[];
  
  /**
   * Opcjonalna konfiguracja
   */
  config?: ManifestConfig;
}
```

### 2.2 ProjectMetadata

```typescript
interface ProjectMetadata {
  /**
   * Autor projektu
   */
  author: string;
  
  /**
   * Email autora (opcjonalny)
   */
  email?: string;
  
  /**
   * Data utworzenia projektu (ISO 8601)
   */
  created_at: string;
  
  /**
   * Data ostatniej modyfikacji (ISO 8601)
   */
  last_modified: string;
  
  /**
   * Nazwa aplikacji która stworzyła projekt
   */
  application: string;
  
  /**
   * Opis projektu
   */
  description?: string;
  
  /**
   * Tagi/słowa kluczowe
   */
  tags?: string[];
  
  /**
   * Pole do custom metadata
   */
  extra?: Record<string, unknown>;
}
```

### 2.3 FileEntry

```typescript
/**
 * Wpis w fileMap - reprezentuje plik w projekcie
 */
interface FileEntry {
  /**
   * Typ pliku
   */
  type: 'text' | 'binary';
  
  /**
   * Unikalny ID pliku (symulacja Unix inode)
   * Nie zmienia się po rename
   */
  inodeId: string; // UUID v4
  
  /**
   * Dla plików binarnych: hash SHA-256 aktualnej zawartości
   */
  currentHash?: string;
  
  /**
   * Encoding (dla plików tekstowych)
   */
  encoding?: string; // 'utf-8', 'base64', etc.
  
  /**
   * Data utworzenia
   */
  created_at?: string; // ISO 8601
  
  /**
   * Data ostatniej modyfikacji
   */
  modified_at?: string; // ISO 8601
  
  /**
   * Rozmiar w bajtach
   */
  size?: number;
  
  /**
   * MIME type
   */
  mime?: string; // 'text/javascript', 'image/png', etc.
  
  /**
   * Custom metadata
   */
  extra?: Record<string, unknown>;
}
```

### 2.4 Version

```typescript
/**
 * Reprezentacja pojedynczej wersji (commit)
 */
interface Version {
  /**
   * Unikalny ID wersji
   */
  id: string; // UUID v4
  
  /**
   * Timestamp utworzenia (ISO 8601)
   */
  timestamp: string;
  
  /**
   * Wiadomość commit
   */
  message: string;
  
  /**
   * Autor commita
   */
  author: string;
  
  /**
   * Email autora
   */
  email?: string;
  
  /**
   * ID wersji rodzica (null dla pierwszego commit)
   */
  parentId: string | null;
  
  /**
   * Snapshot stanu wszystkich plików w tej wersji
   */
  fileStates: Record<string, FileState>;
  
  /**
   * Tagi tej wersji (np. 'release', 'stable')
   */
  tags?: string[];
  
  /**
   * Custom metadata
   */
  extra?: Record<string, unknown>;
}
```

### 2.5 FileState

```typescript
/**
 * Stan pliku w konkretnej wersji
 */
interface FileState {
  /**
   * ID pliku (link do FileEntry)
   */
  inodeId: string;
  
  /**
   * Ścieżka pliku w tej wersji
   */
  path: string;
  
  /**
   * Hash SHA-256 (dla plików binarnych)
   */
  hash?: string;
  
  /**
   * Referencja do zawartości
   * Może wskazywać na:
   * - `.store/blobs/{hash}` dla binarnych
   * - `.store/deltas/{version}_{path_hash}.patch` dla tekstowych
   */
  contentRef?: string;
  
  /**
   * Rozmiar pliku w bajtach
   */
  size: number;
  
  /**
   * Czy plik jest usunięty w tej wersji (soft delete)
   */
  deleted?: boolean;
  
  /**
   * Typ zmiany (dla UI/diff)
   */
  changeType?: 'added' | 'modified' | 'deleted' | 'renamed';
}
```

### 2.6 RenameEntry

```typescript
/**
 * Wpis w logu zmian nazw
 */
interface RenameEntry {
  /**
   * ID pliku
   */
  inodeId: string;
  
  /**
   * Ścieżka źródłowa
   */
  fromPath: string;
  
  /**
   * Ścieżka docelowa
   */
  toPath: string;
  
  /**
   * ID wersji w której nastąpił rename
   */
  versionId: string;
  
  /**
   * Timestamp operacji (ISO 8601)
   */
  timestamp: string;
}
```

## 3. Configuration Types

### 3.1 JCFConfig

```typescript
/**
 * Konfiguracja JCFManager
 */
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
   * Poziom kompresji ZIP (0-9)
   * 0 = STORE (bez kompresji)
   * 9 = MAX (najlepsza kompresja, wolniejsze)
   * Default: 6
   */
  compressionLevel?: number;
  
  /**
   * Czy automatycznie uruchamiać GC
   */
  autoGC?: boolean;
  
  /**
   * Co ile commitów tworzyć pełne snapshoty
   * Default: 50
   */
  snapshotInterval?: number;
  
  /**
   * Maksymalny rozmiar pojedynczego pliku (bytes)
   * Default: 500MB w przeglądarce, unlimited w Node.js
   */
  maxFileSize?: number;
  
  /**
   * Czy używać Web Workers
   * Default: true (jeśli dostępne)
   */
  useWorkers?: boolean;
  
  /**
   * Liczba workerów w puli
   * Default: navigator.hardwareConcurrency || 4
   */
  workerCount?: number;
  
  /**
   * Timeout dla operacji (ms)
   * Default: 30000 (30s)
   */
  operationTimeout?: number;
}
```

### 3.2 ManifestConfig

```typescript
/**
 * Konfiguracja przechowywana w manifest.json
 */
interface ManifestConfig {
  /**
   * Czy włączyć auto GC
   */
  autoGC?: boolean;
  
  /**
   * Poziom kompresji
   */
  compressionLevel?: number;
  
  /**
   * Maksymalny rozmiar historii (MB)
   * Po przekroczeniu najstarsze wersje są archiwizowane
   */
  maxHistorySize?: number;
  
  /**
   * Custom settings
   */
  [key: string]: unknown;
}
```

## 4. Operation Types

### 4.1 FileInfo

```typescript
/**
 * Informacje o pliku zwracane przez listFiles()
 */
interface FileInfo {
  /**
   * Ścieżka pliku
   */
  path: string;
  
  /**
   * Typ pliku
   */
  type: 'text' | 'binary';
  
  /**
   * Rozmiar w bajtach
   */
  size: number;
  
  /**
   * Hash SHA-256 (tylko dla binary)
   */
  hash?: string;
  
  /**
   * Data ostatniej modyfikacji
   */
  modified?: Date;
  
  /**
   * MIME type
   */
  mime?: string;
}
```

### 4.2 VersionDiff

```typescript
/**
 * Różnice między dwoma wersjami
 */
interface VersionDiff {
  /**
   * Pliki dodane
   */
  added: string[];
  
  /**
   * Pliki zmodyfikowane
   */
  modified: string[];
  
  /**
   * Pliki usunięte
   */
  deleted: string[];
  
  /**
   * Pliki z zmienioną nazwą
   */
  renamed: Array<{
    from: string;
    to: string;
  }>;
}
```

### 4.3 FileHistoryEntry

```typescript
/**
 * Wpis w historii pojedynczego pliku
 */
interface FileHistoryEntry {
  /**
   * ID wersji
   */
  versionId: string;
  
  /**
   * Timestamp (ISO 8601)
   */
  timestamp: string;
  
  /**
   * Wiadomość commit
   */
  message: string;
  
  /**
   * Autor zmiany
   */
  author: string;
  
  /**
   * Typ zmiany
   */
  changeType: 'added' | 'modified' | 'deleted' | 'renamed';
  
  /**
   * Ścieżka pliku w tej wersji
   * (może być inna niż obecna jeśli był rename)
   */
  path: string;
  
  /**
   * Rozmiar pliku
   */
  size: number;
}
```

### 4.4 ProjectStats

```typescript
/**
 * Statystyki projektu
 */
interface ProjectStats {
  // === Ogólne ===
  
  /**
   * Całkowita liczba wersji
   */
  totalVersions: number;
  
  /**
   * Całkowita liczba plików (w HEAD)
   */
  totalFiles: number;
  
  /**
   * Całkowity rozmiar projektu (bytes)
   */
  totalSize: number;
  
  // === Breakdown pamięci ===
  
  /**
   * Rozmiar /content/ (working copy)
   */
  contentSize: number;
  
  /**
   * Rozmiar .store/blobs/
   */
  blobsSize: number;
  
  /**
   * Rozmiar .store/deltas/
   */
  deltasSize: number;
  
  /**
   * Rozmiar manifest.json
   */
  manifestSize: number;
  
  // === Deduplikacja ===
  
  /**
   * Liczba unikalnych blobów
   */
  uniqueBlobs: number;
  
  /**
   * Całkowita liczba referencji do blobów
   */
  blobReferences: number;
  
  /**
   * Współczynnik deduplikacji
   * (blobReferences / uniqueBlobs)
   * Wyższy = więcej oszczędności
   */
  deduplicationRatio: number;
  
  // === Historia ===
  
  /**
   * Najstarsza wersja
   */
  oldestVersion: {
    id: string;
    timestamp: string;
  };
  
  /**
   * Najnowsza wersja (HEAD)
   */
  newestVersion: {
    id: string;
    timestamp: string;
  };
  
  // === Pliki ===
  
  /**
   * Liczba plików per typ
   */
  filesByType: {
    text: number;
    binary: number;
  };
  
  /**
   * Największy plik
   */
  largestFile: {
    path: string;
    size: number;
  };
}
```

### 4.5 GCReport

```typescript
/**
 * Raport z garbage collection
 */
interface GCReport {
  /**
   * Liczba usuniętych blobów
   */
  blobsRemoved: number;
  
  /**
   * Liczba usuniętych delt
   */
  deltasRemoved: number;
  
  /**
   * Zwolniona przestrzeń (bytes)
   */
  spaceFreed: number;
  
  /**
   * Czas trwania operacji (ms)
   */
  duration: number;
  
  /**
   * Bloby w grace period (nie usunięte)
   */
  blobsInGracePeriod?: number;
}
```

### 4.6 GCOptions

```typescript
/**
 * Opcje dla garbage collection
 */
interface GCOptions {
  /**
   * Okres grace - nie usuwa blobów młodszych niż X dni
   * Default: 0 (usuwa wszystkie orphaned)
   */
  gracePeriodDays?: number;
  
  /**
   * Czy pokazywać progress
   */
  showProgress?: boolean;
  
  /**
   * Callback dla progress updates
   */
  onProgress?: (progress: GCProgress) => void;
}

interface GCProgress {
  phase: 'marking' | 'sweeping' | 'compacting';
  current: number;
  total: number;
  percent: number;
}
```

### 4.7 VerificationReport

```typescript
/**
 * Raport weryfikacji integralności
 */
interface VerificationReport {
  /**
   * Czy plik JCF jest prawidłowy
   */
  valid: boolean;
  
  /**
   * Lista błędów krytycznych
   */
  errors: VerificationError[];
  
  /**
   * Lista ostrzeżeń (nie-krytyczne)
   */
  warnings: VerificationWarning[];
}

interface VerificationError {
  /**
   * Typ błędu
   */
  type: 
    | 'missing_blob'
    | 'corrupt_blob'
    | 'invalid_manifest'
    | 'broken_chain'
    | 'invalid_hash'
    | 'missing_delta';
  
  /**
   * Opis błędu
   */
  message: string;
  
  /**
   * Dodatkowe szczegóły
   */
  details?: {
    path?: string;
    versionId?: string;
    hash?: string;
    [key: string]: unknown;
  };
}

interface VerificationWarning {
  /**
   * Typ ostrzeżenia
   */
  type:
    | 'orphaned_blob'
    | 'large_file'
    | 'many_versions'
    | 'deprecated_format';
  
  /**
   * Opis
   */
  message: string;
  
  /**
   * Sugerowana akcja
   */
  suggestion?: string;
}
```

## 5. Event Types

### 5.1 Events

```typescript
/**
 * Typy eventów emitowanych przez JCFManager
 */
type JCFEvent =
  | ChangeEvent
  | CheckpointEvent
  | RestoreEvent
  | ErrorEvent
  | ProgressEvent;

interface ChangeEvent {
  type: 'change';
  path: string;
  changeType: 'added' | 'modified' | 'deleted';
  timestamp: string;
}

interface CheckpointEvent {
  type: 'checkpoint';
  versionId: string;
  message: string;
  filesChanged: number;
  timestamp: string;
}

interface RestoreEvent {
  type: 'restore';
  versionId: string;
  previousVersionId: string;
  timestamp: string;
}

interface ErrorEvent {
  type: 'error';
  error: Error;
  operation?: string;
  timestamp: string;
}

interface ProgressEvent {
  type: 'progress';
  operation: 'save' | 'restore' | 'gc' | 'export';
  current: number;
  total: number;
  percent: number;
}
```

## 6. Adapter Types

### 6.1 FileSystemAdapter

```typescript
/**
 * Interfejs adaptera systemu plików
 */
interface FileSystemAdapter {
  readonly name: string;
  readonly supportsStreaming: boolean;
  
  init(): Promise<void>;
  dispose(): Promise<void>;
  
  readFile(path: string): Promise<Uint8Array>;
  readFileStream(path: string): Promise<ReadableStream>;
  
  writeFile(path: string, data: Uint8Array): Promise<void>;
  writeFileStream(path: string, stream: ReadableStream): Promise<void>;
  
  fileExists(path: string): Promise<boolean>;
  deleteFile(path: string): Promise<void>;
  getFileSize(path: string): Promise<number>;
  listFiles(directory: string): Promise<string[]>;
  
  createZipWriter(): ZipWriter;
  createZipReader(source: Uint8Array | ReadableStream): ZipReader;
  
  getMetadata(path: string): Promise<FileMetadata>;
}
```

### 6.2 FileMetadata

```typescript
interface FileMetadata {
  size: number;
  created: Date;
  modified: Date;
  isDirectory: boolean;
  mime?: string;
}
```

## 7. Error Types

### 7.1 Custom Errors

```typescript
/**
 * Bazowy error JCF
 */
class JCFError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = 'JCFError';
  }
}

class InvalidPathError extends JCFError {
  constructor(path: string) {
    super(`Invalid path: ${path}`, 'INVALID_PATH');
    this.name = 'InvalidPathError';
  }
}

class FileNotFoundError extends JCFError {
  constructor(path: string, public versionId?: string) {
    super(
      `File not found: ${path}${versionId ? ` in version ${versionId}` : ''}`,
      'FILE_NOT_FOUND'
    );
    this.name = 'FileNotFoundError';
  }
}

class VersionNotFoundError extends JCFError {
  constructor(versionId: string) {
    super(`Version not found: ${versionId}`, 'VERSION_NOT_FOUND');
    this.name = 'VersionNotFoundError';
  }
}

class FileTooLargeError extends JCFError {
  constructor(
    path: string,
    public size: number,
    public maxSize: number
  ) {
    super(
      `File too large: ${path} (${size} bytes, max ${maxSize})`,
      'FILE_TOO_LARGE'
    );
    this.name = 'FileTooLargeError';
  }
}

class StorageError extends JCFError {
  constructor(message: string, public originalError?: Error) {
    super(message, 'STORAGE_ERROR');
    this.name = 'StorageError';
  }
}

class ManifestCorruptionError extends JCFError {
  constructor(message: string) {
    super(`Manifest corruption: ${message}`, 'MANIFEST_CORRUPT');
    this.name = 'ManifestCorruptionError';
  }
}

class BlobCorruptionError extends JCFError {
  constructor(
    public hash: string,
    public expectedHash?: string
  ) {
    super(
      `Blob corruption: ${hash}${expectedHash ? ` (expected: ${expectedHash})` : ''}`,
      'BLOB_CORRUPT'
    );
    this.name = 'BlobCorruptionError';
  }
}
```

## 8. Utility Types

### 8.1 Type Guards

```typescript
/**
 * Type guards dla runtime checking
 */
function isTextFile(entry: FileEntry): boolean {
  return entry.type === 'text';
}

function isBinaryFile(entry: FileEntry): boolean {
  return entry.type === 'binary';
}

function hasFileChanged(
  state1: FileState | undefined,
  state2: FileState | undefined
): boolean {
  if (!state1 || !state2) return true;
  return state1.hash !== state2.hash || state1.size !== state2.size;
}
```

### 8.2 Helper Types

```typescript
/**
 * Deep Partial - wszystkie pola opcjonalne rekursywnie
 */
type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * ReadonlyDeep - wszystkie pola readonly rekursywnie
 */
type ReadonlyDeep<T> = {
  readonly [P in keyof T]: T[P] extends object ? ReadonlyDeep<T[P]> : T[P];
};

/**
 * Async version of function
 */
type AsyncFn<T extends (...args: any[]) => any> = (
  ...args: Parameters<T>
) => Promise<ReturnType<T>>;
```

## 9. Export Summary

```typescript
// Main exports
export {
  // Classes
  JCFManager,
  BrowserAdapter,
  NodeAdapter,
  TauriAdapter,
  MemoryAdapter,
  
  // Types
  JCFConfig,
  Manifest,
  ProjectMetadata,
  FileEntry,
  Version,
  FileState,
  RenameEntry,
  FileInfo,
  VersionDiff,
  FileHistoryEntry,
  ProjectStats,
  GCReport,
  GCOptions,
  VerificationReport,
  FileSystemAdapter,
  FileMetadata,
  
  // Errors
  JCFError,
  InvalidPathError,
  FileNotFoundError,
  VersionNotFoundError,
  FileTooLargeError,
  StorageError,
  ManifestCorruptionError,
  BlobCorruptionError,
  
  // Utils
  createAdapter,
  formatBytes,
  sha256,
  normalizeText
};
```

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

