# Key Concepts

> **Podstawowe koncepty techniczne Kamaros**

[← Back: Problem Statement](02-problem-statement.md) | [Next: Use Cases →](04-use-cases.md)

---

## 1. Time-Travel Versioning

Możliwość cofnięcia projektu do dowolnego punktu w historii.

```typescript
// Create checkpoint
const v1 = await manager.saveCheckpoint('Initial commit');
// ... make changes ...
const v2 = await manager.saveCheckpoint('Add login feature');

// Travel back in time
await manager.restoreVersion(v1); // Project now at v1 state
```

---

## 2. Reverse Delta Strategy

**HEAD zawsze pełny** - starsze wersje jako patche.

```
v1 (old) ← patch ← v2 ← patch ← v3 (HEAD, full)
```

**Why?** 95% czasu pracujesz z HEAD → optymalizujemy dla common case.

---

## 3. Content Addressable Storage (CAS)

Pliki binarne identyfikowane przez hash (SHA-256). Ten sam content = ten sam hash = jedna kopia.

```
logo.png (hash: abc123) → .store/blobs/abc123
icon.png (różny, hash: def456) → .store/blobs/def456
logo-copy.png (ten sam, hash: abc123) → reuse abc123
```

**Benefit**: Automatyczna deduplikacja.

---

## 4. ZIP Container

Format JCF to standardowy ZIP. Można unzipnąć zwykłymi narzędziami!

```bash
unzip project.jcf
# content/ folder = latest version
```

---

## 5. Streaming Architecture

Duże pliki (>50MB) przetwarzane jako stream → nie ładujemy całości do RAM.

---

## 6. Platform Abstraction

Adapter Pattern - ten sam kod działa wszędzie:
- Browser (IndexedDB)
- Node.js (fs)
- Tauri (tauri.fs)

---

[← Back: Problem Statement](02-problem-statement.md) | [Next: Use Cases →](04-use-cases.md)

