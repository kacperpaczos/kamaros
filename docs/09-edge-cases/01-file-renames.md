# File Renames - Edge Cases

> **Szczegółowe scenariusze i rozwiązania dla zmian nazw plików**

[← Back: Advanced Patterns](../08-usage-guide/08-advanced-patterns.md) | [Next: Type Changes →](02-type-changes.md)

---

## Overview

System Kamaros używa **inode system** do śledzenia plików przez renames. Każdy plik ma unikalny `inodeId` (UUID v4), który pozostaje stały nawet gdy plik jest przemianowany. Dzięki temu historia pliku jest zachowana przez wszystkie renames.

**Kluczowa koncepcja**: Historia jest powiązana z `inodeId`, nie z `path`.

---

## Scenariusz 1: Podstawowy Rename

### Opis

Użytkownik przemianowuje plik `old.js` → `new.js` w jednej wersji.

### Implementacja

```typescript
// v1: Utworzenie pliku
await manager.addFile('old.js', 'console.log("Hello");');
const v1 = await manager.saveCheckpoint('Add old.js');

// v2: Rename
await manager.moveFile('old.js', 'new.js');
const v2 = await manager.saveCheckpoint('Rename to new.js');
```

### Co się dzieje wewnętrznie

1. **InodeId pozostaje ten sam**:
   - `old.js` (v1): `inodeId: abc123`
   - `new.js` (v2): `inodeId: abc123` ✅

2. **RenameLog jest aktualizowany**:
   ```json
   {
     "renameLog": [
       {
         "inodeId": "abc123",
         "fromPath": "old.js",
         "toPath": "new.js",
         "versionId": "v2",
         "timestamp": "2025-01-15T10:30:00Z"
       }
     ]
   }
   ```

3. **FileMap jest aktualizowany**:
   - Usuwa: `fileMap['old.js']`
   - Dodaje: `fileMap['new.js']` (z tym samym inodeId)

### Query History

```typescript
// Historia przez aktualną ścieżkę
const history = await manager.getFileHistory('new.js');
// Zwraca historię od v1 (jako old.js) do v2 (jako new.js)

// Historia przez starą ścieżkę (też działa!)
const oldHistory = await manager.getFileHistory('old.js');
// Zwraca tę samą historię (przez inodeId)
```

**Wniosek**: Historia jest dostępna przez obie ścieżki (starą i nową).

---

## Scenariusz 2: Wielokrotne Renames

### Opis

Plik jest przemianowany wielokrotnie: `a.js` → `b.js` → `c.js`.

### Implementacja

```typescript
// v1: Utworzenie
await manager.addFile('a.js', 'content');
const v1 = await manager.saveCheckpoint('Add a.js');

// v2: Pierwszy rename
await manager.moveFile('a.js', 'b.js');
const v2 = await manager.saveCheckpoint('Rename a → b');

// v3: Drugi rename
await manager.moveFile('b.js', 'c.js');
const v3 = await manager.saveCheckpoint('Rename b → c');
```

### RenameLog

```json
{
  "renameLog": [
    { "inodeId": "abc123", "fromPath": "a.js", "toPath": "b.js", "versionId": "v2" },
    { "inodeId": "abc123", "fromPath": "b.js", "toPath": "c.js", "versionId": "v3" }
  ]
}
```

### Query przez dowolną ścieżkę

```typescript
// Wszystkie te zapytania zwracają tę samą historię:
await manager.getFileHistory('a.js');  // ✅
await manager.getFileHistory('b.js');  // ✅
await manager.getFileHistory('c.js');  // ✅

// Historia zawiera wszystkie ścieżki:
[
  { path: 'a.js', versionId: 'v1', changeType: 'added' },
  { path: 'b.js', versionId: 'v2', changeType: 'renamed' },
  { path: 'c.js', versionId: 'v3', changeType: 'renamed' }
]
```

**Wniosek**: System śledzi wszystkie renames i historia jest kompletna.

---

## Scenariusz 3: Rename + Modify w Tej Samej Wersji

### Opis

Plik jest przemianowany I zmodyfikowany w jednym checkpoint.

### Implementacja

```typescript
// v1: Utworzenie
await manager.addFile('old.js', 'console.log("A");');
const v1 = await manager.saveCheckpoint('Add old.js');

// v2: Rename + Modify
await manager.moveFile('old.js', 'new.js');
await manager.addFile('new.js', 'console.log("B");');  // Modify
const v2 = await manager.saveCheckpoint('Rename and modify');
```

### Co się dzieje

1. **Rename jest recordowany** w renameLog
2. **Modify jest recordowany** w fileStates jako `changeType: 'modified'`
3. **Oba zmiany są w tej samej wersji**

### Query

```typescript
const history = await manager.getFileHistory('new.js');
// [
//   { path: 'old.js', versionId: 'v1', changeType: 'added' },
//   { path: 'new.js', versionId: 'v2', changeType: 'renamed' },  // Rename
//   { path: 'new.js', versionId: 'v2', changeType: 'modified' }  // Modify
// ]
```

