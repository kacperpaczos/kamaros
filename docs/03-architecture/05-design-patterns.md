# Wzorce projektowe w Kamaros

Kamaros wykorzystuje 7 klasycznych wzorców projektowych do zapewnienia modularności, testowalności i rozszerzalności.

## Przegląd wzorców

- **Adapter Pattern** - Abstrakcja platform
- **Strategy Pattern** - Polityka kompresji
- **Factory Pattern** - Tworzenie obiektów
- **Repository Pattern** - Dostęp do danych
- **Observer Pattern** - System zdarzeń
- **Chain of Responsibility** - Obsługa błędów
- **Command Pattern** - Undo/Redo (przyszłość)

## Adapter Pattern - Abstrakcja platform

### Problem
Różne platformy mają różne API dla operacji I/O.

### Rozwiązanie

```typescript
interface FileSystemAdapter {
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  // ...
}

class BrowserAdapter implements FileSystemAdapter { /*...*/ }
class NodeAdapter implements FileSystemAdapter { /*...*/ }
class TauriAdapter implements FileSystemAdapter { /*...*/ }
```

### Korzyści
- ✅ Core logic platform-agnostic
- ✅ Easy testing (MockAdapter)
- ✅ Future-proof (nowe platformy bez refactoringu core)
- ✅ Separation of concerns

## Strategy Pattern - Polityka kompresji

### Problem
Różne typy plików wymagają różnych strategii kompresji.

### Rozwiązanie

```typescript
interface CompressionStrategy {
  shouldCompress(filepath: string, data: Uint8Array): boolean;
  getLevel(): number;
}

class TextCompressionStrategy implements CompressionStrategy {
  shouldCompress(filepath: string, data: Uint8Array): boolean {
    return this.isTextFile(filepath);
  }

  getLevel(): number {
    return 6;
  }
}
```

## Factory Pattern - Tworzenie obiektów

### Problem
Tworzenie ZIP readers/writers z różnymi konfiguracjami jest skomplikowane.

### Rozwiązanie

```typescript
class ZipFactory {
  static createWriter(
    adapter: FileSystemAdapter,
    strategy?: CompressionStrategy
  ): ZipWriter {
    const compressionStrategy = strategy ?? new AdaptiveCompressionStrategy();
    return new FflateZipWriter(adapter, compressionStrategy);
  }
}
```

## Repository Pattern - Dostęp do danych

### Problem
Bezpośredni dostęp do adaptera w całym kodzie prowadzi do duplikacji logiki.

### Rozwiązanie

```typescript
class BlobRepository {
  constructor(
    private adapter: FileSystemAdapter,
    private cache: LRUCache<string, Uint8Array>
  ) {}

  async save(hash: string, data: Uint8Array): Promise<void> {
    await this.adapter.writeFile(`.store/blobs/${hash}`, data);
    this.cache.set(hash, data);
  }

  async load(hash: string): Promise<Uint8Array> {
    const cached = this.cache.get(hash);
    if (cached) return cached;

    const data = await this.adapter.readFromZip(`.store/blobs/${hash}`);
    this.cache.set(hash, data);
    return data;
  }
}
```

## Observer Pattern - System zdarzeń

### Problem
Jak informować użytkownika o progress długotrwałych operacji?

### Rozwiązanie

```typescript
class JCFManager extends EventEmitter {
  async saveCheckpoint(message: string): Promise<string> {
    this.emit('checkpoint:start', { message, timestamp: Date.now() });

    // ... operation logic ...

    this.emit('checkpoint:complete', {
      versionId,
      duration: Date.now() - startTime
    });

    return versionId;
  }
}
```

## Chain of Responsibility - Obsługa błędów

### Problem
Różne typy błędów wymagają różnych strategii obsługi.

### Rozwiązanie

```typescript
abstract class ErrorHandler {
  protected next?: ErrorHandler;

  setNext(handler: ErrorHandler): ErrorHandler {
    this.next = handler;
    return handler;
  }

  handle(error: Error, context: ErrorContext): void {
    if (this.canHandle(error)) {
      this.process(error, context);
    } else if (this.next) {
      this.next.handle(error, context);
    } else {
      throw error;
    }
  }
}
```

## Podsumowanie wzorców

| Pattern | Purpose | Status | Lokalizacja |
|---------|---------|--------|-------------|
| Adapter | Platform abstraction | ✅ v1.0 | src/adapters/ |
| Strategy | Compression policy | ✅ v1.0 | src/core/compression/ |
| Factory | Object creation | ✅ v1.0 | src/core/zip/ |
| Repository | Data access layer | ✅ v1.0 | src/repositories/ |
| Observer | Event system | ✅ v1.0 | src/core/JCFManager.ts |
| Chain of Responsibility | Error handling | ✅ v1.0 | src/core/errors/ |
| Command | Undo/Redo | ⚠️ v2.0 | src/commands/ (future) |

## Dlaczego te wzorce?

### Testability
Każdy wzorzec ułatwia unit testing poprzez dependency injection i mocki.

### Maintainability
Separation of concerns - każda klasa ma jasną odpowiedzialność.

### Extensibility
Łatwo dodawać nowe implementacje bez modyfikacji core logic.

### Industry Standard
Sprawdzone wzorce używane w produkcyjnych systemach.