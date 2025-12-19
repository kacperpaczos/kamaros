# API Reference: JCFManager

## Przegląd

`JCFManager` to główna klasa biblioteki wielojęzycznej JCF (JSON Content Format), zapewniająca interfejs do wszystkich operacji na plikach projektowych z wersjonowaniem. API dostępne dla **TypeScript/JavaScript** i **Python** z wysokowydajnym core w **Rust**.

## Konstruktor

```typescript
constructor(config?: JCFConfig)
```

### Parametry
- `config` (opcjonalny): Konfiguracja managera

### Przykład
```typescript
const manager = new JCFManager({
  author: 'Jan Kowalski',
  compressionLevel: 6,
  autoGC: true
});
```

## Konfiguracja i opcje

### JCFConfig

Główna konfiguracja managera:

```typescript
interface JCFConfig {
  // === Autor i tożsamość ===
  author?: string;           // Domyślny autor commitów
  email?: string;            // Email autora

  // === Wydajność i kompresja ===
  compressionLevel?: number; // Poziom kompresji ZIP (0-9, domyślnie 6)
  useWorkers?: boolean;      // Używaj Web Workers (domyślnie auto-detect)
  workerCount?: number;      // Liczba workerów (domyślnie hardwareConcurrency)
  operationTimeout?: number; // Timeout operacji w ms (domyślnie 30000)

  // === Zarządzanie pamięcią ===
  maxFileSize?: number;      // Max rozmiar pojedynczego pliku (bytes)
  maxHistorySize?: number;   // Max rozmiar historii (MB)
  snapshotInterval?: number; // Co ile commitów robić pełny snapshot

  // === Automatyzacja ===
  autoGC?: boolean;          // Automatyczne GC po commit (domyślnie false)
  autoBackup?: boolean;      // Automatyczne backup przed destruktywnymi operacjami

  // === Bezpieczeństwo ===
  validateOnLoad?: boolean;  // Walidacja przy ładowaniu (domyślnie true)
  validateOnSave?: boolean;  // Walidacja przy zapisie (domyślnie true)

  // === Debugowanie ===
  debug?: boolean;           // Włącz logowanie debug
  verbose?: boolean;         // Szczegółowe komunikaty

  // === Custom ===
  metadata?: Record<string, unknown>; // Dodatkowe metadane projektu
}
```

### Przykład konfiguracji

```typescript
const manager = new JCFManager({
  // Podstawowa konfiguracja
  author: 'Jan Kowalski',
  email: 'jan@example.com',

  // Optymalizacja wydajności
  compressionLevel: 9,    // Maksymalna kompresja
  useWorkers: true,       // Włącz Web Workers
  workerCount: 4,         // 4 workery

  // Zarządzanie pamięcią
  maxFileSize: 500 * 1024 * 1024, // 500MB max
  maxHistorySize: 100,            // 100MB historii

  // Automatyzacja
  autoGC: true,           // GC po każdym commicie
  autoBackup: true,       // Backup przed restore

  // Debug
  debug: process.env.NODE_ENV === 'development'
});
```

### Runtime Configuration

Konfigurację można zmieniać w runtime:

```typescript
// Pobierz aktualną konfigurację
const config = manager.getConfig();

// Aktualizuj konfigurację
manager.updateConfig({
  compressionLevel: 9,
  autoGC: true
});

// Resetuj do domyślnych
manager.resetConfig();
```

### Environment Variables

Konfiguracja może być również ustawiana przez zmienne środowiskowe:

```bash
# Autor
export JCF_AUTHOR="Jan Kowalski"
export JCF_EMAIL="jan@example.com"

# Wydajność
export JCF_COMPRESSION_LEVEL=9
export JCF_USE_WORKERS=true
export JCF_WORKER_COUNT=4

# Pamięć
export JCF_MAX_FILE_SIZE=524288000  # 500MB
export JCF_MAX_HISTORY_SIZE=100     # 100MB

# Debug
export JCF_DEBUG=true
export JCF_VERBOSE=true
```

