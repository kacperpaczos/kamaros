# Query Methods

> **Szczegółowa dokumentacja metod odczytu danych z JCFManager**

[← Back: Core Methods](02-core-methods.md) | [Next: Utility Methods →](04-utility-methods.md)

---

## Overview

Query Methods to metody odpowiedzialne za odczyt danych z projektu - plików, historii wersji, metadanych. Te metody są **read-only** i nie modyfikują stanu projektu.

---

## getFileContent(path, versionId?)

Pobiera zawartość pliku z wskazanej wersji (domyślnie HEAD).

### Sygnatura

```typescript
async getFileContent(
  path: string,
  versionId?: string
): Promise<Uint8Array>
```

### Parametry

- **path** (`string`): Ścieżka pliku
- **versionId** (`string?`): ID wersji (opcjonalne, domyślnie HEAD)

### Zwraca

`Promise<Uint8Array>` - Zawartość pliku jako bajty

### Przykłady

```typescript
// Zawartość z HEAD
const content = await manager.getFileContent('src/index.js');
const text = new TextDecoder().decode(content);

// Zawartość z konkretnej wersji
const oldContent = await manager.getFileContent('src/index.js', 'v1.0');

// Binary file
const imageData = await manager.getFileContent('assets/logo.png');
```

### Błędy

- `FileNotFoundError` - Plik nie istnieje w wskazanej wersji
- `VersionNotFoundError` - Wersja nie istnieje

### Implementacja

```typescript
async getFileContent(path: string, versionId?: string): Promise<Uint8Array> {
  const targetVersionId = versionId || this.manifest.refs.head;
  
  if (targetVersionId === this.manifest.refs.head) {
    // HEAD - read directly from content/
    return await this.adapter.readFile(`content/${path}`);
  } else {
    // Older version - reconstruct from patches/blobs
    return await this.reconstructFileFromVersion(path, targetVersionId);
  }
}
```

---

## getFileInfo(path, versionId?)

Pobiera informacje o pliku (metadane, bez zawartości).

### Sygnatura

```typescript
async getFileInfo(
  path: string,
  versionId?: string
): Promise<FileInfo>
```

### Parametry

- **path** (`string`): Ścieżka pliku
- **versionId** (`string?`): ID wersji (opcjonalne)

### Zwraca

`Promise<FileInfo>` - Informacje o pliku

```typescript
interface FileInfo {
  path: string;
  inodeId: string;
  type: 'text' | 'binary';
  size: number;
  hash?: string;              // SHA-256 dla binariów
  encoding?: string;          // Dla plików tekstowych
  mime?: string;              // MIME type
  created: string;            // ISO timestamp
  modified: string;           // ISO timestamp
  versionId?: string;        // Wersja z której pochodzą dane
}
```

### Przykłady

```typescript
const info = await manager.getFileInfo('src/index.js');
console.log(`File: ${info.path}`);
console.log(`Size: ${info.size} bytes`);
console.log(`Type: ${info.type}`);
console.log(`Modified: ${info.modified}`);

// Info z konkretnej wersji
const oldInfo = await manager.getFileInfo('src/index.js', 'v1.0');
```

---

## listFiles(directory?, options?)

Lista plików w katalogu z opcjonalnymi filtrami.

### Sygnatura

```typescript
async listFiles(
  directory?: string,
  options?: ListFilesOptions
): Promise<FileInfo[]>
```

### Parametry

- **directory** (`string?`): Katalog do przeszukania (domyślnie: root)
- **options** (`ListFilesOptions?`): Opcje listowania
  ```typescript
  interface ListFilesOptions {
    recursive?: boolean;           // Przeszukiwanie rekursywne (domyślnie: false)
    includeMetadata?: boolean;     // Dołącz metadane (domyślnie: true)
    type?: 'text' | 'binary';      // Filtrowanie po typie
    pattern?: string;              // Glob pattern (np. '*.js')
    versionId?: string;            // Lista plików z konkretnej wersji
  }
  ```

### Zwraca

`Promise<FileInfo[]>` - Lista plików z informacjami

### Przykłady

```typescript
// Wszystkie pliki w katalogu
const files = await manager.listFiles('src/');

// Rekursywnie wszystkie pliki
const allFiles = await manager.listFiles('', {
  recursive: true
});

// Tylko pliki JavaScript
const jsFiles = await manager.listFiles('src/', {
  pattern: '*.js',
  recursive: true
});

// Tylko pliki binarne
const binaryFiles = await manager.listFiles('assets/', {
  type: 'binary'
});

// Pliki z konkretnej wersji
const oldFiles = await manager.listFiles('', {
  versionId: 'v1.0',
  recursive: true
});
```

---

## getHistory(options?)

Pobiera historię wersji z opcjonalnymi filtrami.

### Sygnatura

```typescript
async getHistory(
  options?: HistoryOptions
): Promise<Version[]>
```

### Parametry

