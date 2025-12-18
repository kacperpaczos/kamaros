# Problem Statement

> **Jaki problem rozwiązuje Kamaros?**

[← Back: Overview](01-overview.md) | [Next: Key Concepts →](03-key-concepts.md)

---

## Problem: Zarządzanie Wersjami Plików w Aplikacjach Web

### Scenariusz
Tworzysz edytor kodu w przeglądarce (jak CodeSandbox/StackBlitz). Użytkownik pracuje nad projektem i potrzebuje:
1. **Undo/Redo** - cofanie zmian
2. **History** - przeglądanie historii edycji
3. **Restore** - przywrócenie do dowolnego punktu
4. **Offline** - działanie bez serwera
5. **Performance** - szybki dostęp do aktualnego stanu

### Dostępne Rozwiązania

#### Opcja A: Git (isomorphic-git)
**Pros:**
- Sprawdzony system wersjonowania
- Potężne narzędzia

**Cons:**
- ❌ Słaba obsługa plików binarnych (duże obrazy, wideo)
- ❌ Skomplikowany (staging, branches, merge conflicts)
- ❌ Pełna historia w pamięci (problem dla dużych projektów)
- ❌ Nie działa dobrze z >100MB projektami w przeglądarce

#### Opcja B: Własny Format (Custom Binary)
**Pros:**
- Pełna kontrola

**Cons:**
- ❌ Brak standardów
- ❌ Zero toolingu (debugging, recovery)
- ❌ Długi czas rozwoju
- ❌ Trudne utrzymanie

#### Opcja C: Baza Danych (IndexedDB/SQLite)
**Pros:**
- Structured query

**Cons:**
- ❌ Brak kompresji
- ❌ Trudne backup/export
- ❌ Nie działa jako standalone file
- ❌ Platform-specific

### Gap w Rynku

**Potrzebujemy:**
- ✅ Prostego formatu (łatwiejszy niż Git)
- ✅ Dobrze radzi sobie z binariami
- ✅ Działa offline
- ✅ Fast HEAD access
- ✅ Standard recovery tools (ZIP)
- ✅ Izomorficzny (Browser/Node/Tauri)

**→ To jest Kamaros!**

---

## Problem 2: Rozmiar Pliku

### Scenariusz
Projekt zawiera:
- 1000 plików tekstowych (~10MB)
- 50 obrazów (~50MB)
- Historia 100 commitów

**Naiwne podejście** (snapshot każdej wersji):
```
100 versions × (10MB text + 50MB images) = 6GB
```

**Git approach** (forward deltas + pack):
```
~500MB (lepiej, ale nadal duże)
```

**Kamaros approach** (reverse delta + CAS):
```
10MB (current text) +
50MB (current images) +
5MB (patches for 100 versions) +
10MB (deduplicated old images) =
~75MB (8x mniejsze niż Git!)
```

### Techniki Kompresji

1. **Reverse Delta**: HEAD zawsze pełny (szybki), historia jako patche
2. **Content Addressable Storage**: Ten sam obraz użyty 10x → przechowany 1x
3. **Smart Compression**: Tylko pliki, które się kompresują

---

## Problem 3: Performance w Przeglądarce

### Scenariusz
Użytkownik dodaje 500MB wideo do projektu w edytorze webowym.

**Naiwne podejście**:
```javascript
const file = await fileInput.files[0];
const buffer = await file.arrayBuffer(); // ❌ 500MB w RAM
const hash = sha256(buffer);              // ❌ UI freeze
await saveToIndexedDB(buffer);            // ❌ Quota exceeded
```

**Kamaros approach**:
```javascript
const file = await fileInput.files[0];
const stream = file.stream();             // ✅ Streaming
const hash = await hashStream(stream, {   // ✅ Web Worker (non-blocking)
  worker: true
});
await manager.addFile('video.mp4', stream); // ✅ Chunked write
```

**Result**:
- ✅ RAM usage: ~50MB (buffer chunks)
- ✅ UI responsive (Web Workers)
- ✅ Incremental progress updates

---

## Rozwiązanie: Kamaros JCF

### Jak Kamaros Rozwiązuje Te Problemy?

| Problem | Kamaros Solution |
|---------|------------------|
| **Complexity** | Simple API (addFile, save Checkpoint, restoreVersion) |
| **Binary files** | Content Addressable Storage + deduplication |
| **File size** | Reverse delta + smart compression |
| **Performance** | Streaming + Web Workers + LRU cache |
| **Recovery** | Standard ZIP tools |
| **Platform** | Adapter pattern (Browser/Node/Tauri) |

---

[← Back: Overview](01-overview.md) | [Next: Key Concepts →](03-key-concepts.md)

