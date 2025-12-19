# Kluczowe koncepcje techniczne

## Time-Travel Versioning

Możliwość cofnięcia projektu do dowolnego punktu w historii.

```typescript
// Tworzenie checkpoint
const v1 = await manager.saveCheckpoint('Initial commit');
// ... wprowadzanie zmian ...
const v2 = await manager.saveCheckpoint('Add login feature');

// Cofnięcie w czasie
await manager.restoreVersion(v1); // Projekt w stanie v1
```

## Reverse Delta Strategy

HEAD zawsze pełny - starsze wersje jako patche.

```
v1 (old) ← patch ← v2 ← patch ← v3 (HEAD, full)
```

Dlaczego? 95% czasu pracuje się z HEAD → optymalizacja dla common case.

## Content Addressable Storage (CAS)

Pliki binarne identyfikowane przez hash (SHA-256). Ten sam content = ten sam hash = jedna kopia.

```
logo.png (hash: abc123) → .store/blobs/abc123
icon.png (różny, hash: def456) → .store/blobs/def456
logo-copy.png (ten sam, hash: abc123) → reuse abc123
```

Zaleta: Automatyczna deduplikacja.

## Format kontenerowy ZIP

Format JCF to standardowy ZIP. Można rozpakować zwykłym archiverem!

```bash
unzip project.jcf
# folder content/ = najnowsza wersja
```

## Streaming Architecture

Duże pliki (>50MB) przetwarzane jako stream → nie ładuje się całości do RAM.

## Platform Abstraction

Adapter Pattern - ten sam kod działa wszędzie:
- Browser (IndexedDB)
- Node.js (fs)
- Tauri (tauri.fs)