- **options** (`HistoryOptions?`): Opcje filtrowania
  ```typescript
  interface HistoryOptions {
    limit?: number;                // Maksymalna liczba wersji (domyślnie: wszystkie)
    since?: string;                // Tylko wersje od daty (ISO 8601)
    until?: string;                // Tylko wersje do daty (ISO 8601)
    author?: string;               // Filtrowanie po autorze
    message?: string;              // Filtrowanie po wiadomości (substring)
    tags?: string[];               // Tylko wersje z tymi tagami
    includeFileChanges?: boolean;  // Dołącz informacje o zmienionych plikach
    includeStats?: boolean;        // Dołącz statystyki (liczba plików, rozmiar)
  }
  ```

### Zwraca

`Promise<Version[]>` - Lista wersji od najnowszej do najstarszej

```typescript
interface Version {
  id: string;
  timestamp: string;
  message: string;
  author: string;
  email?: string;
  parentId: string | null;
  fileStates?: Record<string, FileState>;  // Jeśli includeFileChanges: true
  tags?: string[];
  stats?: VersionStats;                    // Jeśli includeStats: true
}
```

### Przykłady

```typescript
// Wszystkie wersje
const history = await manager.getHistory();

// Ostatnie 10 wersji
const recent = await manager.getHistory({ limit: 10 });

// Wersje od konkretnej daty
const since = await manager.getHistory({
  since: '2025-01-01T00:00:00Z'
});

// Wersje konkretnego autora
const byAuthor = await manager.getHistory({
  author: 'Jan Kowalski'
});

// Wersje z tagiem 'release'
const releases = await manager.getHistory({
  tags: ['release']
});

// Historia ze szczegółami
const detailed = await manager.getHistory({
  includeFileChanges: true,
  includeStats: true,
  limit: 20
});
```

---

## getVersion(versionId)

Pobiera szczegóły konkretnej wersji.

### Sygnatura

```typescript
async getVersion(versionId: string): Promise<Version | null>
```

### Parametry

- **versionId** (`string`): ID wersji lub tag

### Zwraca

`Promise<Version | null>` - Szczegóły wersji lub null jeśli nie istnieje

### Przykłady

```typescript
const version = await manager.getVersion('abc123-def456-...');
if (version) {
  console.log(`Version ${version.id}: ${version.message}`);
  console.log(`Author: ${version.author} at ${version.timestamp}`);
  console.log(`Files changed: ${Object.keys(version.fileStates || {}).length}`);
}

// Użycie tagu
const v1 = await manager.getVersion('v1.0');
```

---

## getVersionDiff(fromVersion, toVersion)

Porównuje dwie wersje i zwraca różnice.

### Sygnatura

```typescript
async getVersionDiff(
  fromVersion: string,
  toVersion: string
): Promise<VersionDiff>
```

### Parametry

- **fromVersion** (`string`): Bazowa wersja
- **toVersion** (`string`): Docelowa wersja

### Zwraca

`Promise<VersionDiff>` - Różnice między wersjami

```typescript
interface VersionDiff {
  fromVersion: string;
  toVersion: string;
  added: string[];           // Pliki dodane
  modified: string[];        // Pliki zmodyfikowane
  deleted: string[];        // Pliki usunięte
  renamed: RenameDiff[];    // Pliki przemianowane
  stats: {
    filesAdded: number;
    filesModified: number;
    filesDeleted: number;
    filesRenamed: number;
    totalSizeChange: number;  // Zmiana rozmiaru w bajtach
  };
}

interface RenameDiff {
  inodeId: string;
  fromPath: string;
  toPath: string;
}
```

### Przykłady

```typescript
const diff = await manager.getVersionDiff('v1.0', 'v1.1');
console.log('Added files:', diff.added);
console.log('Modified files:', diff.modified);
console.log('Deleted files:', diff.deleted);
console.log('Renamed files:', diff.renamed);

// Statystyki
console.log(`Total changes: ${diff.stats.filesAdded + diff.stats.filesModified}`);
console.log(`Size change: ${diff.stats.totalSizeChange} bytes`);
```

---

## getFileHistory(path, options?)

Pobiera historię zmian konkretnego pliku (włącznie z renames).

### Sygnatura

```typescript
async getFileHistory(
  path: string,
  options?: FileHistoryOptions
): Promise<FileHistoryEntry[]>
```

### Parametry

- **path** (`string`): Ścieżka pliku (może być aktualna lub historyczna)
- **options** (`FileHistoryOptions?`): Opcje
  ```typescript
  interface FileHistoryOptions {
    limit?: number;              // Maksymalna liczba wpisów
    includeContent?: boolean;     // Dołącz zawartość plików (domyślnie: false)
    includeDiff?: boolean;       // Dołącz diff zmian (domyślnie: false)
  }
  ```

### Zwraca

`Promise<FileHistoryEntry[]>` - Historia zmian pliku

```typescript
interface FileHistoryEntry {
  versionId: string;
  path: string;                 // Ścieżka w tej wersji (może się zmieniać przez renames)
  timestamp: string;
  changeType: 'added' | 'modified' | 'deleted' | 'renamed';
  size: number;
  hash?: string;
  content?: Uint8Array;         // Jeśli includeContent: true
  diff?: string;                 // Jeśli includeDiff: true
  author: string;
  message: string;               // Commit message
}
```