Zmienne środowiskowe mają niższy priorytet niż konfiguracja przekazana programistycznie.

## Metody główne

### init(adapter: FileSystemAdapter, options?: InitOptions)
Inicjalizuje manager z adapterem systemu plików.

```typescript
await manager.init(new BrowserAdapter(), {
  createIfMissing: true,
  validateOnLoad: true
});
```

**Parametry:**
- `adapter`: Adapter systemu plików
- `options.createIfMissing`: Czy utworzyć nowy projekt jeśli nie istnieje (domyślnie: `true`)
- `options.validateOnLoad`: Czy walidować integralność przy ładowaniu (domyślnie: `true`)

**Zwraca**: `Promise<void>`

### saveCheckpoint(message: string, options?: CheckpointOptions)
Tworzy nowy checkpoint z bieżącymi zmianami.

```typescript
// Prosty checkpoint
const versionId = await manager.saveCheckpoint('Add login feature');

// Checkpoint z metadanymi
const versionId = await manager.saveCheckpoint('Release v1.0', {
  author: 'Jan Kowalski',
  email: 'jan@example.com',
  tags: ['release', 'v1.0']
});
```

**Parametry:**
- `message`: Wiadomość opisująca zmiany
- `options.author`: Autor checkpoint (nadpisuje domyślnego)
- `options.email`: Email autora
- `options.tags`: Tagi dla tej wersji
- `options.metadata`: Dodatkowe metadane

**Zwraca**: `Promise<string>` - ID nowej wersji

### restoreVersion(versionId: string, options?: RestoreOptions)
Przywraca projekt do wskazanej wersji.

```typescript
// Przywracanie do konkretnej wersji
await manager.restoreVersion('abc123def456');

// Przywracanie z opcjami
await manager.restoreVersion('v1.0', {
  createBackup: true,
  preserveStaged: false
});
```

**Parametry:**
- `versionId`: ID wersji do przywrócenia
- `options.createBackup`: Czy utworzyć backup obecnego stanu
- `options.preserveStaged`: Czy zachować pliki w staging area

**Zwraca**: `Promise<void>`

### addFile(path: string, content: Uint8Array | string, options?: AddFileOptions)
Dodaje lub aktualizuje plik w projekcie.

```typescript
// Dodanie pliku tekstowego
await manager.addFile('src/index.js', 'console.log("Hello");');

// Dodanie pliku binarnego
const imageData = await fetch('/logo.png').then(r => r.arrayBuffer());
await manager.addFile('assets/logo.png', new Uint8Array(imageData));

// Dodanie z opcjami
await manager.addFile('config.json', JSON.stringify(config), {
  encoding: 'utf-8',
  mimeType: 'application/json'
});
```

**Parametry:**
- `path`: Ścieżka pliku
- `content`: Zawartość jako string lub Uint8Array
- `options.encoding`: Kodowanie tekstu (domyślnie auto-detect)
- `options.mimeType`: MIME type pliku
- `options.metadata`: Dodatkowe metadane

**Zwraca**: `Promise<void>`

### removeFile(path: string, options?: RemoveOptions)
Usuwa plik z projektu.

```typescript
// Proste usunięcie
await manager.removeFile('old-file.js');

// Usunięcie z opcjami
await manager.removeFile('temp/cache.dat', {
  force: true,
  recursive: false
});
```

**Parametry:**
- `path`: Ścieżka do usunięcia
- `options.force`: Wymuś usunięcie nawet jeśli plik się zmienił
- `options.recursive`: Usuń rekursywnie katalogi

**Zwraca**: `Promise<void>`

### getFileContent(path: string, versionId?: string)
Pobiera zawartość pliku z wskazanej wersji (domyślnie HEAD).

