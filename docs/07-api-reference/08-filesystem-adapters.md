# FileSystem Adapters API

## Przegląd

FileSystem Adapters zapewniają abstrakcję dostępu do różnych środowisk przechowywania. Każdy adapter implementuje wspólny interfejs `FileSystemAdapter`, umożliwiając JCFManager pracę w przeglądarce, Node.js, Tauri i innych środowiskach.

## Architektura adapterów

```
┌─────────────────┐
│   JCFManager    │
│                 │
│  FileSystem     │
│  Operations     │
└─────────┬───────┘
          │
          │ uses
          ▼
┌─────────────────┐
│ FileSystem      │
│   Adapter       │
│                 │
│ Platform-specific│
│ Implementation  │
└─────────┬───────┘
          │
          │ implements
          ▼
┌─────────────────┐
│   BrowserFS     │  ┌─────────────────┐
│                 │  │   NodeFS        │
├─────────────────┤  ├─────────────────┤
│ IndexedDB       │  │   fs/promises   │
│ File API        │  │   Streams       │
│ Web Streams     │  │   Compression   │
└─────────────────┘  └─────────────────┘
```

## Interfejs FileSystemAdapter

Wszystkie adaptery implementują ten interfejs:

```typescript
interface FileSystemAdapter {
  readonly name: string;
  readonly supportsStreaming: boolean;
  readonly supportsCompression: boolean;
  readonly maxFileSize: number;

  // === Lifecycle ===
  init(options?: AdapterInitOptions): Promise<void>;
  dispose(): Promise<void>;

  // === File Operations ===
  readFile(path: string): Promise<Uint8Array>;
  readFileStream(path: string): Promise<ReadableStream>;

  writeFile(path: string, data: Uint8Array): Promise<void>;
  writeFileStream(path: string, stream: ReadableStream): Promise<void>;

  fileExists(path: string): Promise<boolean>;
  deleteFile(path: string): Promise<void>;
  getFileSize(path: string): Promise<number>;

  // === Directory Operations ===
  listFiles(directory: string): Promise<string[]>;
  createDirectory(path: string): Promise<void>;
  deleteDirectory(path: string): Promise<void>;
  directoryExists(path: string): Promise<boolean>;

  // === Metadata ===
  getMetadata(path: string): Promise<FileMetadata>;

  // === ZIP Operations ===
  createZipWriter(): ZipWriter;
  createZipReader(source: Uint8Array | ReadableStream): Promise<ZipReader>;

  // === Utilities ===
  normalizePath(path: string): string;
  joinPath(...paths: string[]): string;
  getTempDirectory(): string;
}
```

## Wbudowane adaptery

### IndexedDBAdapter
 
 Adapter dla przeglądarek internetowych używający API IndexedDB. Zapewnia szeroką kompatybilność i trwałe przechowywanie.
 
 #### Funkcjonalności
 - ✅ Trwałe przechowywanie danych (Persistent Storage)
 - ✅ Szerokie wsparcie (praktycznie każda nowoczesna przeglądarka)
 - ✅ Izolacja per domena
 
 #### Ograniczenia
 - ❌ Wolniejszy niż OPFS dla dużych plików
 - ❌ Operacje binarne mogą być kosztowne pamięciowo
 
 #### Przykład użycia
 
 ```typescript
 import { JCFManager, IndexedDBAdapter } from 'kamaros-ts';
 
 const manager = await JCFManager.create(new IndexedDBAdapter('my-project'));
 await manager.createProject("MyProject");
 ```

 ### OPFSAdapter (Origin Private File System)
 
 Adapter wykorzystujący nowoczesne API File System Access API (OPFS) dla maksymalnej wydajności w przeglądarce. 
 Wymaga bezpiecznego kontekstu (HTTPS lub localhost).
 
 #### Funkcjonalności
 - ✅ Najwyższa wydajność I/O w przeglądarce
 - ✅ Dostęp do "prawdziwych" plików w prywatnym sandboksie przeglądarki
 - ✅ Efektywne operacje na plikach binarnych
 
 #### Ograniczenia
 - ❌ Wymaga Secure Context (HTTPS/localhost)
 - ❌ Mniejsze wsparcie w starszych przeglądarkach (głównie Chromium/Firefox/Safari modern)
 - ❌ Trudniejszy debugging (pliki są ukryte wewnątrz przeglądarki)
 
 #### Przykład użycia
 
 ```typescript
 import { JCFManager, OPFSAdapter } from 'kamaros-ts';
 
 if (OPFSAdapter.isAvailable()) {
     const manager = await JCFManager.create(new OPFSAdapter('my-project'));
 } else {
     // Fallback to IndexedDB
     const manager = await JCFManager.create(new IndexedDBAdapter('my-project'));
 }
 ```

