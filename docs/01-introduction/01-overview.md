# Kamaros JCF Manager: Overview

> **Czym jest Kamaros i dlaczego został stworzony?**

[← Back to Index](../README.md) | [Next: Problem Statement →](02-problem-statement.md)

---

## Co to jest JCF Manager?

**JCF (JSON Container Format) Manager** to izomorficzna biblioteka JavaScript/TypeScript do zarządzania plikami projektowymi z wbudowanym systemem wersjonowania (Time-Travel Versioning).

## Kluczowe Cechy

### ✅ Reverse Delta Versioning
HEAD zawsze przechowywany w pełnej formie, historia jako patche. Gwarantuje natychmiastowy dostęp do aktualnego stanu projektu.

### ✅ Content Addressable Storage (CAS)
Automatyczna deduplikacja plików binarnych poprzez haszowanie SHA-256. Te same pliki przechowywane tylko raz.

### ✅ Izomorficzność
Działa w każdym środowisku JavaScript:
- **Browser**: IndexedDB + File API
- **Node.js**: fs/promises
- **Tauri**: @tauri-apps/api/fs
- **Deno**: Deno.* APIs (future)

### ✅ Streaming Support
Obsługa plików >500MB bez przepełnienia pamięci RAM dzięki strumieniowemu przetwarzaniu.

### ✅ Multi-threading
Web Workers dla operacji CPU-intensive (hashing, diffing, compression) - UI pozostaje responsywny.

### ✅ Production Ready
Kompletna obsługa błędów, walidacja danych, comprehensive testing suite.

---

## Use Cases

### 1. Code Editors
Edytory kodu w stylu VSCode z pełną historią zmian, offline-first.

```
Przykład: Visual Studio Code Online
- Local-first editing
- Complete version history
- Fast restore to any point
```

### 2. Design Tools
Aplikacje graficzne w stylu Figma z version control dla designów.

```
Przykład: Design Editor
- Layer versioning
- Asset deduplication (ikony, obrazy)
- Branch designs (future)
```

### 3. Game Editors
Edytory gier jak Unity z wersjonowaniem asset'ów.

```
Przykład: Game Level Editor
- Scene versioning
- Texture/model deduplication
- Collaborative editing (with sync layer)
```

### 4. Document Editors
Edytory dokumentów jak Google Docs z offline-first approach.

```
Przykład: Rich Text Editor
- Document history
- Binary embedding (images, files)
- Conflict-free merges (future)
```

### 5. Project Management & CAD
Narzędzia CAD i 3D modeling z kompleksowym version control.

```
Przykład: 3D Modeling Tool
- Model versioning
- Material library (deduped)
- Large file support (streaming)
```

---

## Architektura High-Level

```
┌─────────────────────────────────────────┐
│         User Application                │
│    (Browser/Node/Tauri/Deno)           │
└────────────────┬────────────────────────┘
                 │
                 │ Public API
                 ↓
┌─────────────────────────────────────────┐
│         JCFManager Class                │
│  (Facade for all operations)            │
└────────────────┬────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
        ↓                 ↓
┌──────────────┐  ┌──────────────┐
│ Core Layer   │  │ Storage      │
│ - Versioning │  │ - ZIP        │
│ - Diffing    │  │ - CAS        │
│ - CAS        │  │ - Deltas     │
└──────┬───────┘  └──────┬───────┘
       │                 │
       │    ┌────────────┘
       ↓    ↓
┌──────────────────────┐
│  Adapter Layer       │
│  (Platform-specific) │
│  - Browser           │
│  - Node.js           │
│  - Tauri             │
└──────────────────────┘
```

---

## Kluczowe Technologie

| Komponent | Technologia | Dlaczego? |
|-----------|-------------|-----------|
| **Kompresja** | fflate | 20x szybsza od JSZip, native streaming |
| **Diffing** | diff-match-patch | Battle-tested (Google Docs), pełne API |
| **Hashing** | WebCrypto/hash-wasm | Native, secure SHA-256 |
| **ID Generation** | uuid v4 | Collision-resistant, standard |
| **Type Safety** | TypeScript 5.3+ | Compile-time errors, better DX |

---

## Korzyści dla Użytkowników

### Dla Developerów
- **Zero configuration**: Działa out-of-the-box
- **Type-safe API**: TypeScript interfaces
- **Platform agnostic**: Jeden kod, wszystkie platformy
- **Extensible**: Adapter pattern, plugin system (future)

### Dla End-Users
- **Offline-first**: Nie wymaga serwera
- **Fast**: Reverse delta = instant HEAD access
- **Reliable**: ZIP format = standard recovery tools
- **Efficient**: Deduplication = mniejszy rozmiar

---

## Porównanie z Alternatywami

| Feature | Kamaros JCF | Git (isomorphic-git) | Custom Format |
|---------|-------------|----------------------|---------------|
| Browser support | ✅ Native | ✅ Port | ⚠️ Custom |
| File size (large) | ✅ Streaming | ❌ RAM limit | ✅ Custom |
| Binary dedup | ✅ CAS | ❌ No | ⚠️ Manual |
| HEAD performance | ✅ O(1) | ✅ O(1) | ⚠️ Varies |
| Recovery tools | ✅ ZIP tools | ✅ Git | ❌ None |
| Learning curve | ✅ Simple | ⚠️ Steep | ❌ Unknown |

**Verdict**: Kamaros wypełnia niszę między Git (zbyt złożony, słaby z binariami) a custom solutions (brak standardów).

---

## Next Steps

Teraz, gdy rozumiesz czym jest Kamaros, przejdźmy do szczegółowego problemu, który rozwiązuje:

**→ [02. Problem Statement](02-problem-statement.md)**: Dlaczego potrzebujemy nowego formatu?

---

[← Back to Index](../README.md) | [Next: Problem Statement →](02-problem-statement.md)

