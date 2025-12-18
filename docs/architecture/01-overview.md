# 🏗️ Przegląd Architektury JCF Manager

## 1. Wprowadzenie

**JCF Manager** to biblioteka JavaScript/TypeScript zaprojektowana do zarządzania plikami projektowymi w formacie kontenerowym z wbudowaną historią zmian (Time-Travel Versioning). System umożliwia cofanie się do dowolnego punktu w historii projektu, zachowując przy tym efektywność przechowywania i szybkość dostępu do najnowszej wersji.

## 2. Kluczowe Cechy

### 2.1 Format Plików
- **Kontener**: Standardowy ZIP archive
- **Kompatybilność**: Można otworzyć zwykłym unzipperem
- **Mimetype**: `application/x-jcf`
- **Struktura**: Samoopisująca się (manifest.json)

### 2.2 Time-Travel Versioning
- **Reverse Delta Strategy**: Najnowsza wersja zawsze pełna
- **Efficient History**: Starsze wersje jako kompresowane delty
- **Binary Deduplication**: Content Addressable Storage (CAS)
- **Metadata Rich**: Pełna historia zmian z timestampami i autorami

### 2.3 Performance
- **Streaming Support**: Obsługa plików >500MB bez ładowania do RAM
- **Multi-threading**: Web Workers dla CPU-intensive operacji
- **Lazy Loading**: Historie ładowane tylko na żądanie
- **Smart Compression**: fflate z automatyczną optymalizacją

### 2.4 Izomorficzność
- **Browser**: IndexedDB + File API
- **Node.js**: fs/promises
- **Tauri**: tauri.fs API
- **Deno/Bun**: Gotowe do wsparcia

## 3. Architektura Wysokopoziomowa

```
┌─────────────────────────────────────────────────────────┐
│                    USER APPLICATION                     │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│                   JCFManager API                        │
│  - saveCheckpoint()  - restoreVersion()                 │
│  - addFile()         - getFileStream()                  │
│  - runGC()           - getHistory()                     │
└──────┬──────────────┬──────────────┬───────────────────┘
       │              │              │
┌──────▼──────┐  ┌───▼────────┐  ┌──▼─────────────────┐
│ Version     │  │ File       │  │ Delta/Blob         │
│ Manager     │  │ Manager    │  │ Managers           │
└──────┬──────┘  └───┬────────┘  └──┬─────────────────┘
       │              │              │
┌──────▼──────────────▼──────────────▼─────────────────┐
│              FileSystem Adapter                       │
│  (Browser / Node.js / Tauri / Custom)                │
└──────┬──────────────┬──────────────┬─────────────────┘
       │              │              │
┌──────▼──────┐  ┌───▼────────┐  ┌──▼─────────────────┐
│ fflate      │  │ diff-match │  │ Web Workers        │
│ (ZIP)       │  │ -patch     │  │ (Hash/Diff)        │
└─────────────┘  └────────────┘  └────────────────────┘
```

## 4. Warstwy Systemu

### 4.1 API Layer (Warstwa Publiczna)
**Odpowiedzialność**: Interfejs użytkownika, walidacja wejścia

**Klasy**:
- `JCFManager` - główny punkt wejścia
- `JCFConfig` - konfiguracja systemu

**Zasady**:
- Wszystkie metody async (Promise-based)
- TypeScript strict mode
- Error handling z custom exceptions
- Event emitters dla progress tracking

### 4.2 Core Layer (Warstwa Logiki)
**Odpowiedzialność**: Implementacja algorytmów biznesowych

**Moduły**:
- `VersionManager` - zarządzanie historią commitów
- `FileManager` - CRUD operacje na plikach
- `DeltaManager` - obliczanie i aplikowanie diff
- `BlobManager` - CAS dla plików binarnych

**Zasady**:
- Separacja concerns
- Dependency injection
- Unit testable
- Immutable data structures gdzie możliwe

### 4.3 Storage Layer (Warstwa Przechowywania)
**Odpowiedzialność**: Abstrakcja dostępu do systemu plików

**Pattern**: Adapter Pattern

**Implementacje**:
- `BrowserAdapter` - IndexedDB + File API
- `NodeAdapter` - fs/promises
- `TauriAdapter` - tauri.fs
- `MemoryAdapter` - testy jednostkowe

**Zasady**:
- Interface-first design
- Streaming API where possible
- Error handling per platform
- Transaction support

### 4.4 Worker Layer (Warstwa Obliczeniowa)
**Odpowiedzialność**: Offloading CPU-intensive tasks

**Workers**:
- `HashWorker` - SHA-256 hashing
- `DiffWorker` - Text diff computation
- `CompressWorker` - ZIP compression/decompression

**Zasady**:
- Message passing (structured clone)
- Graceful degradation (fallback to main thread)
- Worker pool management
- Cancellable operations

## 5. Data Flow

### 5.1 Save Checkpoint Flow

```
User calls saveCheckpoint()
         │
         ▼
┌────────────────────┐
│ Identify Changed   │
│ Files (dirty check)│
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ For Text Files:    │
│ - Compute Reverse  │
│   Delta (NEW→OLD)  │
│ - Store in .store/ │
│   deltas/          │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ For Binary Files:  │
│ - Hash content     │
│ - Store in .store/ │
│   blobs/ (if new)  │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Update manifest.   │
│ json with new      │
│ version metadata   │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Rebuild /content/  │
│ with current state │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Write ZIP to disk  │
│ (streaming)        │
└────────────────────┘
```

### 5.2 Restore Version Flow

```
User calls restoreVersion(id)
         │
         ▼
┌────────────────────┐
│ Build version path │
│ (HEAD → target)    │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ For each file:     │
│ - Load current     │
│ - Apply patches    │
│   backwards        │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Update /content/   │
│ with restored      │
│ files              │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Update HEAD ref    │
│ in manifest        │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Save manifest.json │
└────────────────────┘
```