### NodeAdapter

Adapter dla Node.js używający natywnego systemu plików.

#### Funkcjonalności
- ✅ Pełny dostęp do systemu plików
- ✅ Node.js streams
- ✅ Natywna kompresja (zlib)
- ✅ Symlinks i permissions
- ✅ Duże pliki bez limitów

#### Ograniczenia
- ❌ Tylko środowisko Node.js
- ❌ Brak izolacji (direct FS access)

#### Przykład użycia

```typescript
import { JCFManager, NodeAdapter } from 'jcf-manager';

const manager = new JCFManager();
const adapter = new NodeAdapter({
  rootPath: './my-project',
  createIfMissing: true,
  enableCompression: true
});

await manager.init(adapter);
```

#### Konfiguracja

```typescript
interface NodeAdapterOptions {
  rootPath: string;             // Katalog root projektu
  createIfMissing?: boolean;    // Utwórz katalog jeśli nie istnieje
  enableCompression?: boolean;  // Włącz kompresję
  encoding?: BufferEncoding;    // Kodowanie plików tekstowych
  fsModule?: any;               // Custom fs module (dla testów)
}
```

### TauriAdapter

Adapter dla aplikacji Tauri (Rust + web frontend).

#### Funkcjonalności
- ✅ Dostęp do natywnego systemu plików przez Tauri API
- ✅ Bezpieczeństwo (sandboxed)
- ✅ Kompresja przez Rust
- ✅ Wysoka wydajność

#### Ograniczenia
- ❌ Tylko w aplikacjach Tauri
- ❌ Wymaga konfiguracji Tauri

#### Przykład użycia

```typescript
import { JCFManager, TauriAdapter } from 'jcf-manager';

const manager = new JCFManager();
const adapter = new TauriAdapter({
  appDataDir: true,  // Użyj katalogu aplikacji
  enableCompression: true
});

await manager.init(adapter);
```

#### Konfiguracja

```typescript
interface TauriAdapterOptions {
  basePath?: string;            // Bazowa ścieżka
  appDataDir?: boolean;         // Użyj app data directory
  documentsDir?: boolean;       // Użyj documents directory
  enableCompression?: boolean;  // Włącz kompresję
  chunkSize?: number;           // Rozmiar chunków
}
```

### MemoryAdapter

Adapter in-memory dla testów i tymczasowych operacji.

#### Funkcjonalności
- ✅ Szybki (RAM)
- ✅ Izolowany
- ✅ Wszystkie operacje synchroniczne
- ✅ Serializacja/deserializacja

#### Ograniczenia
- ❌ Dane tracone po zamknięciu
- ❌ Ograniczona pamięć RAM
- ❌ Brak persystencji

#### Przykład użycia

```typescript
import { JCFManager, MemoryAdapter } from 'jcf-manager';

const manager = new JCFManager();
const adapter = new MemoryAdapter();

await manager.init(adapter);

// Użycie dla testów
describe('JCFManager', () => {
  let manager: JCFManager;

  beforeEach(async () => {
    manager = new JCFManager();
    await manager.init(new MemoryAdapter());
  });
});
```

## Tworzenie własnych adapterów

Możesz stworzyć własny adapter implementując interfejs `FileSystemAdapter`.

### Przykład prostego adaptera