**Wniosek**: Oba typy zmian są śledzone osobno.

---

## Scenariusz 4: Rename do Istniejącego Pliku

### Opis

Próba przemianowania pliku na ścieżkę, która już istnieje.

### Problem

```typescript
await manager.addFile('old.js', 'content A');
await manager.addFile('new.js', 'content B');

// Próba rename old.js → new.js
await manager.moveFile('old.js', 'new.js');  // ❌ Error!
```

### Rozwiązanie A: Error (Domyślne)

```typescript
try {
  await manager.moveFile('old.js', 'new.js');
} catch (error) {
  if (error instanceof FileExistsError) {
    console.error('Target file already exists');
    // Użytkownik musi najpierw usunąć new.js lub użyć replace
  }
}
```

### Rozwiązanie B: Replace (Opcjonalne)

```typescript
// Future: Możliwość replace
await manager.moveFile('old.js', 'new.js', {
  replace: true  // Usuń istniejący new.js i zastąp go old.js
});
```

**Wniosek**: Domyślnie rename do istniejącego pliku jest błędem (bezpieczne).

---

## Scenariusz 5: Circular Rename

### Opis

Plik A → B, potem B → A (w różnych wersjach).

### Implementacja

```typescript
// v1: Utworzenie
await manager.addFile('a.js', 'content');
const v1 = await manager.saveCheckpoint('Add a.js');

// v2: Rename a → b
await manager.moveFile('a.js', 'b.js');
const v2 = await manager.saveCheckpoint('Rename a → b');

// v3: Rename b → a
await manager.moveFile('b.js', 'a.js');
const v3 = await manager.saveCheckpoint('Rename b → a');
```

### Co się dzieje

1. **InodeId pozostaje ten sam** przez wszystkie renames
2. **RenameLog zawiera wszystkie renames**:
   ```json
   [
     { "fromPath": "a.js", "toPath": "b.js", "versionId": "v2" },
     { "fromPath": "b.js", "toPath": "a.js", "versionId": "v3" }
   ]
   ```
3. **Historia jest kompletna**:
   ```typescript
   const history = await manager.getFileHistory('a.js');
   // [
   //   { path: 'a.js', versionId: 'v1' },
   //   { path: 'b.js', versionId: 'v2' },
   //   { path: 'a.js', versionId: 'v3' }  // Z powrotem do a.js!
   // ]
   ```

**Wniosek**: System radzi sobie z circular renames poprawnie.

---

## Scenariusz 6: Rename + Delete

### Opis

Plik jest przemianowany, potem usunięty.

### Implementacja

```typescript
// v1: Utworzenie
await manager.addFile('old.js', 'content');
const v1 = await manager.saveCheckpoint('Add old.js');

// v2: Rename
await manager.moveFile('old.js', 'new.js');
const v2 = await manager.saveCheckpoint('Rename');

// v3: Delete
await manager.removeFile('new.js');
const v3 = await manager.saveCheckpoint('Delete');
```

### Co się dzieje

1. **Rename jest recordowany** w v2
2. **Delete jest recordowany** w v3
3. **Historia zawiera wszystkie zmiany**:
   ```typescript
   const history = await manager.getFileHistory('new.js');
   // [
   //   { path: 'old.js', versionId: 'v1', changeType: 'added' },
   //   { path: 'new.js', versionId: 'v2', changeType: 'renamed' },
   //   { path: 'new.js', versionId: 'v3', changeType: 'deleted' }
   // ]
   ```

### Restore do v1 (przed rename)

```typescript
await manager.restoreVersion('v1');
// Plik jest dostępny jako 'old.js' (oryginalna ścieżka)
```

**Wniosek**: Historia jest zachowana nawet po delete.

---

## Scenariusz 7: Rename w Głębokiej Historii

### Opis

Plik został przemianowany 50 wersji temu. Czy historia jest dostępna?

### Problem

```typescript
// v1: Utworzenie jako 'old.js'
// v50: Rename 'old.js' → 'new.js'
// v100: HEAD (aktualna wersja)

// Query przez starą ścieżkę
const history = await manager.getFileHistory('old.js');
// Czy to działa?
```

### Rozwiązanie

**Tak, działa!** System używa inodeId do znajdowania historii:

```typescript
async function getFileHistory(path: string): Promise<FileHistoryEntry[]> {
  // 1. Znajdź inodeId dla aktualnej ścieżki
  let fileEntry = this.manifest.fileMap[path];
  
  // 2. Jeśli nie znaleziono, szukaj w renameLog
  if (!fileEntry) {
    const inodeId = this.findInodeInRenameLog(path);
    if (inodeId) {
      // Znajdź aktualną ścieżkę dla tego inodeId
      path = this.getCurrentPathForInode(inodeId);
      fileEntry = this.manifest.fileMap[path];
    }
  }
  
  if (!fileEntry) {
    throw new FileNotFoundError(path);
  }
  
  // 3. Pobierz historię przez inodeId
  return await this.getFileHistoryByInode(fileEntry.inodeId);
}
```

