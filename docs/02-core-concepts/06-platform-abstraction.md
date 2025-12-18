# 🔌 Adapter Pattern - Abstrakcja Systemu Plików

## 1. Wprowadzenie

**Adapter Pattern** w JCF Manager pozwala na **unifikację dostępu do różnych środowisk** (Browser, Node.js, Tauri, Deno) poprzez wspólny interfejs. To klucz do izomorficzności biblioteki.

## 2. Problem i Rozwiązanie

### 2.1 Problem: Różne API dla I/O

**Browser**:
```typescript
// IndexedDB + File API
const db = indexedDB.open('jcf-storage');
const file = new File([data], 'project.jcf');
```

**Node.js**:
```typescript
// fs/promises
import { readFile, writeFile } from 'fs/promises';
await writeFile('project.jcf', data);
```

**Tauri**:
```typescript
// @tauri-apps/api/fs
import { writeBinaryFile } from '@tauri-apps/api/fs';
await writeBinaryFile('project.jcf', data);
```

**Problem**: Core logic musiałby znać wszystkie API = niemożliwe do utrzymania

### 2.2 Rozwiązanie: Adapter Pattern

```typescript
// Core używa abstrakcyjnego interfejsu
interface FileSystemAdapter {
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  // ...
}

// Każda platforma ma swoją implementację
class BrowserAdapter implements FileSystemAdapter { /*...*/ }
class NodeAdapter implements FileSystemAdapter { /*...*/ }
class TauriAdapter implements FileSystemAdapter { /*...*/ }

// Core jest platform-agnostic
class JCFManager {
  constructor(private adapter: FileSystemAdapter) {}
  
  async save() {
    // Używa adapter.writeFile() - nie wie czy to browser czy node
    await this.adapter.writeFile('project.jcf', data);
  }
}
```

**Zalety**:
- ✅ Single source of truth (core logic)
- ✅ Easy testing (MockAdapter)
- ✅ Future-proof (nowa platforma = nowy adapter, core niezmieniony)
- ✅ Clean separation of concerns

## 3. Interfejs FileSystemAdapter

### 3.1 Core Interface

```typescript
/**
 * Abstrakcja dostępu do systemu plików
 * Implementowana per platforma (Browser, Node, Tauri, etc.)
 */
interface FileSystemAdapter {
  /**
   * Nazwa adaptera (dla debugowania)
   */
  readonly name: string;
  
  /**
   * Czy adapter wspiera streaming?
   */
  readonly supportsStreaming: boolean;
  
  /**
   * Inicjalizacja adaptera (np. otworzenie DB)
   */
  init(): Promise<void>;
  
  /**
   * Cleanup (zamknięcie połączeń, etc.)
   */
  dispose(): Promise<void>;
  
  // === FILE OPERATIONS ===
  
  /**
   * Odczyt pliku
   */
  readFile(path: string): Promise<Uint8Array>;
  
  /**
   * Odczyt pliku jako stream (jeśli wspierane)
   */
  readFileStream(path: string): Promise<ReadableStream>;
  
  /**
   * Zapis pliku
   */
  writeFile(path: string, data: Uint8Array): Promise<void>;
  
  /**
   * Zapis pliku ze streama
   */
  writeFileStream(path: string, stream: ReadableStream): Promise<void>;
  
  /**
   * Sprawdź czy plik istnieje
   */
  fileExists(path: string): Promise<boolean>;
  
  /**
   * Usuń plik
   */
  deleteFile(path: string): Promise<void>;
  
  /**
   * Pobierz rozmiar pliku
   */
  getFileSize(path: string): Promise<number>;
  
  /**
   * Lista plików w katalogu
   */
  listFiles(directory: string): Promise<string[]>;
  
  // === ZIP OPERATIONS ===
  
  /**
   * Stwórz ZIP writer
   */
  createZipWriter(): ZipWriter;
  
  /**
   * Stwórz ZIP reader
   */
  createZipReader(source: Uint8Array | ReadableStream): ZipReader;
  
  // === METADATA ===
  
  /**
   * Pobierz metadata pliku
   */
  getMetadata(path: string): Promise<FileMetadata>;
}

interface FileMetadata {
  size: number;
  created: Date;
  modified: Date;
  isDirectory: boolean;
}
```

### 3.2 ZIP Writer Interface

