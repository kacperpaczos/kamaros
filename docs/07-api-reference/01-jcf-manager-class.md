# API Reference: JCFManager

## Przegląd

`JCFManager` to główna klasa biblioteki JCF, zapewniająca interfejs do wszystkich operacji na plikach projektowych z wersjonowaniem.

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

## Konfiguracja

```typescript
interface JCFConfig {
  author?: string;           // Domyślny autor commitów
  email?: string;            // Email autora
  compressionLevel?: number; // Poziom kompresji (0-9)
  autoGC?: boolean;          // Automatyczne GC
  maxHistorySize?: number;   // Max rozmiar historii (MB)
}
```

## Metody główne

### init(adapter: FileSystemAdapter)
Inicjalizuje manager z adapterem systemu plików.

```typescript
await manager.init(new BrowserAdapter());
```

### saveCheckpoint(message: string)
Tworzy nowy checkpoint z bieżącymi zmianami.

```typescript
const versionId = await manager.saveCheckpoint('Add login feature');
```

**Zwraca**: `Promise<string>` - ID nowej wersji

### restoreVersion(versionId: string)
Przywraca projekt do wskazanej wersji.

```typescript
await manager.restoreVersion('v1');
```

### addFile(path: string, content: Uint8Array | string)
Dodaje plik do projektu.

```typescript
await manager.addFile('src/index.js', 'console.log("Hello");');
```

### getFileContent(path: string)
Pobiera zawartość pliku z HEAD.

```typescript
const content = await manager.getFileContent('src/index.js');
```

**Zwraca**: `Promise<Uint8Array>`

### listFiles(directory?: string)
Lista plików w katalogu.

```typescript
const files = await manager.listFiles('src/');
```

**Zwraca**: `Promise<string[]>`

## Metody zarządzania wersjami

### getHistory()
Pobiera historię wersji.

```typescript
const history = await manager.getHistory();
```

**Zwraca**: `Promise<Version[]>`

### getVersion(versionId: string)
Pobiera szczegóły wersji.

```typescript
const version = await manager.getVersion('v1');
```

**Zwraca**: `Promise<Version | null>`

### getCurrentVersion()
Pobiera ID bieżącej wersji (HEAD).

```typescript
const headId = await manager.getCurrentVersion();
```

**Zwraca**: `Promise<string>`

## Metody narzędziowe

### runGC()
Uruchamia garbage collection.

```typescript
const report = await manager.runGC();
```

**Zwraca**: `Promise<GCReport>`

### export()
Eksportuje projekt jako stream.

```typescript
const stream = await manager.export();
```

**Zwraca**: `Promise<ReadableStream>`

### import(stream: ReadableStream)
Importuje projekt ze streama.

```typescript
await manager.import(zipStream);
```

## Zdarzenia

Manager emituje zdarzenia podczas długotrwałych operacji:

```typescript
manager.on('checkpoint:start', (data) => {
  console.log('Starting checkpoint:', data.message);
});

manager.on('checkpoint:progress', (data) => {
  console.log(`Progress: ${data.percent}%`);
});

manager.on('checkpoint:complete', (data) => {
  console.log('Checkpoint complete:', data.versionId);
});
```

### Zdarzenia dostępne
- `checkpoint:start` - Rozpoczęcie checkpoint
- `checkpoint:progress` - Progress checkpoint
- `checkpoint:complete` - Zakończenie checkpoint
- `checkpoint:error` - Błąd checkpoint
- `restore:start` - Rozpoczęcie restore
- `restore:progress` - Progress restore
- `restore:complete` - Zakończenie restore
- `gc:complete` - Zakończenie GC

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

## Błędy

Manager może rzucać następujące błędy:

- `ValidationError` - Nieprawidłowe dane wejściowe
- `FileNotFoundError` - Plik nie istnieje
- `VersionNotFoundError` - Wersja nie istnieje
- `StorageError` - Błąd systemu plików
- `CorruptionError` - Uszkodzone dane

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