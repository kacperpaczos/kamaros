# Platform Abstraction - Adapter Pattern

Adapter Pattern w JCF Manager pozwala na unifikację dostępu do różnych środowisk poprzez wspólny interfejs.

## Problem i rozwiązanie

### Problem: Różne API dla I/O

**Browser**: IndexedDB + File API
**Node.js**: fs/promises
**Tauri**: @tauri-apps/api/fs

Core logic nie może znać wszystkich API = niemożliwe do utrzymania

### Rozwiązanie: Adapter Pattern

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
}
```

## Interfejs FileSystemAdapter

```typescript
interface FileSystemAdapter {
  readonly name: string;
  readonly supportsStreaming: boolean;

  init(): Promise<void>;
  dispose(): Promise<void>;

  // File operations
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  fileExists(path: string): Promise<boolean>;
  deleteFile(path: string): Promise<void>;

  // ZIP operations
  createZipWriter(): ZipWriter;
  createZipReader(source: Uint8Array | ReadableStream): ZipReader;
}
```

## Implementacje adapterów

### BrowserAdapter (IndexedDB)

Użycie: Progressive Web Apps, Electron
Storage: IndexedDB (persistent) + Memory (cache)

### NodeAdapter (fs/promises)

Użycie: CLI tools, Node.js servers
Storage: Native filesystem

### TauriAdapter

Użycie: Tauri desktop applications
Storage: Native filesystem via Tauri API

### MemoryAdapter (Testing)

Użycie: Unit tests, in-memory prototyping

## Platform detection

```typescript
export function createAdapter(): FileSystemAdapter {
  // Browser
  if (typeof window !== 'undefined' && 'indexedDB' in window) {
    return new BrowserAdapter();
  }

  // Node.js
  if (typeof process !== 'undefined' && process.versions?.node) {
    return new NodeAdapter();
  }

  // Tauri
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return new TauriAdapter();
  }

  throw new Error('Unsupported platform');
}
```

## Best practices

### Do's
1. Zawsze używaj abstrakcji (FileSystemAdapter), nigdy bezpośrednio platform API
2. Test z MemoryAdapter najpierw (szybkie, deterministyczne)
3. Sprawdzaj supportsStreaming przed użyciem streaming API
4. Implement graceful degradation (fallback z streaming na buffered)
5. Call init() i dispose() properly (resource management)

### Don'ts
1. Nie leak platform-specific code do core logic
2. Nie assume filesystem semantics (np. case-sensitivity)
3. Nie ignore errors z adapter methods
4. Nie mix adapters (jeden manager = jeden adapter)
5. Nie block UI thread (używaj streaming gdzie możliwe)