```typescript
interface ZipWriter {
  /**
   * Dodaj plik do ZIP
   */
  addFile(
    path: string,
    content: Uint8Array | string,
    options?: ZipFileOptions
  ): Promise<void>;
  
  /**
   * Dodaj plik ze streama
   */
  addFileStream(
    path: string,
    stream: ReadableStream,
    options?: ZipFileOptions
  ): Promise<void>;
  
  /**
   * Finalizuj ZIP i zwróć jako stream
   */
  finalize(): Promise<ReadableStream>;
  
  /**
   * Anuluj operację
   */
  cancel(): Promise<void>;
}

interface ZipFileOptions {
  compression?: 'store' | 'deflate';
  compressionLevel?: number; // 0-9
  mtime?: Date;
}
```

### 3.2 ZIP Reader Interface

```typescript
interface ZipReader {
  /**
   * Lista plików w ZIP
   */
  list(): Promise<ZipEntry[]>;
  
  /**
   * Odczyt pliku z ZIP
   */
  readFile(path: string): Promise<Uint8Array>;
  
  /**
   * Odczyt pliku jako stream
   */
  readFileStream(path: string): Promise<ReadableStream>;
  
  /**
   * Sprawdź czy plik istnieje w ZIP
   */
  hasFile(path: string): Promise<boolean>;
  
  /**
   * Zamknij reader
   */
  close(): Promise<void>;
}

interface ZipEntry {
  path: string;
  size: number;
  compressedSize: number;
  compression: 'store' | 'deflate';
  crc32: number;
  mtime: Date;
}
```

## 4. Implementacje Adapterów

### 4.1 BrowserAdapter (IndexedDB)

**Użycie**: Progressive Web Apps, Electron (renderer), Web Applications

**Storage**: IndexedDB (persistentny) + Memory (cache)

```typescript
class BrowserAdapter implements FileSystemAdapter {
  readonly name = 'BrowserAdapter';
  readonly supportsStreaming = true;
  
  private db: IDBDatabase | null = null;
  private dbName = 'jcf-storage';
  private storeName = 'files';
  
  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, 1);
      
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };
      
      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        
        if (!db.objectStoreNames.contains(this.storeName)) {
          const store = db.createObjectStore(this.storeName, {
            keyPath: 'path'
          });
          store.createIndex('size', 'size', { unique: false });
          store.createIndex('modified', 'modified', { unique: false });
        }
      };
    });
  }
  
  async dispose(): Promise<void> {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }
  
  async readFile(path: string): Promise<Uint8Array> {
    if (!this.db) throw new Error('Adapter not initialized');
    
    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.get(path);
      
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const result = request.result;
        if (!result) {
          reject(new Error(`File not found: ${path}`));
        } else {
          resolve(result.data);
        }
      };
    });
  }
  
  async writeFile(path: string, data: Uint8Array): Promise<void> {
    if (!this.db) throw new Error('Adapter not initialized');
    
    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);
      
      const record = {
        path,
        data,
        size: data.byteLength,
        modified: new Date()
      };
      
      const request = store.put(record);
      
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  }
  
  async readFileStream(path: string): Promise<ReadableStream> {
    // IndexedDB doesn't support streaming directly
    // Load to memory and create stream
    const data = await this.readFile(path);
    
    return new ReadableStream({
      start(controller) {
        controller.enqueue(data);
        controller.close();
      }
    });
  }
  
  async fileExists(path: string): Promise<boolean> {
    if (!this.db) return false;
    
    return new Promise((resolve) => {
      const transaction = this.db!.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.count(IDBKeyRange.only(path));
      
      request.onsuccess = () => resolve(request.result > 0);
      request.onerror = () => resolve(false);
    });
  }
  
  createZipWriter(): ZipWriter {
    return new FflateZipWriter();
  }
  
  createZipReader(source: Uint8Array | ReadableStream): ZipReader {
    return new FflateZipReader(source);
  }
}
```

**Limit Storage**:
```typescript
// Check available storage
async function checkStorageQuota(): Promise<StorageEstimate> {
  if ('storage' in navigator && 'estimate' in navigator.storage) {
    return await navigator.storage.estimate();
  }
  
  return { usage: 0, quota: Infinity };
}

// Before write
const estimate = await checkStorageQuota();
if (estimate.usage! + data.byteLength > estimate.quota! * 0.9) {
  throw new Error('Storage quota exceeded');
}
```

### 4.2 NodeAdapter (fs/promises)

**Użycie**: CLI tools, Node.js servers, Electron (main process)

**Storage**: Native filesystem