```typescript
// Zawartość z HEAD
const content = await manager.getFileContent('src/index.js');

// Zawartość z konkretnej wersji
const oldContent = await manager.getFileContent('src/index.js', 'abc123');
```

**Parametry:**
- `path`: Ścieżka pliku
- `versionId`: ID wersji (opcjonalne, domyślnie HEAD)

**Zwraca**: `Promise<Uint8Array>` - Zawartość pliku jako bajty

### getFileInfo(path: string, versionId?: string)
Pobiera informacje o pliku.

```typescript
const info = await manager.getFileInfo('src/index.js');
// {
//   path: 'src/index.js',
//   size: 1024,
//   type: 'text',
//   hash: 'abc123...',
//   modified: new Date('2025-01-01'),
//   encoding: 'utf-8',
//   mime: 'application/javascript'
// }
```

**Parametry:**
- `path`: Ścieżka pliku
- `versionId`: ID wersji (opcjonalne)

**Zwraca**: `Promise<FileInfo>`

### listFiles(directory?: string, options?: ListOptions)
Lista plików w katalogu.

```typescript
// Wszystkie pliki w katalogu
const files = await manager.listFiles('src/');

// Szczegółowa lista
const detailedFiles = await manager.listFiles('', {
  recursive: true,
  includeMetadata: true,
  type: 'text' // tylko pliki tekstowe
});
```

**Parametry:**
- `directory`: Katalog do przeszukania (domyślnie root)
- `options.recursive`: Przeszukiwanie rekursywne
- `options.includeMetadata`: Dołącz metadane
- `options.type`: Filtrowanie po typie ('text' | 'binary')
- `options.pattern`: Wzorzec glob do filtrowania

**Zwraca**: `Promise<FileInfo[]>` - Lista plików z informacjami

## Metody zarządzania wersjami

### getCurrentVersion()
Pobiera ID bieżącej wersji (HEAD).

```typescript
const headId = await manager.getCurrentVersion();
console.log('Current HEAD:', headId);
```

**Zwraca**: `Promise<string>` - ID wersji HEAD

### getHistory(options?: HistoryOptions)
Pobiera historię wersji.

```typescript
// Wszystkie wersje
const history = await manager.getHistory();

// Ograniczona historia
const recent = await manager.getHistory({
  limit: 10,
  since: '2025-01-01',
  author: 'Jan Kowalski'
});

// Historia z szczególamu
const detailed = await manager.getHistory({
  includeFileChanges: true,
  includeStats: true
});
```

**Parametry:**
- `options.limit`: Maksymalna liczba wersji
- `options.since`: Tylko wersje od daty (ISO 8601)
- `options.until`: Tylko wersje do daty
- `options.author`: Filtrowanie po autorze
- `options.includeFileChanges`: Dołącz informacje o zmienionych plikach
- `options.includeStats`: Dołącz statystyki

**Zwraca**: `Promise<Version[]>` - Lista wersji od najnowszej

### getVersion(versionId: string)
Pobiera szczegóły konkretnej wersji.

```typescript
const version = await manager.getVersion('abc123def456');
if (version) {
  console.log(`Version ${version.id}: ${version.message}`);
  console.log(`Author: ${version.author} at ${version.timestamp}`);
}
```

**Parametry:**
- `versionId`: ID wersji do pobrania

**Zwraca**: `Promise<Version | null>` - Szczegóły wersji lub null jeśli nie istnieje

### getVersionDiff(fromVersion: string, toVersion: string)
Porównuje dwie wersje i zwraca różnice.

```typescript
const diff = await manager.getVersionDiff('v1.0', 'v1.1');
console.log('Added files:', diff.added);
console.log('Modified files:', diff.modified);
console.log('Deleted files:', diff.deleted);
```

**Parametry:**
- `fromVersion`: Bazowa wersja do porównania
- `toVersion`: Docelowa wersja

**Zwraca**: `Promise<VersionDiff>` - Różnice między wersjami