**Wniosek**: Historia jest dostępna przez dowolną ścieżkę (starą lub nową).

---

## Scenariusz 8: Concurrent Renames (Future)

### Opis

Dwa procesy próbują przemianować ten sam plik jednocześnie.

### Problem

```
Process A: old.js → new-a.js
Process B: old.js → new-b.js
```

### Obecne Zachowanie

- Ostatni zapis wygrywa (last write wins)
- Możliwe race condition

### Future: Conflict Resolution

```typescript
// v2.0: Merge conflict resolution
try {
  await manager.moveFile('old.js', 'new.js');
} catch (error) {
  if (error instanceof RenameConflictError) {
    // Rozwiąż konflikt
    await manager.resolveRenameConflict('old.js', {
      preferredPath: 'new.js',
      strategy: 'manual'  // lub 'auto'
    });
  }
}
```

---

## Best Practices

### Do's ✅

1. **Używaj `moveFile()`** zamiast `removeFile()` + `addFile()` dla renames
2. **Query przez aktualną ścieżkę** (szybsze)
3. **Sprawdzaj `FileNotFoundError`** - może oznaczać rename
4. **Używaj `getFileHistory()`** dla pełnej historii (włącznie z renames)

### Don'ts ❌

1. **Nie używaj path** do śledzenia historii (zmienia się)
2. **Nie ignoruj renames** - tracisz historię
3. **Nie modyfikuj inodeId** - jest niezmienny
4. **Nie używaj `removeFile()` + `addFile()`** dla renames (utrata historii)

---

## Troubleshooting

### Problem: "File not found" po rename

```typescript
// ❌ Błędne
const content = await manager.getFileContent('old.js');  // FileNotFoundError

// ✅ Poprawne
// Sprawdź czy plik został przemianowany
const history = await manager.getFileHistory('old.js');
const currentPath = history[history.length - 1].path;
const content = await manager.getFileContent(currentPath);
```

### Problem: Historia nie zawiera wszystkich zmian

```typescript
// Upewnij się, że używasz inodeId
const fileEntry = await manager.getFileInfo('current-path.js');
const history = await manager.getFileHistoryByInode(fileEntry.inodeId);
// Zawiera wszystkie zmiany (włącznie z renames)
```

---

## Implementacja Inode System - Szczegóły Techniczne

```typescript
// Struktura FileEntry w manifest.json
interface FileEntry {
  inodeId: string;        // UUID v4 - unikalny, niezmienny
  path: string;           // Aktualna ścieżka (może się zmieniać)
  type: 'text' | 'binary';
  currentHash?: string;   // SHA-256 dla binariów
  created: string;        // ISO timestamp
  modified: string;       // ISO timestamp
}

// Przykład manifest.fileMap:
{
  "src/main.js": {
    "inodeId": "550e8400-e29b-41d4-a716-446655440000",
    "path": "src/main.js",
    "type": "text",
    "created": "2025-01-10T10:00:00Z",
    "modified": "2025-01-15T10:30:00Z"
  }
}

// Struktura RenameEntry w manifest.renameLog:
interface RenameEntry {
  inodeId: string;
  fromPath: string;
  toPath: string;
  versionId: string;
  timestamp: string;
}

// Przykład renameLog:
[
  {
    "inodeId": "550e8400-e29b-41d4-a716-446655440000",
    "fromPath": "src/index.js",
    "toPath": "src/main.js",
    "versionId": "v2-abc123",
    "timestamp": "2025-01-15T10:30:00Z"
  }
]

// Query przez inodeId - implementacja
async function getFileHistoryByInode(inodeId: string): Promise<FileHistoryEntry[]> {
  const history: FileHistoryEntry[] = [];
  
  // 1. Znajdź wszystkie wersje używające tego inodeId
  for (const version of this.manifest.versionHistory) {
    for (const [path, fileState] of Object.entries(version.fileStates)) {
      const fileEntry = this.manifest.fileMap[path];
      if (fileEntry?.inodeId === inodeId) {
        history.push({
          versionId: version.id,
          path,  // Może się zmieniać przez renames
          timestamp: version.timestamp,
          changeType: fileState.changeType || 'modified',
          size: fileState.size,
          hash: fileState.hash,
          author: version.author,
          message: version.message
        });
      }
    }
  }
  
  // 2. Sortuj chronologicznie
  history.sort((a, b) => 
    new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  );
  
  return history;
}

// Użycie:
const fileEntry = await manager.getFileInfo('src/main.js');
const fullHistory = await manager.getFileHistoryByInode(fileEntry.inodeId);
// Zwraca historię przez wszystkie renames
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
