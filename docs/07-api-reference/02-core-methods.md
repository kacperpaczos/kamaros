# Core Methods

> **Szczegółowa dokumentacja głównych metod JCFManager**

[← Back: JCFManager Class](01-jcf-manager-class.md) | [Next: Query Methods →](03-query-methods.md)

---

## Overview

Core Methods to podstawowe operacje JCFManager odpowiedzialne za inicjalizację, wersjonowanie i zarządzanie plikami. Te metody są używane w większości operacji na projekcie.

---

## init(adapter, options?)

Inicjalizuje manager z adapterem systemu plików i opcjonalnymi opcjami.

### Sygnatura

```typescript
async init(
  adapter: FileSystemAdapter,
  options?: InitOptions
): Promise<void>
```

### Parametry

- **adapter** (`FileSystemAdapter`): Adapter systemu plików dla danej platformy
  - `BrowserAdapter` - dla przeglądarki (IndexedDB)
  - `NodeAdapter` - dla Node.js (fs/promises)
  - `TauriAdapter` - dla Tauri (tauri.fs)
  - `MemoryAdapter` - dla testów (in-memory)

- **options** (`InitOptions?`): Opcjonalne opcje inicjalizacji
  ```typescript
  interface InitOptions {
    createIfMissing?: boolean;    // Utwórz nowy projekt jeśli nie istnieje (domyślnie: true)
    validateOnLoad?: boolean;    // Waliduj integralność przy ładowaniu (domyślnie: true)
    projectPath?: string;         // Ścieżka do projektu (domyślnie: './project.jcf')
  }
  ```

### Zwraca

`Promise<void>` - Resolves gdy inicjalizacja zakończona

### Przykłady

```typescript
// Browser
import { JCFManager, BrowserAdapter } from 'jcf-manager';

const manager = new JCFManager();
await manager.init(new BrowserAdapter(), {
  createIfMissing: true,
  validateOnLoad: true
});

// Node.js
import { NodeAdapter } from 'jcf-manager/node';

const manager = new JCFManager();
await manager.init(new NodeAdapter('./my-project'), {
  createIfMissing: false,  // Rzuć błąd jeśli projekt nie istnieje
  validateOnLoad: true
});

// Tauri
import { TauriAdapter } from 'jcf-manager/tauri';

const manager = new JCFManager();
await manager.init(new TauriAdapter());
```

### Błędy

- `StorageError` - Błąd dostępu do storage
- `ValidationError` - Błąd walidacji manifestu
- `FileNotFoundError` - Projekt nie istnieje (jeśli `createIfMissing: false`)

### Implementacja

```typescript
async init(adapter: FileSystemAdapter, options: InitOptions = {}): Promise<void> {
  this.adapter = adapter;
  this.config = { ...this.config, ...options };
  
  // Initialize adapter
  await adapter.init();
  
  // Load or create project
  if (await adapter.fileExists(this.config.projectPath)) {
    await this.loadProject();
  } else if (options.createIfMissing !== false) {
    await this.createNewProject();
  } else {
    throw new FileNotFoundError(`Project not found: ${this.config.projectPath}`);
  }
  
  // Validate if requested
  if (options.validateOnLoad !== false) {
    await this.validateIntegrity();
  }
}
```

---

## saveCheckpoint(message, options?)

Tworzy nowy checkpoint (commit) z bieżącymi zmianami w projekcie.

### Sygnatura

```typescript
async saveCheckpoint(
  message: string,
  options?: CheckpointOptions
): Promise<string>
```

### Parametry

- **message** (`string`): Wiadomość opisująca zmiany (commit message)
  - Wymagane
  - Maksymalna długość: 1000 znaków (rekomendowane: <200)

- **options** (`CheckpointOptions?`): Opcjonalne opcje checkpoint
  ```typescript
  interface CheckpointOptions {
    author?: string;              // Autor (nadpisuje domyślnego z config)
    email?: string;               // Email autora
    tags?: string[];              // Tagi dla tej wersji (np. ['release', 'v1.0'])
    metadata?: Record<string, unknown>;  // Dodatkowe metadane
    skipValidation?: boolean;     // Pomiń walidację (nie rekomendowane)
  }
  ```

