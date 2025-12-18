# 📚 Dokumentacja JCF Manager

Witaj w dokumentacji biblioteki **JCF Manager** - systemu zarządzania plikami projektowymi z wbudowanym Time-Travel Versioning.

## 📖 Spis Treści

### 🏗️ Architektura
- [**01. Przegląd Architektury**](./architecture/01-overview.md) - Ogólny zarys systemu
- [**02. Format JCF**](./architecture/02-jcf-format.md) - Specyfikacja formatu plików
- [**03. Strategia Reverse Delta**](./architecture/03-reverse-delta.md) - System wersjonowania
- [**04. Content Addressable Storage**](./architecture/04-cas-blobs.md) - Zarządzanie plikami binarnymi
- [**05. Adapter Pattern**](./architecture/05-adapters.md) - Abstrakcja systemu plików
- [**06. Worker Pool Architecture**](./architecture/06-workers.md) - Architektura wielowątkowa

### 🔌 API Reference
- [**JCFManager**](./api/JCFManager.md) - Główna klasa interfejsu
- [**VersionManager**](./api/VersionManager.md) - Zarządzanie wersjami
- [**FileManager**](./api/FileManager.md) - Operacje na plikach
- [**DeltaManager**](./api/DeltaManager.md) - System diff/patch
- [**BlobManager**](./api/BlobManager.md) - Zarządzanie blobami
- [**TypeScript Interfaces**](./api/types.md) - Typy i interfejsy

### 💡 Przykłady
- [**Quick Start**](./examples/01-quickstart.md) - Pierwsze kroki
- [**Zaawansowane Operacje**](./examples/02-advanced.md) - Złożone scenariusze
- [**Streaming Dużych Plików**](./examples/03-streaming.md) - Obsługa plików >500MB
- [**Integracja z Tauri**](./examples/04-tauri.md) - Aplikacje desktopowe
- [**Browser Storage**](./examples/05-browser.md) - IndexedDB i File API

### 🎯 Szczegóły Implementacji
- [**Algorytm Save Checkpoint**](./architecture/algorithms/save-checkpoint.md)
- [**Algorytm Restore Version**](./architecture/algorithms/restore-version.md)
- [**Garbage Collection**](./architecture/algorithms/garbage-collection.md)
- [**File Rename Tracking**](./architecture/algorithms/rename-tracking.md)

### ⚡ Optymalizacje
- [**Performance Guide**](./architecture/performance.md)
- [**Memory Management**](./architecture/memory.md)
- [**Benchmarks**](./architecture/benchmarks.md)

### 🚨 Edge Cases i Problemy
- [**Risk Assessment**](./architecture/risk-assessment.md)
- [**Error Handling**](./architecture/error-handling.md)
- [**Common Pitfalls**](./architecture/pitfalls.md)

## 🚀 Szybki Start

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

// Inicjalizacja
const manager = new JCFManager();
await manager.init(new BrowserAdapter());

// Dodaj pliki
await manager.addFile('src/index.js', 'console.log("Hello");');
await manager.addFile('assets/logo.png', pngBlob);

// Zapisz checkpoint
const versionId = await manager.saveCheckpoint('Initial commit');

// Edytuj
await manager.addFile('src/index.js', 'console.log("Hello World");');

// Kolejny checkpoint
await manager.saveCheckpoint('Update message');

// Time travel
await manager.restoreVersion(versionId);
```

## 📦 Stack Technologiczny

- **Kompresja ZIP**: `fflate` (wydajność + streaming)
- **Diff/Patch**: `diff-match-patch` (Google)
- **Hashing**: WebCrypto API (SHA-256)
- **Workers**: Web Workers API
- **TypeScript**: Pełne typowanie

## 🏛️ Filozofia Designu

1. **Performance First**: Streaming, workers, lazy loading
2. **Isomorphic**: Browser + Node.js + Tauri
3. **Type Safety**: TypeScript z pełnym typowaniem
4. **Developer Experience**: Intuitive API, good errors
5. **Production Ready**: Error handling, validation, tests

## 📞 Wsparcie

- **GitHub Issues**: [github.com/yourrepo/jcf-manager/issues](https://github.com)
- **Discord**: [discord.gg/jcf](https://discord.gg)
- **Email**: support@jcf-manager.dev

## 📄 Licencja

MIT License - patrz [LICENSE](../LICENSE)