### getFileHistory(path: string, options?: FileHistoryOptions)
Pobiera historię zmian konkretnego pliku.

```typescript
const history = await manager.getFileHistory('src/index.js', {
  limit: 20,
  includeContent: false // tylko metadane, bez zawartości
});

history.forEach(entry => {
  console.log(`${entry.timestamp}: ${entry.changeType} (${entry.size} bytes)`);
});
```

**Parametry:**
- `path`: Ścieżka pliku
- `options.limit`: Maksymalna liczba wpisów
- `options.includeContent`: Czy dołączyć zawartość plików
- `options.includeDiff`: Czy dołączyć diff zmian

**Zwraca**: `Promise<FileHistoryEntry[]>` - Historia zmian pliku

## Metody narzędziowe

### getStats()
Pobiera statystyki projektu.

```typescript
const stats = await manager.getStats();
console.log(`Project has ${stats.totalVersions} versions`);
console.log(`Total size: ${stats.totalSize} bytes`);
console.log(`Deduplication ratio: ${stats.deduplicationRatio}x`);
```

**Zwraca**: `Promise<ProjectStats>` - Szczegółowe statystyki projektu

### runGC(options?: GCOptions)
Uruchamia garbage collection dla optymalizacji przestrzeni.

```typescript
// Standardowe GC
const report = await manager.runGC();

// GC z opcjami
const report = await manager.runGC({
  gracePeriodDays: 7, // nie usuwaj plików młodszych niż 7 dni
  showProgress: true,
  onProgress: (progress) => {
    console.log(`GC: ${progress.percent}% (${progress.phase})`);
  }
});

console.log(`Freed ${report.spaceFreed} bytes`);
```

**Parametry:**
- `options.gracePeriodDays`: Okres karencji dla nowych plików
- `options.showProgress`: Wyświetlaj postęp
- `options.onProgress`: Callback dla aktualizacji postępu

**Zwraca**: `Promise<GCReport>` - Raport z wyników czyszczenia

### verifyIntegrity(options?: VerifyOptions)
Sprawdza integralność danych projektu.

```typescript
const report = await manager.verifyIntegrity({
  checkBlobs: true,
  checkManifest: true,
  repair: false, // tylko sprawdź, nie naprawiaj
  onProgress: (progress) => console.log(`Verify: ${progress.percent}%`)
});

if (report.valid) {
  console.log('Project integrity OK');
} else {
  console.log('Found errors:', report.errors);
}
```

**Parametry:**
- `options.checkBlobs`: Sprawdź integralność blobów
- `options.checkManifest`: Sprawdź manifest
- `options.repair`: Automatycznie napraw wykryte błędy
- `options.onProgress`: Callback postępu

**Zwraca**: `Promise<VerificationReport>` - Raport weryfikacji

### export(options?: ExportOptions)
Eksportuje projekt jako stream JCF.

```typescript
// Eksport całego projektu
const stream = await manager.export();

// Eksport konkretnej wersji
const stream = await manager.export({
  versionId: 'v1.0',
  includeHistory: true,
  compressionLevel: 9
});

// Zapisz do pliku
const response = new Response(stream);
const blob = await response.blob();
// ... zapisz blob do pliku
```

**Parametry:**
- `options.versionId`: Wersja do eksportu (domyślnie HEAD)
- `options.includeHistory`: Czy dołączyć pełną historię
- `options.compressionLevel`: Poziom kompresji (0-9)
- `options.onProgress`: Callback postępu

**Zwraca**: `Promise<ReadableStream>` - Stream z danymi JCF

### import(source: ReadableStream | Uint8Array, options?: ImportOptions)
Importuje projekt z streama lub danych JCF.

```typescript
// Import ze streama
const response = await fetch('project.jcf');
const stream = response.body;
await manager.import(stream);

// Import z ArrayBuffer
const buffer = await file.arrayBuffer();
await manager.import(new Uint8Array(buffer), {
  validateOnImport: true,
  mergeHistory: false // nadpisz istniejącą historię
});
```