### Zwraca

`Promise<string>` - ID nowej wersji (UUID v4)

### Przykłady

```typescript
// Prosty checkpoint
const versionId = await manager.saveCheckpoint('Add login feature');

// Checkpoint z metadanymi
const versionId = await manager.saveCheckpoint('Release v1.0', {
  author: 'Jan Kowalski',
  email: 'jan@example.com',
  tags: ['release', 'v1.0', 'production'],
  metadata: {
    buildNumber: 1234,
    deployedAt: new Date().toISOString()
  }
});

// Checkpoint z custom autorem
const versionId = await manager.saveCheckpoint('Fix bug #42', {
  author: 'Anna Nowak',
  email: 'anna@example.com'
});
```

### Zdarzenia

Manager emituje zdarzenia podczas saveCheckpoint:

```typescript
manager.on('checkpoint:start', (event) => {
  console.log('Starting checkpoint:', event.message);
});

manager.on('checkpoint:progress', (event) => {
  console.log(`Progress: ${event.percent}% (${event.phase})`);
  // phase: 'scanning' | 'diffing' | 'hashing' | 'saving' | 'updating'
});

manager.on('checkpoint:complete', (event) => {
  console.log('Checkpoint created:', event.versionId);
  console.log('Files changed:', event.filesChanged);
});

manager.on('checkpoint:error', (event) => {
  console.error('Checkpoint failed:', event.error);
});
```

### Błędy

- `NoChangesError` - Brak zmian do zapisania
- `StorageError` - Błąd zapisu do storage
- `ValidationError` - Błąd walidacji danych
- `OperationTimeoutError` - Timeout operacji

### Implementacja

```typescript
async saveCheckpoint(message: string, options?: CheckpointOptions): Promise<string> {
  const startTime = Date.now();
  
  try {
    this.emit('checkpoint:start', { message });
    
    // 1. Identify changed files
    const changes = await this.identifyChangedFiles();
    if (changes.length === 0) {
      throw new NoChangesError('No changes to commit');
    }
    
    // 2. Process text files (generate reverse patches)
    for (const change of changes.filter(c => c.type === 'text')) {
      const newContent = await this.readWorkingCopy(change.path);
      const oldContent = await this.getFileFromVersion(change.path, this.manifest.refs.head);
      const patch = await this.computeDiff(newContent, oldContent);
      await this.saveDelta(this.manifest.refs.head, change.path, patch);
    }
    
    // 3. Process binary files (CAS)
    for (const change of changes.filter(c => c.type === 'binary')) {
      const content = await this.readWorkingCopy(change.path);
      const hash = await this.hashContent(content);
      if (!await this.blobExists(hash)) {
        await this.saveBlob(hash, content);
      }
    }
    
    // 4. Create version
    const versionId = uuidv4();
    const version = {
      id: versionId,
      parentId: this.manifest.refs.head,
      timestamp: new Date().toISOString(),
      message,
      author: options?.author || this.config.author,
      fileStates: this.buildFileStates(changes)
    };
    
    // 5. Update manifest
    this.manifest.versionHistory.push(version);
    this.manifest.refs.head = versionId;
    await this.writeManifest();
    
    this.emit('checkpoint:complete', { versionId, duration: Date.now() - startTime });
    return versionId;
  } catch (error) {
    this.emit('checkpoint:error', { error });
    throw error;
  }
}
```

---

## restoreVersion(versionId, options?)

Przywraca projekt do wskazanej wersji (time-travel).

### Sygnatura

```typescript
async restoreVersion(
  versionId: string,
  options?: RestoreOptions
): Promise<void>
```

### Parametry

- **versionId** (`string`): ID wersji do przywrócenia
  - Może być UUID wersji lub tag (np. 'v1.0')
  - Jeśli `'HEAD'` lub `null` - przywraca do najnowszej wersji