## 6. Kluczowe Decyzje Projektowe

### 6.1 Dlaczego Reverse Delta?

**Alternatywy rozważane**:
1. **Forward Delta** (Git-style): Stary→Nowy
   - ❌ Wymaga przechodzenia całej historii dla HEAD
   - ✅ Efficient dla old versions
   
2. **Full Snapshots**: Każda wersja pełna
   - ❌ Ogromny rozmiar pliku
   - ✅ Instant access do każdej wersji
   
3. **Reverse Delta** (wybrany): Nowy→Stary
   - ✅ HEAD zawsze instant
   - ✅ Reasonable file size
   - ⚠️ Old versions require reconstruction

**Uzasadnienie**: 
W 95% przypadków użytkownik pracuje z najnowszą wersją (HEAD). Historia jest używana rzadko i głównie do przeglądania, nie codziennej pracy.

### 6.2 Dlaczego fflate zamiast JSZip?

| Kryterium | JSZip | fflate | Waga |
|-----------|-------|--------|------|
| Performance | 3/10 | 9/10 | 40% |
| Bundle Size | 5/10 | 10/10 | 20% |
| API Ease | 10/10 | 6/10 | 15% |
| Streaming | 4/10 | 10/10 | 25% |

**Wynik**: fflate wygrywa (8.4 vs 5.8)

### 6.3 Dlaczego Adapter Pattern?

**Problem**: Różne środowiska mają różne API dla I/O

**Rozwiązanie**: Abstrakcja przez interfejs + implementacje per platform

**Korzyści**:
- Testability (MockAdapter)
- Future-proof (nowe platformy bez refactoringu core)
- Clean separation of concerns

## 7. Performance Targets

### 7.1 Benchmarks Celowe

| Operacja | Target | Warunek |
|----------|--------|---------|
| Load Project | <2s | 500 plików, 50MB |
| Save Checkpoint | <1s | 10 zmienionych plików |
| Restore Version | <3s | 50 commitów wstecz |
| Add File (large) | <5s | 100MB binary |
| GC | <10s | 1000 orphaned blobs |

### 7.2 Memory Constraints

- **Browser**: Max 500MB per operation
- **Node.js**: Unlimited (ale streaming preferred)
- **Mobile**: Max 100MB per operation

## 8. Security Considerations

### 8.1 Threats

1. **ZIP Bombs**: Malicious highly-compressed files
   - Mitigacja: Decompression size limits
   
2. **Path Traversal**: `../../etc/passwd` w nazwach plików
   - Mitigacja: Path sanitization
   
3. **Manifest Tampering**: Ręczna edycja manifest.json
   - Mitigacja: Checksums + validation

### 8.2 Best Practices

- Input validation na wszystkich entry points
- JSON Schema dla manifestu
- CRC checks dla ZIP entries
- Atomic writes (temp file → rename)

## 9. Extensibility Points

### 9.1 Plugin System (Future)

```typescript
interface JCFPlugin {
  name: string;
  version: string;
  onBeforeSave?: (context: SaveContext) => Promise<void>;
  onAfterSave?: (context: SaveContext) => Promise<void>;
  onBeforeRestore?: (context: RestoreContext) => Promise<void>;
}

// Example: Auto-formatter plugin
class FormatterPlugin implements JCFPlugin {
  async onBeforeSave(context: SaveContext) {
    for (const file of context.changedFiles) {
      if (file.path.endsWith('.js')) {
        file.content = await prettier.format(file.content);
      }
    }
  }
}
```

### 9.2 Custom Compression Algorithms

```typescript
interface CompressionAdapter {
  compress(data: Uint8Array): Promise<Uint8Array>;
  decompress(data: Uint8Array): Promise<Uint8Array>;
}

// Example: Brotli for text files
class BrotliAdapter implements CompressionAdapter {
  // ...
}
```

## 10. Roadmap

### Phase 1: MVP (Obecny)
- ✅ Basic ZIP structure
- ✅ Manifest management
- ✅ Reverse delta for text
- ✅ Binary CAS

### Phase 2: Production Ready
- ⏳ Full adapter implementations
- ⏳ Worker pool
- ⏳ Streaming support
- ⏳ Error recovery

### Phase 3: Advanced Features
- 🔮 Branching support
- 🔮 Merge conflict resolution
- 🔮 Partial clone (sparse checkout)
- 🔮 Network sync (WebRTC/WebSocket)

### Phase 4: Ecosystem
- 🔮 Plugin system
- 🔮 CLI tools
- 🔮 GUI explorer
- 🔮 VS Code extension

## 11. Porównanie z Alternatywami

### 11.1 Git
- ✅ Git: Mature, proven, powerful
- ❌ Git: Complex, large footprint, not browser-friendly
- ✅ JCF: Simple, lightweight, isomorphic
- ❌ JCF: New, less features

**Use case**: JCF dla single-file projects w browser/electron, Git dla code repositories

### 11.2 Automerge/Yjs
- ✅ Automerge: CRDT, automatic merge
- ❌ Automerge: Memory overhead, specific data structures
- ✅ JCF: Standard files, low overhead
- ❌ JCF: No automatic merge

**Use case**: Automerge dla collaborative editing, JCF dla versioned storage

## 12. Następne Kroki

1. Przeczytaj [Format JCF](./02-jcf-format.md) dla szczegółów struktury
2. Zrozum [Reverse Delta Strategy](./03-reverse-delta.md)
3. Zobacz [API Reference](../api/JCFManager.md) dla implementacji
4. Sprawdź [Examples](../examples/01-quickstart.md) dla praktyki

---

**Autorzy**: Zespół JCF Manager  
**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