**Parametry:**
- `source`: Stream lub dane do importu
- `options.validateOnImport`: Waliduj podczas importu
- `options.mergeHistory`: Scal historię zamiast nadpisywać
- `options.onProgress`: Callback postępu

**Zwraca**: `Promise<void>`

### createBackup(name?: string)
Tworzy backup bieżącego stanu projektu.

```typescript
const backupId = await manager.createBackup('before-refactor');
console.log('Backup created:', backupId);

// Lista backupów
const backups = await manager.listBackups();

// Przywróć z backup
await manager.restoreFromBackup(backupId);
```

**Parametry:**
- `name`: Opcjonalna nazwa backup

**Zwraca**: `Promise<string>` - ID backup

### cleanup(options?: CleanupOptions)
Czyści tymczasowe pliki i optymalizuje projekt.

```typescript
await manager.cleanup({
  removeTempFiles: true,
  compactManifest: true,
  rebuildIndexes: true
});
```

**Parametry:**
- `options.removeTempFiles`: Usuń pliki tymczasowe
- `options.compactManifest`: Zoptymalizuj manifest
- `options.rebuildIndexes`: Przebuduj indeksy

**Zwraca**: `Promise<void>`

## System zdarzeń

JCFManager emituje zdarzenia podczas operacji, umożliwiając monitorowanie postępu i reagowanie na zmiany.

### Subskrypcja zdarzeń

```typescript
import { JCFManager, JCFEvent } from 'jcf-manager';

const manager = new JCFManager();

// Subskrypcja pojedynczego zdarzenia
manager.on('checkpoint:complete', (event) => {
  console.log('Checkpoint done:', event.versionId);
});

// Subskrypcja wielu zdarzeń
manager.on(['checkpoint:start', 'checkpoint:complete'], (event) => {
  console.log('Checkpoint event:', event.type);
});

// Jednorazowa subskrypcja
manager.once('error', (event) => {
  console.error('Error occurred:', event.error);
});

// Usunięcie subskrypcji
const handler = (event) => console.log(event);
manager.on('progress', handler);
manager.off('progress', handler);

// Usunięcie wszystkich handlerów dla zdarzenia
manager.off('progress');

// Usunięcie wszystkich subskrypcji
manager.removeAllListeners();
```

### Zdarzenia operacyjne

#### Checkpoint Events
```typescript
manager.on('checkpoint:start', (event: CheckpointStartEvent) => {
  console.log('Starting checkpoint:', event.message);
  // { type: 'checkpoint:start', message: string, timestamp: string }
});

manager.on('checkpoint:progress', (event: CheckpointProgressEvent) => {
  console.log(`Progress: ${event.percent}% (${event.phase})`);
  // { type: 'checkpoint:progress', percent: number, phase: string, current: number, total: number }
});

manager.on('checkpoint:complete', (event: CheckpointCompleteEvent) => {
  console.log('Checkpoint created:', event.versionId);
  // { type: 'checkpoint:complete', versionId: string, message: string, filesChanged: number }
});

manager.on('checkpoint:error', (event: CheckpointErrorEvent) => {
  console.error('Checkpoint failed:', event.error);
  // { type: 'checkpoint:error', error: Error, operation: string }
});
```

#### Restore Events
```typescript
manager.on('restore:start', (event: RestoreStartEvent) => {
  console.log('Starting restore to:', event.versionId);
});

manager.on('restore:progress', (event: RestoreProgressEvent) => {
  console.log(`Restore: ${event.percent}%`);
});

manager.on('restore:complete', (event: RestoreCompleteEvent) => {
  console.log('Restore complete');
});

manager.on('restore:error', (event: RestoreErrorEvent) => {
  console.error('Restore failed:', event.error);
});
```