```typescript
import { readFile, writeFile, stat, unlink, mkdir, readdir } from 'fs/promises';
import { createReadStream, createWriteStream } from 'fs';
import { dirname } from 'path';

class NodeAdapter implements FileSystemAdapter {
  readonly name = 'NodeAdapter';
  readonly supportsStreaming = true;
  
  private basePath: string;
  
  constructor(basePath: string = process.cwd()) {
    this.basePath = basePath;
  }
  
  async init(): Promise<void> {
    // Ensure base directory exists
    await mkdir(this.basePath, { recursive: true });
  }
  
  async dispose(): Promise<void> {
    // Nothing to cleanup
  }
  
  private resolvePath(path: string): string {
    return `${this.basePath}/${path}`;
  }
  
  async readFile(path: string): Promise<Uint8Array> {
    const fullPath = this.resolvePath(path);
    const buffer = await readFile(fullPath);
    return new Uint8Array(buffer);
  }
  
  async readFileStream(path: string): Promise<ReadableStream> {
    const fullPath = this.resolvePath(path);
    const nodeStream = createReadStream(fullPath);
    
    // Convert Node.js stream to Web Stream
    return Readable.toWeb(nodeStream) as ReadableStream;
  }
  
  async writeFile(path: string, data: Uint8Array): Promise<void> {
    const fullPath = this.resolvePath(path);
    
    // Ensure directory exists
    await mkdir(dirname(fullPath), { recursive: true });
    
    // Write with atomic operation (write to temp, then rename)
    const tempPath = `${fullPath}.tmp`;
    
    try {
      await writeFile(tempPath, data);
      await rename(tempPath, fullPath);
    } catch (error) {
      // Cleanup temp file on error
      try {
        await unlink(tempPath);
      } catch {}
      throw error;
    }
  }
  
  async writeFileStream(
    path: string,
    stream: ReadableStream
  ): Promise<void> {
    const fullPath = this.resolvePath(path);
    
    // Ensure directory exists
    await mkdir(dirname(fullPath), { recursive: true });
    
    // Convert Web Stream to Node.js stream
    const nodeStream = Readable.fromWeb(stream as any);
    const writeStream = createWriteStream(fullPath);
    
    return new Promise((resolve, reject) => {
      nodeStream.pipe(writeStream);
      writeStream.on('finish', resolve);
      writeStream.on('error', reject);
    });
  }
  
  async fileExists(path: string): Promise<boolean> {
    try {
      await stat(this.resolvePath(path));
      return true;
    } catch {
      return false;
    }
  }
  
  async deleteFile(path: string): Promise<void> {
    await unlink(this.resolvePath(path));
  }
  
  async getFileSize(path: string): Promise<number> {
    const stats = await stat(this.resolvePath(path));
    return stats.size;
  }
  
  async listFiles(directory: string): Promise<string[]> {
    const fullPath = this.resolvePath(directory);
    return await readdir(fullPath);
  }
  
  createZipWriter(): ZipWriter {
    return new FflateZipWriter();
  }
  
  createZipReader(source: Uint8Array | ReadableStream): ZipReader {
    return new FflateZipReader(source);
  }
}
```

### 4.3 TauriAdapter

**Użycie**: Tauri desktop applications

**Storage**: Native filesystem via Tauri API

```typescript
import { 
  readBinaryFile, 
  writeBinaryFile, 
  exists, 
  removeFile,
  BaseDirectory 
} from '@tauri-apps/api/fs';
import { appDataDir } from '@tauri-apps/api/path';

class TauriAdapter implements FileSystemAdapter {
  readonly name = 'TauriAdapter';
  readonly supportsStreaming = false; // Tauri API doesn't support streaming yet
  
  private baseDir: string = '';
  
  async init(): Promise<void> {
    // Get app data directory
    this.baseDir = await appDataDir();
    console.log(`Tauri base directory: ${this.baseDir}`);
  }
  
  async dispose(): Promise<void> {
    // Nothing to cleanup
  }
  
  async readFile(path: string): Promise<Uint8Array> {
    try {
      const data = await readBinaryFile(path, {
        dir: BaseDirectory.AppData
      });
      return data;
    } catch (error) {
      throw new Error(`Failed to read file: ${path}`);
    }
  }
  
  async readFileStream(path: string): Promise<ReadableStream> {
    // Fallback: load to memory and create stream
    const data = await this.readFile(path);
    
    return new ReadableStream({
      start(controller) {
        controller.enqueue(data);
        controller.close();
      }
    });
  }
  
  async writeFile(path: string, data: Uint8Array): Promise<void> {
    await writeBinaryFile(path, data, {
      dir: BaseDirectory.AppData
    });
  }
  
  async writeFileStream(
    path: string,
    stream: ReadableStream
  ): Promise<void> {
    // Fallback: collect stream to memory then write
    const chunks: Uint8Array[] = [];
    const reader = stream.getReader();
    
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
      }
      
      const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
      const data = new Uint8Array(totalLength);
      let offset = 0;
      
      for (const chunk of chunks) {
        data.set(chunk, offset);
        offset += chunk.length;
      }
      
      await this.writeFile(path, data);
      
    } finally {
      reader.releaseLock();
    }
  }
  
  async fileExists(path: string): Promise<boolean> {
    return await exists(path, {
      dir: BaseDirectory.AppData
    });
  }
  
  async deleteFile(path: string): Promise<void> {
    await removeFile(path, {
      dir: BaseDirectory.AppData
    });
  }
  
  createZipWriter(): ZipWriter {
    return new FflateZipWriter();
  }
  
  createZipReader(source: Uint8Array | ReadableStream): ZipReader {
    return new FflateZipReader(source);
  }
}
```