- **options** (`RestoreOptions?`): Opcjonalne opcje restore
  ```typescript
  interface RestoreOptions {
    createBackup?: boolean;       // Utwórz backup przed restore (domyślnie: false)
    preserveStaged?: boolean;     // Zachowaj pliki w staging area (domyślnie: false)
    validateAfter?: boolean;      // Waliduj po restore (domyślnie: true)
  }
  ```

### Zwraca

`Promise<void>` - Resolves gdy restore zakończony

### Przykłady

```typescript
// Przywracanie do konkretnej wersji
await manager.restoreVersion('abc123-def456-...');

// Przywracanie z backup
await manager.restoreVersion('v1.0', {
  createBackup: true
});

// Przywracanie do HEAD (najnowszej wersji)
await manager.restoreVersion('HEAD');

// Przywracanie z zachowaniem staged files
await manager.restoreVersion('v2.0', {
  preserveStaged: true
});
```

### Zdarzenia

```typescript
manager.on('restore:start', (event) => {
  console.log('Restoring to:', event.versionId);
});

manager.on('restore:progress', (event) => {
  console.log(`Restore: ${event.percent}%`);
  // event.file - aktualnie przetwarzany plik
  // event.fromVersion, event.toVersion - wersje
});

manager.on('restore:complete', (event) => {
  console.log('Restore complete');
});

manager.on('restore:error', (event) => {
  console.error('Restore failed:', event.error);
});
```

### Błędy

- `VersionNotFoundError` - Wersja nie istnieje
- `StorageError` - Błąd odczytu/zapisu
- `PatchApplicationError` - Błąd aplikowania patchy
- `BlobNotFoundError` - Brakujący blob

### Implementacja

Zobacz [Restore Version Algorithm](../05-algorithms/02-restore-version.md) dla szczegółów implementacji.

---

## addFile(path, content, options?)

Dodaje lub aktualizuje plik w projekcie.

### Sygnatura

```typescript
async addFile(
  path: string,
  content: string | Uint8Array | ReadableStream,
  options?: AddFileOptions
): Promise<void>
```

### Parametry

- **path** (`string`): Ścieżka pliku (relative do root projektu)
  - Musi być valid path (no '..', no absolute paths)
  - Przykłady: `'src/index.js'`, `'assets/logo.png'`

- **content** (`string | Uint8Array | ReadableStream`): Zawartość pliku
  - `string` - dla plików tekstowych
  - `Uint8Array` - dla plików binarnych (małe pliki)
  - `ReadableStream` - dla dużych plików (streaming)

- **options** (`AddFileOptions?`): Opcjonalne opcje
  ```typescript
  interface AddFileOptions {
    encoding?: string;            // Kodowanie tekstu (domyślnie: auto-detect)
    mimeType?: string;           // MIME type (domyślnie: auto-detect z extension)
    metadata?: Record<string, unknown>;  // Dodatkowe metadane
    createInode?: boolean;        // Utwórz nowy inodeId (domyślnie: auto)
  }
  ```

### Zwraca

`Promise<void>` - Resolves gdy plik dodany

### Przykłady

```typescript
// Dodanie pliku tekstowego
await manager.addFile('src/index.js', 'console.log("Hello");');

// Dodanie pliku binarnego (mały)
const imageData = await fetch('/logo.png').then(r => r.arrayBuffer());
await manager.addFile('assets/logo.png', new Uint8Array(imageData));

// Dodanie dużego pliku (streaming)
const fileStream = file.stream();
await manager.addFile('videos/demo.mp4', fileStream);

// Dodanie z opcjami
await manager.addFile('config.json', JSON.stringify(config), {
  encoding: 'utf-8',
  mimeType: 'application/json',
  metadata: {
    source: 'user-input',
    validated: true
  }
});
```

### Automatyczne wykrywanie typu

Manager automatycznie wykrywa typ pliku na podstawie:
1. **Extension** - `.js`, `.ts`, `.json` → text; `.png`, `.jpg` → binary
2. **Content** - Jeśli extension nieznany, analizuje zawartość
3. **MIME type** - Jeśli podany w options

### Błędy