#### GC Events
```typescript
manager.on('gc:start', (event: GCStartEvent) => {
  console.log('Starting garbage collection');
});

manager.on('gc:progress', (event: GCProgressEvent) => {
  console.log(`GC: ${event.percent}% (${event.phase})`);
});

manager.on('gc:complete', (event: GCCompleteEvent) => {
  console.log(`GC complete: ${event.spaceFreed} bytes freed`);
});
```

#### File Operation Events
```typescript
manager.on('file:change', (event: FileChangeEvent) => {
  console.log(`File ${event.changeType}: ${event.path}`);
  // { type: 'file:change', path: string, changeType: 'added' | 'modified' | 'deleted' }
});
```

#### General Events
```typescript
manager.on('progress', (event: ProgressEvent) => {
  // Uniwersalne zdarzenie postępu dla wszystkich operacji
  console.log(`${event.operation}: ${event.percent}%`);
});

manager.on('error', (event: ErrorEvent) => {
  // Globalne zdarzenie błędu
  console.error('Operation failed:', event.error);
});
```

### Typy zdarzeń

Wszystkie zdarzenia implementują wspólny interfejs:

```typescript
interface BaseEvent {
  type: string;
  timestamp: string;
}

interface CheckpointStartEvent extends BaseEvent {
  type: 'checkpoint:start';
  message: string;
}

interface CheckpointProgressEvent extends BaseEvent {
  type: 'checkpoint:progress';
  percent: number;
  phase: 'analyzing' | 'hashing' | 'saving' | 'updating';
  current: number;
  total: number;
}

// ... pozostałe typy zdarzeń
```

## Typy

```typescript
interface Version {
  id: string;
  timestamp: string;
  message: string;
  author: string;
  parentId: string | null;
  fileStates: Map<string, FileState>;
}

interface FileState {
  inodeId: string;
  hash?: string;
  contentRef?: string;
  deleted?: boolean;
}

interface GCReport {
  blobsRemoved: number;
  spaceFreed: number;
}
```

## Obsługa błędów

JCFManager używa hierarchii błędów dziedziczących po `JCFError`. Wszystkie błędy zawierają kod, wiadomość i opcjonalne szczegóły.

### Hierarchia błędów

```typescript
class JCFError extends Error {
  constructor(message: string, public code: string, public details?: any) {
    super(message);
    this.name = 'JCFError';
  }
}

// === Błędy walidacji ===
class ValidationError extends JCFError {
  constructor(message: string, public field?: string) {
    super(message, 'VALIDATION_ERROR', { field });
    this.name = 'ValidationError';
  }
}

// === Błędy systemu plików ===
class FileNotFoundError extends JCFError {
  constructor(path: string, public versionId?: string) {
    super(
      `File not found: ${path}${versionId ? ` in version ${versionId}` : ''}`,
      'FILE_NOT_FOUND',
      { path, versionId }
    );
    this.name = 'FileNotFoundError';
  }
}

class FileExistsError extends JCFError {
  constructor(path: string) {
    super(`File already exists: ${path}`, 'FILE_EXISTS', { path });
    this.name = 'FileExistsError';
  }
}

class StorageError extends JCFError {
  constructor(message: string, public originalError?: Error) {
    super(message, 'STORAGE_ERROR', { originalError });
    this.name = 'StorageError';
  }
}

// === Błędy wersjonowania ===
class VersionNotFoundError extends JCFError {
  constructor(versionId: string) {
    super(`Version not found: ${versionId}`, 'VERSION_NOT_FOUND', { versionId });
    this.name = 'VersionNotFoundError';
  }
}

class VersionConflictError extends JCFError {
  constructor(message: string, public localVersion: string, public remoteVersion: string) {
    super(message, 'VERSION_CONFLICT', { localVersion, remoteVersion });
    this.name = 'VersionConflictError';
  }
}

// === Błędy integralności ===
class CorruptionError extends JCFError {
  constructor(message: string, public corruptedItem: string) {
    super(`Data corruption: ${message}`, 'CORRUPTION_ERROR', { corruptedItem });
    this.name = 'CorruptionError';
  }
}

class ManifestCorruptionError extends CorruptionError {
  constructor(details: string) {
    super(`Manifest corruption: ${details}`, 'manifest');
    this.name = 'ManifestCorruptionError';
  }
}

class BlobCorruptionError extends CorruptionError {
  constructor(hash: string, expectedHash?: string) {
    super(
      `Blob corruption: ${hash}${expectedHash ? ` (expected: ${expectedHash})` : ''}`,
      `blob:${hash}`
    );
    this.name = 'BlobCorruptionError';
  }
}

// === Błędy operacyjne ===
class OperationTimeoutError extends JCFError {
  constructor(operation: string, timeout: number) {
    super(
      `Operation timeout: ${operation} (${timeout}ms)`,
      'OPERATION_TIMEOUT',
      { operation, timeout }
    );
    this.name = 'OperationTimeoutError';
  }
}

class FileTooLargeError extends JCFError {
  constructor(path: string, size: number, maxSize: number) {
    super(
      `File too large: ${path} (${size} bytes, max ${maxSize})`,
      'FILE_TOO_LARGE',
      { path, size, maxSize }
    );
    this.name = 'FileTooLargeError';
  }
}

class InsufficientSpaceError extends JCFError {
  constructor(required: number, available: number) {
    super(
      `Insufficient space: ${required} required, ${available} available`,
      'INSUFFICIENT_SPACE',
      { required, available }
    );
    this.name = 'InsufficientSpaceError';
  }
}
```