### 4.4 MemoryAdapter (Testing)

**Użycie**: Unit tests, in-memory prototyping

```typescript
class MemoryAdapter implements FileSystemAdapter {
  readonly name = 'MemoryAdapter';
  readonly supportsStreaming = true;
  
  private files = new Map<string, Uint8Array>();
  private metadata = new Map<string, FileMetadata>();
  
  async init(): Promise<void> {
    this.files.clear();
    this.metadata.clear();
  }
  
  async dispose(): Promise<void> {
    this.files.clear();
    this.metadata.clear();
  }
  
  async readFile(path: string): Promise<Uint8Array> {
    const data = this.files.get(path);
    if (!data) {
      throw new Error(`File not found: ${path}`);
    }
    return data;
  }
  
  async writeFile(path: string, data: Uint8Array): Promise<void> {
    this.files.set(path, data);
    this.metadata.set(path, {
      size: data.byteLength,
      created: new Date(),
      modified: new Date(),
      isDirectory: false
    });
  }
  
  async fileExists(path: string): Promise<boolean> {
    return this.files.has(path);
  }
  
  async deleteFile(path: string): Promise<void> {
    this.files.delete(path);
    this.metadata.delete(path);
  }
  
  async listFiles(directory: string): Promise<string[]> {
    return Array.from(this.files.keys())
      .filter(path => path.startsWith(directory));
  }
  
  // ... rest of interface implementation
  
  // Test helpers
  getStorageSize(): number {
    let total = 0;
    for (const data of this.files.values()) {
      total += data.byteLength;
    }
    return total;
  }
  
  dump(): Record<string, Uint8Array> {
    return Object.fromEntries(this.files);
  }
}
```

## 5. Użycie w JCFManager

```typescript
class JCFManager {
  private adapter: FileSystemAdapter;
  
  constructor() {
    // Adapter będzie ustawiony w init()
  }
  
  async init(
    adapter: FileSystemAdapter,
    existingData?: Uint8Array
  ): Promise<void> {
    this.adapter = adapter;
    await this.adapter.init();
    
    if (existingData) {
      await this.loadFromData(existingData);
    } else {
      await this.createNew();
    }
  }
  
  async saveCheckpoint(message: string): Promise<string> {
    // ... logic ...
    
    // Write using adapter
    await this.adapter.writeFile('project.jcf', zipData);
    
    return versionId;
  }
  
  async export(): Promise<ReadableStream> {
    if (this.adapter.supportsStreaming) {
      return await this.adapter.readFileStream('project.jcf');
    } else {
      // Fallback
      const data = await this.adapter.readFile('project.jcf');
      return new ReadableStream({
        start(controller) {
          controller.enqueue(data);
          controller.close();
        }
      });
    }
  }
}
```

## 6. Platform Detection

```typescript
/**
 * Auto-detect platform and return appropriate adapter
 */
export function createAdapter(options?: AdapterOptions): FileSystemAdapter {
  // Browser
  if (typeof window !== 'undefined' && 'indexedDB' in window) {
    return new BrowserAdapter(options?.browser);
  }
  
  // Node.js
  if (typeof process !== 'undefined' && process.versions?.node) {
    return new NodeAdapter(options?.node?.basePath);
  }
  
  // Tauri
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return new TauriAdapter(options?.tauri);
  }
  
  // Deno
  if (typeof Deno !== 'undefined') {
    return new DenoAdapter(options?.deno);
  }
  
  throw new Error('Unsupported platform');
}

// Usage
const manager = new JCFManager();
const adapter = createAdapter();
await manager.init(adapter);
```

## 7. Testing z Adapterami

### 7.1 Unit Tests

