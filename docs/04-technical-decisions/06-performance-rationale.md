# Uzasadnienie wydajności

## Kluczowe decyzje

Wszystkie decyzje techniczne priorytetyzują:

### 1. Fast HEAD access
**Reverse Delta Strategy**: HEAD zawsze pełny, starsze wersje jako patche
- **Zysk**: O(1) dostęp do bieżącego stanu
- **Koszt**: Rekonstrukcja starszych wersji

### 2. Efficient storage
**Content Addressable Storage**: Deduplikacja plików binarnych
- **Zysk**: Automatyczna deduplikacja (ten sam plik = jedna kopia)
- **Koszt**: Overhead haszowania

### 3. Good UX
**Web Workers**: Offload CPU-intensive operations
- **Zysk**: Responsywny UI podczas ciężkich operacji
- **Koszt**: Overhead message passing (~5-10%)

### 4. Standard recovery
**ZIP format**: Standardowe narzędzia mogą czytać pliki
- **Zysk**: Recovery bez specjalnych narzędzi
- **Koszt**: Overhead formatu ZIP

## Architektura High-Level

```
┌─────────────────────────────────────────────────────────┐
│                    USER APPLICATION                     │
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

## Data Flow Optimization

### Save Checkpoint Flow
1. **Identify changes** (O(n) scan)
2. **Process files** (parallel Web Workers)
3. **Update manifest** (O(1) write)
4. **Rebuild content/** (streaming copy)
5. **Write ZIP** (streaming compression)

### Restore Version Flow
1. **Build version path** (graph traversal)
2. **Apply patches backwards** (streaming)
3. **Update content/** (streaming copy)
4. **Update HEAD ref** (O(1) write)

## Performance Targets

### Benchmarks
- **Load project**: <2s for 500 files, 50MB
- **Save checkpoint**: <1s for 10 changed files
- **Restore version**: <3s for 50 commits back
- **Add large file**: <5s for 100MB binary
- **GC**: <10s for 1000 orphaned blobs

### Memory Limits
- **Browser**: <500MB peak per operation
- **Node.js**: Unlimited (streaming preferred)
- **Mobile**: <100MB peak per operation

## Trade-offs

### Speed vs Size
- **Reverse Delta**: Fast HEAD, larger history storage
- **CAS**: Storage efficient, hash computation overhead
- **ZIP**: Standard recovery, compression overhead

### Complexity vs Maintainability
- **Adapter Pattern**: Testable, platform-specific code isolated
- **Web Workers**: Complex message passing, responsive UX
- **Streaming**: Complex implementation, memory efficient

### Future Optimizations
- **Path Trie**: Fast file lookup (autocomplete)
- **Delta Compression**: Smaller history storage
- **Incremental GC**: Faster cleanup
- **Worker Pool**: Better resource utilization