### Obsługa błędów

```typescript
try {
  await manager.saveCheckpoint('My changes');
} catch (error) {
  if (error instanceof FileNotFoundError) {
    console.error('File not found:', error.details.path);
  } else if (error instanceof VersionNotFoundError) {
    console.error('Version does not exist:', error.details.versionId);
  } else if (error instanceof ValidationError) {
    console.error('Invalid input:', error.details.field, error.message);
  } else if (error instanceof StorageError) {
    console.error('Storage error:', error.originalError?.message);
  } else if (error.code === 'OPERATION_TIMEOUT') {
    console.error('Operation timed out, try again');
  } else {
    console.error('Unknown error:', error);
  }
}
```

### Async Error Handling

```typescript
// Z użyciem zdarzeń
manager.on('error', (event) => {
  console.error('Async error:', event.error);
  // Obsłuż błąd asynchroniczny
});

// Z użyciem try/catch w async funkcjach
async function safeOperation() {
  try {
    await manager.restoreVersion('some-version');
  } catch (error) {
    if (error instanceof VersionNotFoundError) {
      // Spróbuj inną wersję
      await manager.restoreVersion('HEAD~1');
    } else {
      throw error; // Przepuść dalej
    }
  }
}
```

### Error Recovery

Niektóre błędy można automatycznie naprawić:

```typescript
async function robustOperation() {
  try {
    await manager.verifyIntegrity({ repair: true });
  } catch (error) {
    if (error instanceof CorruptionError) {
      console.log('Attempting repair...');
      await manager.verifyIntegrity({ repair: true });
    }
  }
}
```

## Przykład użycia

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

// Inicjalizacja
const manager = new JCFManager();
await manager.init(new BrowserAdapter());

// Dodanie plików
await manager.addFile('package.json', JSON.stringify({
  name: 'my-project',
  version: '1.0.0'
}));

await manager.addFile('src/index.js', 'console.log("Hello World");');

// Checkpoint
const versionId = await manager.saveCheckpoint('Initial commit');

// Lista plików
const files = await manager.listFiles();

// Pobranie zawartości
const content = await manager.getFileContent('src/index.js');

// Historia
const history = await manager.getHistory();
console.log(`Project has ${history.length} versions`);
```