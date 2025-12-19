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

## Format pliku: Inteligentny ZIP (JCF)

**JCF = JSON Content Format** to inteligentny format pliku oparty na standardowym ZIP archive z dodatkowymi funkcjami wersjonowania.

### Jak działa biblioteka JCF Manager:

1. **Tworzy lub wczytuje specyfikację JCF** (`manifest.json`)
   - Definiuje strukturę projektu
   - Zawiera historię wersji
   - Opisuje metadane plików

2. **Wczytuje pliki zgodnie ze specyfikacją**
   - Odczytuje `content/` dla aktualnego stanu (HEAD)
   - Rekonstruuje starsze wersje z `.store/deltas/` i `.store/blobs/`
   - Waliduje zgodność z manifest.json

3. **Zapisuje pliki zgodnie ze specyfikacją**
   - Zapisuje aktualny stan do `content/`
   - Tworzy reverse delta patches dla plików tekstowych
   - Przechowuje pliki binarne w CAS (Content Addressable Storage)
   - Aktualizuje manifest.json

4. **Waliduje zgodność ze specyfikacją**
   - Sprawdza integralność manifest.json
   - Weryfikuje hashe SHA-256 dla plików binarnych
   - Waliduje strukturę ZIP
   - Sprawdza spójność historii wersji

5. **Wersjonuje zgodnie ze specyfikacją**
   - Tworzy checkpointy z metadanymi (timestamp, autor, message)
   - Zarządza referencjami do blobów i delt
   - Utrzymuje łańcuch wersji (parentId)

### Struktura JCF (rozpakowane):
```
project.jcf (ZIP) zawiera:
├── content/           # Aktualny stan plików (HEAD)
│   └── [dowolne pliki zgodnie ze specyfikacją]
├── .store/           # Wersjonowanie (ukryte)
│   ├── manifest.json # Specyfikacja projektu i historii
│   ├── blobs/       # Deduplikowane pliki binarne (CAS)
│   └── deltas/      # Reverse delta patches dla tekstu
└── [metadata]       # Informacje o kompresji, etc.
```

**Ważne**: Biblioteka JCF nie interpretuje zawartości plików - przechowuje dowolne pliki zgodnie ze specyfikacją formatu. To aplikacja decyduje, jakie pliki przechowuje (np. `.js`, `.glsl`, `.json`, `.png` - JCF nie rozróżnia typów).

### Standardowość:
- **Rozszerzenie**: `.jcf`
- **MIME type**: `application/x-jcf`
- **Kompatybilność**: Można rozpakować zwykłym ZIP archiverem
- **Recovery**: Standardowe narzędzia ZIP do naprawy uszkodzonych plików

```bash
# Można otworzyć zwykłym unzip
unzip project.jcf
ls content/  # Zobaczysz aktualną wersję projektu
```

## Streaming Architecture

Duże pliki (>50MB) przetwarzane jako stream → nie ładuje się całości do RAM.

## Platform Abstraction

Adapter Pattern - ten sam kod działa wszędzie:
- Browser (IndexedDB)
- Node.js (fs)
- Tauri (tauri.fs)