```typescript
import { FileSystemAdapter, FileMetadata } from 'jcf-manager';

class CustomAdapter implements FileSystemAdapter {
  readonly name = 'CustomAdapter';
  readonly supportsStreaming = false;
  readonly supportsCompression = false;
  readonly maxFileSize = 10 * 1024 * 1024; // 10MB

  private storage = new Map<string, Uint8Array>();

  async init(): Promise<void> {
    // Inicjalizacja
  }

  async dispose(): Promise<void> {
    this.storage.clear();
  }

  async readFile(path: string): Promise<Uint8Array> {
    const data = this.storage.get(path);
    if (!data) {
      throw new Error(`File not found: ${path}`);
    }
    return data;
  }

  async writeFile(path: string, data: Uint8Array): Promise<void> {
    this.storage.set(path, data);
  }

  async fileExists(path: string): Promise<boolean> {
    return this.storage.has(path);
  }

  async deleteFile(path: string): Promise<void> {
    this.storage.delete(path);
  }

  async getFileSize(path: string): Promise<number> {
    const data = this.storage.get(path);
    return data?.length ?? 0;
  }

  async listFiles(directory: string): Promise<string[]> {
    const prefix = directory.endsWith('/') ? directory : directory + '/';
    return Array.from(this.storage.keys())
      .filter(path => path.startsWith(prefix))
      .map(path => path.slice(prefix.length));
  }

  async createDirectory(): Promise<void> {
    // In-memory - nie trzeba tworzyć katalogów
  }

  async getMetadata(path: string): Promise<FileMetadata> {
    const data = this.storage.get(path);
    if (!data) {
      throw new Error(`File not found: ${path}`);
    }

    return {
      size: data.length,
      created: new Date(),
      modified: new Date(),
      isDirectory: false
    };
  }

  createZipWriter(): ZipWriter {
    throw new Error('ZIP not supported in this adapter');
  }

  async createZipReader(): Promise<ZipReader> {
    throw new Error('ZIP not supported in this adapter');
  }

  normalizePath(path: string): string {
    return path.replace(/\/+/g, '/').replace(/^\/+/, '');
  }

  joinPath(...paths: string[]): string {
    return paths.join('/').replace(/\/+/g, '/');
  }

  getTempDirectory(): string {
    return '/tmp';
  }
}
```

### Zaawansowany adapter z streaming

```typescript
class StreamingAdapter implements FileSystemAdapter {
  readonly name = 'StreamingAdapter';
  readonly supportsStreaming = true;
  readonly supportsCompression = true;
  readonly maxFileSize = 1024 * 1024 * 1024; // 1GB

  async readFileStream(path: string): Promise<ReadableStream> {
    // Implementacja streaming read
    const response = await fetch(`/api/files/${encodeURIComponent(path)}`);
    return response.body!;
  }

  async writeFileStream(path: string, stream: ReadableStream): Promise<void> {
    // Implementacja streaming write
    const response = await fetch(`/api/files/${encodeURIComponent(path)}`, {
      method: 'PUT',
      body: stream
    });

    if (!response.ok) {
      throw new Error(`Upload failed: ${response.statusText}`);
    }
  }

  // ... pozostałe metody
}
```

## Adapter Factory

Możesz używać factory function do automatycznego wyboru adaptera:

```typescript
import {
  JCFManager,
  createAdapter,
  BrowserAdapter,
  NodeAdapter,
  TauriAdapter
} from 'jcf-manager';

// Automatyczny wybór adaptera
const adapter = createAdapter({
  // Opcje wspólne dla wszystkich adapterów
  maxFileSize: 500 * 1024 * 1024,
  enableCompression: true
});

const manager = new JCFManager();
await manager.init(adapter);
```

## Testowanie adapterów

Biblioteka zawiera wspólny test suite dla wszystkich adapterów:

```typescript
import { testAdapter } from 'jcf-manager/test-utils';

describe('My Custom Adapter', () => {
  testAdapter({
    createAdapter: () => new MyAdapter(),
    supportsStreaming: false,
    supportsCompression: true
  });
});
```

## Bezpieczeństwo i wydajność

### Zasady bezpieczeństwa

1. **Sandboxing**: Adaptery powinny izolować dostęp do danych
2. **Validation**: Walidacja wszystkich ścieżek i danych wejściowych
3. **Permissions**: Minimalne wymagane uprawnienia
4. **Encryption**: Opcjonalne szyfrowanie danych

### Optymalizacje wydajności

1. **Chunking**: Podział dużych plików na mniejsze części
2. **Caching**: Cache dla często używanych danych
3. **Lazy Loading**: Ładowanie danych na żądanie
4. **Compression**: Automatyczna kompresja dużych plików
5. **Parallel Operations**: Równoległe przetwarzanie gdzie możliwe

### Metryki wydajności

Adaptery mogą udostępniać metryki:

```typescript
interface AdapterMetrics {
  readLatency: number;    // Średni czas odczytu (ms)
  writeLatency: number;   // Średni czas zapisu (ms)
  throughput: number;     // Przepustowość (bytes/s)
  errorRate: number;      // Częstość błędów
  cacheHitRate: number;   // Efektywność cache
}

// Pobierz metryki
const metrics = await adapter.getMetrics();
console.log(`Read latency: ${metrics.readLatency}ms`);
```

---

**Zobacz również:**
- [JCFManager Class](01-jcf-manager-class.md) - Główny interfejs API
- [TypeScript Types](05-typescript-types.md) - Typy używane przez adaptery
- [Error Handling](09-error-handling.md) - Obsługa błędów w adapterach