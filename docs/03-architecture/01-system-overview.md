# Przegląd architektury systemu

**JCF (JSON Content Format)** to inteligentny format pliku oparty na standardowym ZIP archive z wbudowanym systemem wersjonowania Time-Travel. **JCF Manager** to biblioteka wielojęzyczna (TypeScript, Python) z core w Rust do efektywnego zarządzania tym formatem.

## Kluczowe cechy

### Format plików
- Kontener: Standardowy ZIP archive
- Kompatybilność: Można otworzyć zwykłym unzipperem
- Mimetype: application/x-jcf
- Struktura: Samoopisująca się (manifest.json)

### Time-Travel Versioning
- Reverse Delta Strategy: Najnowsza wersja zawsze pełna
- Efficient History: Starsze wersje jako kompresowane delty
- Binary Deduplication: Content Addressable Storage (CAS)
- Metadata Rich: Pełna historia zmian z timestampami i autorami

### Performance
- Streaming Support: Obsługa plików >500MB bez ładowania do RAM
- Multi-threading: Web Workers dla CPU-intensive operacji
- Lazy Loading: Historie ładowane tylko na żądanie
- Smart Compression: fflate z automatyczną optymalizacją

### Izomorficzność
- Browser: IndexedDB + File API
- Node.js: fs/promises
- Tauri: tauri.fs API
- Deno/Bun: Gotowe do wsparcia

## Architektura wysokiego poziomu

```
┌─────────────────────────────────────────────────────────┐
│                    APLIKACJA UŻYTKOWNIKA                │
│  (Browser/Node/Tauri/Deno)                              │
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

## Warstwy systemu

### API Layer - Warstwa publiczna
Odpowiedzialność: Interfejs użytkownika, walidacja wejścia

Klasy:
- JCFManager - główny punkt wejścia
- JCFConfig - konfiguracja systemu

### Core Layer - Warstwa logiki
Odpowiedzialność: Implementacja algorytmów biznesowych

Moduły:
- VersionManager - zarządzanie historią commitów
- FileManager - CRUD operacje na plikach
- DeltaManager - obliczanie i aplikowanie diff
- BlobManager - CAS dla plików binarnych

### Storage Layer - Warstwa przechowywania
Odpowiedzialność: Abstrakcja dostępu do systemu plików

Pattern: Adapter Pattern

Implementacje:
- BrowserAdapter - IndexedDB + File API
- NodeAdapter - fs/promises
- TauriAdapter - tauri.fs
- MemoryAdapter - testy jednostkowe

### Worker Layer - Warstwa obliczeniowa
Odpowiedzialność: Offloading CPU-intensive tasks

Workers:
- HashWorker - SHA-256 hashing
- DiffWorker - Text diff computation
- CompressWorker - ZIP compression/decompression

## Data flow

### Save Checkpoint Flow

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

### Restore Version Flow

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

## Kluczowe decyzje projektowe

### Dlaczego Reverse Delta?

Alternatywy rozważane:
1. Forward Delta (Git-style): Stary→Nowy
   - Problem: Wymaga przechodzenia całej historii dla HEAD

2. Full Snapshots: Każda wersja pełna
   - Problem: Ogromny rozmiar pliku

3. Reverse Delta (wybrany): Nowy→Stary
   - Zaletą: HEAD zawsze instant
   - Reasonable file size
   - Old versions require reconstruction

Uzasadnienie: 95% czasu pracuje się z najnowszą wersją. Historia jest używana rzadko.

### Dlaczego fflate zamiast JSZip?

| Kryterium | JSZip | fflate | Waga |
|-----------|-------|--------|------|
| Performance | 3/10 | 9/10 | 40% |
| Bundle Size | 5/10 | 10/10 | 20% |
| API Ease | 10/10 | 6/10 | 15% |
| Streaming | 4/10 | 10/10 | 25% |

Wynik: fflate wygrywa (8.4 vs 5.8)

### Dlaczego Adapter Pattern?

Problem: Różne środowiska mają różne API dla I/O

Rozwiązanie: Abstrakcja przez interfejs + implementacje per platform

Korzyści:
- Testability (MockAdapter)
- Future-proof (nowe platformy bez refactoringu core)
- Clean separation of concerns

## Performance targets

### Benchmarks celowe

| Operacja | Target | Warunek |
|----------|--------|---------|
| Load Project | <2s | 500 plików, 50MB |
| Save Checkpoint | <1s | 10 zmienionych plików |
| Restore Version | <3s | 50 commitów wstecz |
| Add File (large) | <5s | 100MB binary |
| GC | <10s | 1000 orphaned blobs |

### Memory constraints

- Browser: Max 500MB per operation
- Node.js: Unlimited (ale streaming preferred)
- Mobile: Max 100MB per operation

## Security considerations

### Threats

1. ZIP Bombs: Malicious highly-compressed files
   - Mitigacja: Decompression size limits

2. Path Traversal: ../../etc/passwd w nazwach plików
   - Mitigacja: Path sanitization

3. Manifest Tampering: Ręczna edycja manifest.json
   - Mitigacja: Checksums + validation

### Best practices

- Input validation na wszystkich entry points
- JSON Schema dla manifestu
- CRC checks dla ZIP entries
- Atomic writes (temp file → rename)

## Extensibility points

### Plugin system (Future)

```typescript
interface JCFPlugin {
  name: string;
  version: string;
  onBeforeSave?: (context: SaveContext) => Promise<void>;
  onAfterSave?: (context: SaveContext) => Promise<void>;
}

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

### Custom compression algorithms

```typescript
interface CompressionAdapter {
  compress(data: Uint8Array): Promise<Uint8Array>;
  decompress(data: Uint8Array): Promise<Uint8Array>;
}
```

## Roadmap

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

---

## Zobacz też

- [Diagramy Architektury](07-architecture-diagrams.md) - Wizualizacje Mermaid