```typescript
describe('JCFManager', () => {
  let manager: JCFManager;
  let adapter: MemoryAdapter;
  
  beforeEach(async () => {
    adapter = new MemoryAdapter();
    manager = new JCFManager();
    await manager.init(adapter);
  });
  
  it('should save checkpoint', async () => {
    await manager.addFile('test.txt', 'Hello World');
    const versionId = await manager.saveCheckpoint('Test commit');
    
    expect(versionId).toBeTruthy();
    expect(adapter.getStorageSize()).toBeGreaterThan(0);
  });
  
  it('should restore version', async () => {
    await manager.addFile('test.txt', 'v1');
    const v1 = await manager.saveCheckpoint('v1');
    
    await manager.addFile('test.txt', 'v2');
    await manager.saveCheckpoint('v2');
    
    await manager.restoreVersion(v1);
    
    const content = await manager.getFileContent('test.txt');
    expect(content).toBe('v1');
  });
});
```

### 7.2 Integration Tests

```typescript
describe('Platform Integration', () => {
  const platforms = [
    { name: 'Memory', adapter: () => new MemoryAdapter() },
    { name: 'Node', adapter: () => new NodeAdapter('./test-data') },
    // Browser tests run in karma/playwright
  ];
  
  platforms.forEach(({ name, adapter }) => {
    describe(`on ${name}`, () => {
      it('should handle large files', async () => {
        const mgr = new JCFManager();
        await mgr.init(adapter());
        
        const largeData = new Uint8Array(10 * 1024 * 1024); // 10MB
        await mgr.addFile('large.bin', largeData);
        
        await mgr.saveCheckpoint('Add large file');
        
        // Verify
        const retrieved = await mgr.getFileContent('large.bin');
        expect(retrieved.byteLength).toBe(largeData.byteLength);
      });
    });
  });
});
```

## 8. Best Practices

### 8.1 Do's ✅

1. **Zawsze używaj abstrackcji** (FileSystemAdapter), nigdy bezpośrednio platform API
2. **Test z MemoryAdapter** najpierw (szybkie, deterministyczne)
3. **Sprawdzaj `supportsStreaming`** przed użyciem streaming API
4. **Implement graceful degradation** (fallback z streaming na buffered)
5. **Call `init()` i `dispose()`** properly (resource management)

### 8.2 Don'ts ❌

1. **Nie leak platform-specific code** do core logic
2. **Nie assume filesystem semantics** (np. case-sensitivity)
3. **Nie ignore errors** z adapter methods
4. **Nie mix adapters** (jeden manager = jeden adapter)
5. **Nie block UI thread** (używaj streaming gdzie możliwe)

## 9. Rozszerzanie: Custom Adapter

```typescript
// Example: S3 Adapter dla cloud storage
class S3Adapter implements FileSystemAdapter {
  readonly name = 'S3Adapter';
  readonly supportsStreaming = true;
  
  constructor(
    private s3Client: S3Client,
    private bucket: string
  ) {}
  
  async readFile(path: string): Promise<Uint8Array> {
    const command = new GetObjectCommand({
      Bucket: this.bucket,
      Key: path
    });
    
    const response = await this.s3Client.send(command);
    const chunks: Uint8Array[] = [];
    
    for await (const chunk of response.Body as any) {
      chunks.push(chunk);
    }
    
    return concatUint8Arrays(chunks);
  }
  
  async writeFile(path: string, data: Uint8Array): Promise<void> {
    const command = new PutObjectCommand({
      Bucket: this.bucket,
      Key: path,
      Body: data
    });
    
    await this.s3Client.send(command);
  }
  
  // ... rest of implementation
}
```

## 10. Performance Considerations

### 10.1 Adapter Benchmarks

| Operation | Memory | Node.js | Browser (IndexedDB) | Tauri |
|-----------|--------|---------|---------------------|-------|
| Read 1MB | <1ms | 5ms | 15ms | 8ms |
| Write 1MB | <1ms | 8ms | 25ms | 12ms |
| Read 100MB | 10ms | 150ms | 800ms | 200ms |
| Write 100MB | 15ms | 300ms | 1.5s | 350ms |

### 10.2 Optymalizacje

```typescript
// Cache adapter capabilities
class CachedAdapter implements FileSystemAdapter {
  private cache = new Map<string, Uint8Array>();
  
  constructor(private inner: FileSystemAdapter) {}
  
  async readFile(path: string): Promise<Uint8Array> {
    if (this.cache.has(path)) {
      return this.cache.get(path)!;
    }
    
    const data = await this.inner.readFile(path);
    this.cache.set(path, data);
    return data;
  }
  
  // ... delegate other methods
}
```

## 11. Następne Kroki

1. Przeczytaj [Workers](./06-workers.md) dla multi-threading
2. Zobacz [API Reference](../api/JCFManager.md)
3. Sprawdź [Examples - Tauri Integration](../examples/04-tauri.md)

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