- `FileTooLargeError` - Plik przekracza limit (domyślnie: 500MB)
- `InvalidPathError` - Nieprawidłowa ścieżka
- `StorageError` - Błąd zapisu

### Implementacja

```typescript
async addFile(
  path: string,
  content: string | Uint8Array | ReadableStream,
  options: AddFileOptions = {}
): Promise<void> {
  // Validate path
  this.validatePath(path);
  
  // Detect file type
  const fileType = this.detectFileType(path, content, options);
  
  // Create or get file entry
  const fileEntry = this.manifest.fileMap[path];
  const inodeId = fileEntry?.inodeId || uuidv4();
  
  // Write to working copy
  if (content instanceof ReadableStream) {
    await this.writeFileStreaming(`content/${path}`, content);
  } else {
    await this.writeFile(`content/${path}`, content);
  }
  
  // Update fileMap
  this.manifest.fileMap[path] = {
    inodeId,
    path,
    type: fileType,
    created: fileEntry?.created || new Date().toISOString(),
    modified: new Date().toISOString(),
    ...options.metadata
  };
  
  // Mark as dirty (will be included in next checkpoint)
  this.dirtyFiles.add(path);
}
```

---

## removeFile(path, options?)

Usuwa plik z projektu.

### Sygnatura

```typescript
async removeFile(
  path: string,
  options?: RemoveFileOptions
): Promise<void>
```

### Parametry

- **path** (`string`): Ścieżka pliku do usunięcia

- **options** (`RemoveFileOptions?`): Opcjonalne opcje
  ```typescript
  interface RemoveFileOptions {
    force?: boolean;              // Wymuś usunięcie nawet jeśli plik się zmienił
    recursive?: boolean;          // Usuń rekursywnie katalogi (future)
  }
  ```

### Zwraca

`Promise<void>`

### Przykłady

```typescript
// Proste usunięcie
await manager.removeFile('old-file.js');

// Wymuszone usunięcie
await manager.removeFile('temp/cache.dat', {
  force: true
});
```

### Błędy

- `FileNotFoundError` - Plik nie istnieje
- `StorageError` - Błąd usunięcia

---

## Podsumowanie Core Methods

| Metoda | Purpose | Complexity | Events |
|--------|---------|------------|--------|
| `init()` | Inicjalizacja managera | O(1) | - |
| `saveCheckpoint()` | Tworzenie wersji | O(F × L) | checkpoint:* |
| `restoreVersion()` | Przywracanie wersji | O(V × F) | restore:* |
| `addFile()` | Dodanie pliku | O(1) | file:change |
| `removeFile()` | Usunięcie pliku | O(1) | file:change |

**Legenda**:
- F = liczba plików
- L = średnia długość pliku
- V = liczba wersji do przywrócenia

---

## Kompletny Przykład Użycia Core Methods

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

// 1. Inicjalizacja
const manager = new JCFManager({
  author: 'Jan Kowalski',
  compressionLevel: 6
});

await manager.init(new BrowserAdapter(), {
  createIfMissing: true,
  validateOnLoad: true
});

// 2. Dodanie plików
await manager.addFile('src/index.js', 'console.log("Hello");');
await manager.addFile('package.json', JSON.stringify({
  name: 'my-project',
  version: '1.0.0'
}));

// 3. Pierwszy checkpoint
const v1 = await manager.saveCheckpoint('Initial commit');
console.log('Created version:', v1);
// Output: 'Created version: abc123-def456-...'

// 4. Modyfikacja pliku
await manager.addFile('src/index.js', 'console.log("Hello World");');

// 5. Drugi checkpoint
const v2 = await manager.saveCheckpoint('Add World');
console.log('Created version:', v2);

// 6. Przywrócenie do v1
await manager.restoreVersion(v1);
const content = await manager.getFileContent('src/index.js');
console.log(new TextDecoder().decode(content));
// Output: 'console.log("Hello");'

// 7. Przywrócenie z powrotem do HEAD
await manager.restoreVersion('HEAD');
const content2 = await manager.getFileContent('src/index.js');
console.log(new TextDecoder().decode(content2));
// Output: 'console.log("Hello World");'
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