### Przykłady

```typescript
// Historia bez zawartości
const history = await manager.getFileHistory('src/index.js', {
  limit: 20
});

history.forEach(entry => {
  console.log(`${entry.timestamp}: ${entry.changeType} (${entry.size} bytes)`);
  if (entry.changeType === 'renamed') {
    console.log(`  Renamed in version ${entry.versionId}`);
  }
});

// Historia z diff
const historyWithDiff = await manager.getFileHistory('src/index.js', {
  includeDiff: true,
  limit: 10
});

// Historia z zawartością (uważaj - może być dużo danych!)
const historyWithContent = await manager.getFileHistory('src/index.js', {
  includeContent: true,
  limit: 5  // Ogranicz do 5 ostatnich wersji
});
```

**Ważne**: Historia automatycznie śledzi renames dzięki inodeId - nawet jeśli plik został przemianowany, historia jest kompletna.

---

## getCurrentVersion()

Pobiera ID bieżącej wersji (HEAD).

### Sygnatura

```typescript
async getCurrentVersion(): Promise<string>
```

### Zwraca

`Promise<string>` - ID wersji HEAD

### Przykłady

```typescript
const headId = await manager.getCurrentVersion();
console.log('Current HEAD:', headId);

// Porównanie
if (headId === targetVersionId) {
  console.log('Already at target version');
}
```

---

## Podsumowanie Query Methods

| Metoda | Purpose | Complexity | Returns |
|--------|---------|------------|---------|
| `getFileContent()` | Zawartość pliku | O(1) dla HEAD, O(V) dla starych | `Uint8Array` |
| `getFileInfo()` | Metadane pliku | O(1) | `FileInfo` |
| `listFiles()` | Lista plików | O(F) | `FileInfo[]` |
| `getHistory()` | Historia wersji | O(V) | `Version[]` |
| `getVersion()` | Szczegóły wersji | O(1) | `Version \| null` |
| `getVersionDiff()` | Różnice wersji | O(F) | `VersionDiff` |
| `getFileHistory()` | Historia pliku | O(V) | `FileHistoryEntry[]` |
| `getCurrentVersion()` | ID HEAD | O(1) | `string` |

**Legenda**:
- F = liczba plików
- V = liczba wersji

---

## Best Practices

### Do's ✅

1. **Używaj `getFileInfo()`** zamiast `getFileContent()` jeśli potrzebujesz tylko metadanych
2. **Ograniczaj `limit`** w `getHistory()` dla lepszej wydajności
3. **Używaj `includeContent: false`** w `getFileHistory()` jeśli nie potrzebujesz zawartości
4. **Cache wyniki** jeśli są używane wielokrotnie

### Don'ts ❌

1. **Nie pobieraj zawartości** jeśli potrzebujesz tylko metadanych
2. **Nie używaj `includeContent: true`** dla wielu plików naraz
3. **Nie ignoruj błędów** - `FileNotFoundError` może oznaczać rename

---

## Kompletny Przykład Użycia Query Methods

```typescript
// 1. Pobranie zawartości pliku
const content = await manager.getFileContent('src/index.js');
const text = new TextDecoder().decode(content);
console.log('File content:', text);

// 2. Pobranie informacji o pliku
const info = await manager.getFileInfo('src/index.js');
console.log('File info:', {
  path: info.path,           // 'src/index.js'
  size: info.size,           // 1024
  type: info.type,           // 'text'
  hash: info.hash,           // undefined (text files don't have hash)
  modified: info.modified    // '2025-01-15T10:30:00Z'
});

// 3. Lista plików
const allFiles = await manager.listFiles('', { recursive: true });
console.log(`Total files: ${allFiles.length}`);

const jsFiles = await manager.listFiles('src/', {
  pattern: '*.js',
  recursive: true
});
console.log(`JavaScript files: ${jsFiles.length}`);

// 4. Historia wersji
const history = await manager.getHistory({ limit: 10 });
history.forEach(version => {
  console.log(`${version.id}: ${version.message} (${version.author})`);
});

// 5. Historia konkretnego pliku (z renames)
const fileHistory = await manager.getFileHistory('src/main.js');
// Jeśli plik był przemianowany z 'src/index.js', historia zawiera obie ścieżki
fileHistory.forEach(entry => {
  console.log(`${entry.timestamp}: ${entry.path} - ${entry.changeType}`);
  // Output:
  // '2025-01-10T10:00:00Z: src/index.js - added'
  // '2025-01-15T10:30:00Z: src/main.js - renamed'
});

// 6. Porównanie wersji
const diff = await manager.getVersionDiff('v1.0', 'v1.1');
console.log('Added:', diff.added);
console.log('Modified:', diff.modified);
console.log('Deleted:', diff.deleted);
console.log('Renamed:', diff.renamed);
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
