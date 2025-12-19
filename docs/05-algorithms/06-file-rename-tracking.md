# File Rename Tracking

> **Szczegółowy algorytm śledzenia zmian nazw plików przy użyciu systemu inode**

[← Back: Garbage Collection](05-garbage-collection.md) | [Next: Conflict Resolution →](07-conflict-resolution.md)

---

## Overview

Algorytm **File Rename Tracking** odpowiada za śledzenie zmian nazw plików (rename/move) w historii wersji. Używa systemu **inode** - każdy plik ma unikalny UUID (inodeId), który pozostaje stały nawet gdy plik jest przemianowany.

**Kluczowe założenie**: Historia pliku jest powiązana z `inodeId`, nie z `path`. Dzięki temu rename nie powoduje utraty historii.

---

## Inode System

### Koncepcja

**Tradycyjny system** (path-based):
```
v1: src/index.js → historia
v2: src/main.js  → nowa historia (utrata!)
```

**Inode system** (inode-based):
```
v1: src/index.js (inodeId: abc123) → historia
v2: src/main.js (inodeId: abc123)  → ta sama historia!
```

### Struktura Inode

```typescript
interface FileEntry {
  inodeId: string;        // UUID v4 - unikalny identyfikator
  path: string;           // Aktualna ścieżka (może się zmieniać)
  type: 'text' | 'binary';
  currentHash?: string;
  created: string;        // Timestamp utworzenia
  modified: string;       // Timestamp ostatniej modyfikacji
}
```

**Właściwości inodeId**:
- **Unikalny**: UUID v4 - praktycznie zero szans na kolizję
- **Niezmienny**: Nie zmienia się przez rename/move
- **Trwały**: Pozostaje przez całą historię pliku

---

## Algorithm: Detect Rename

### Step 1: Identify File Changes

```typescript
async function identifyFileChanges(
  currentFiles: Map<string, FileEntry>,
  headFiles: Map<string, FileEntry>
): Promise<FileChange[]> {
  const changes: FileChange[] = [];
  
  // Check for added/modified files
  for (const [path, fileEntry] of currentFiles) {
    const headEntry = headFiles.get(path);
    
    if (!headEntry) {
      // New file or renamed file?
      // Check if inodeId exists elsewhere
      const existingInode = this.findInodeInHistory(fileEntry.inodeId);
      
      if (existingInode) {
        // This is a rename!
        changes.push({
          type: 'renamed',
          inodeId: fileEntry.inodeId,
          fromPath: existingInode.path,
          toPath: path
        });
      } else {
        // New file
        changes.push({
          type: 'added',
          path,
          inodeId: fileEntry.inodeId
        });
      }
    } else if (headEntry.inodeId !== fileEntry.inodeId) {
      // Path exists but different inodeId
      // This is a replace (delete old + add new)
      changes.push({
        type: 'replaced',
        oldInodeId: headEntry.inodeId,
        newInodeId: fileEntry.inodeId,
        path
      });
    } else if (headEntry.currentHash !== fileEntry.currentHash) {
      // Same inodeId, different content = modified
      changes.push({
        type: 'modified',
        path,
        inodeId: fileEntry.inodeId
      });
    }
  }
  
  // Check for deleted files
  for (const [path, headEntry] of headFiles) {
    if (!currentFiles.has(path)) {
      // File deleted or renamed?
      // Check if inodeId exists elsewhere
      const existingPath = this.findInodeInCurrent(headEntry.inodeId);
      
      if (existingPath) {
        // This is a rename (already handled above)
        continue;
      } else {
        // File deleted
        changes.push({
          type: 'deleted',
          path,
          inodeId: headEntry.inodeId
        });
      }
    }
  }
  
  return changes;
}
```

### Step 2: Find Inode in History

```typescript
function findInodeInHistory(inodeId: string): FileEntry | null {
  // Search through all versions
  for (const version of this.manifest.versionHistory) {
    for (const [path, fileState] of Object.entries(version.fileStates)) {
      const fileEntry = this.manifest.fileMap[path];
      
      if (fileEntry && fileEntry.inodeId === inodeId) {
        return {
          ...fileEntry,
          path  // Return path from that version
        };
      }
    }
  }
  
  return null;
}

function findInodeInCurrent(inodeId: string): string | null {
  // Search in current fileMap
  for (const [path, fileEntry] of Object.entries(this.manifest.fileMap)) {
    if (fileEntry.inodeId === inodeId) {
      return path;
    }
  }
  
  return null;
}
```

### Step 3: Record Rename in RenameLog

```typescript
interface RenameEntry {
  inodeId: string;
  fromPath: string;
  toPath: string;
  versionId: string;
  timestamp: string;
}

async function recordRename(
  inodeId: string,
  fromPath: string,
  toPath: string,
  versionId: string
): Promise<void> {
  const renameEntry: RenameEntry = {
    inodeId,
    fromPath,
    toPath,
    versionId,
    timestamp: new Date().toISOString()
  };
  
  // Add to rename log
  this.manifest.renameLog.push(renameEntry);
  
  // Update fileMap
  const fileEntry = this.manifest.fileMap[fromPath];
  if (fileEntry) {
    // Remove old path
    delete this.manifest.fileMap[fromPath];
    
    // Add new path with same inodeId
    this.manifest.fileMap[toPath] = {
      ...fileEntry,
      path: toPath,
      modified: renameEntry.timestamp
    };
  }
}
```

---

## Complete Algorithm: Handle Rename

```typescript
async function handleRename(
  fromPath: string,
  toPath: string
): Promise<void> {
  // Get file entry
  const fileEntry = this.manifest.fileMap[fromPath];
  
  if (!fileEntry) {
    throw new FileNotFoundError(`File not found: ${fromPath}`);
  }
  
  // Check if target path exists
  if (this.manifest.fileMap[toPath]) {
    throw new FileExistsError(`File already exists: ${toPath}`);
  }
  
  // Move file in working copy
  await this.adapter.moveFile(
    `content/${fromPath}`,
    `content/${toPath}`
  );
  
  // Record rename
  const versionId = this.manifest.refs.head;
  await this.recordRename(
    fileEntry.inodeId,
    fromPath,
    toPath,
    versionId
  );
  
  // Update fileMap
  delete this.manifest.fileMap[fromPath];
  this.manifest.fileMap[toPath] = {
    ...fileEntry,
    path: toPath,
    modified: new Date().toISOString()
  };
  
  // Update all version fileStates that reference this inodeId
  await this.updateVersionFileStates(
    fileEntry.inodeId,
    fromPath,
    toPath
  );
}
```

---

## Query File History Across Renames

### Get File History by Inode

```typescript
async function getFileHistoryByInode(
  inodeId: string
): Promise<FileHistoryEntry[]> {
  const history: FileHistoryEntry[] = [];
  
  // Find all versions that reference this inodeId
  for (const version of this.manifest.versionHistory) {
    for (const [path, fileState] of Object.entries(version.fileStates)) {
      const fileEntry = this.manifest.fileMap[path];
      
      if (fileEntry && fileEntry.inodeId === inodeId) {
        history.push({
          versionId: version.id,
          path,
          timestamp: version.timestamp,
          changeType: fileState.changeType,
          size: fileState.size,
          hash: fileState.hash
        });
      }
    }
  }
  
  // Sort by timestamp
  history.sort((a, b) => 
    new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  );
  
  return history;
}
```

### Get Current Path for Inode

```typescript
function getCurrentPathForInode(inodeId: string): string | null {
  for (const [path, fileEntry] of Object.entries(this.manifest.fileMap)) {
    if (fileEntry.inodeId === inodeId) {
      return path;
    }
  }
  
  return null;
}
```

### Get All Paths for Inode (History)

```typescript
function getAllPathsForInode(inodeId: string): string[] {
  const paths = new Set<string>();
  
  // Current path
  const currentPath = this.getCurrentPathForInode(inodeId);
  if (currentPath) {
    paths.add(currentPath);
  }
  
  // Historical paths from rename log
  for (const renameEntry of this.manifest.renameLog) {
    if (renameEntry.inodeId === inodeId) {
      paths.add(renameEntry.fromPath);
      paths.add(renameEntry.toPath);
    }
  }
  
  return Array.from(paths);
}
```

---

## Rename Detection During Save Checkpoint

```typescript
async function saveCheckpoint(message: string): Promise<string> {
  // ... identify changes ...
  
  // Detect renames
  const renames = changes.filter(c => c.type === 'renamed');
  
  for (const rename of renames) {
    // Record rename
    await this.recordRename(
      rename.inodeId,
      rename.fromPath,
      rename.toPath,
      newVersionId
    );
    
    // Update fileStates in new version
    // (inodeId stays the same, path changes)
    newVersion.fileStates[rename.toPath] = {
      inodeId: rename.inodeId,
      path: rename.toPath,
      changeType: 'renamed',
      // Copy other properties from old path
      ...oldVersion.fileStates[rename.fromPath]
    };
  }
  
  // ... rest of save checkpoint ...
}
```

---

## Edge Cases

### Case 1: Rename to Existing Path

```typescript
// Problem: Rename 'old.js' → 'new.js', ale 'new.js' już istnieje
// Rozwiązanie: Throw error lub replace (zależy od opcji)

if (this.manifest.fileMap[toPath]) {
  if (options.replace) {
    // Delete existing file first
    await this.deleteFile(toPath);
  } else {
    throw new FileExistsError(`File already exists: ${toPath}`);
  }
}
```

### Case 2: Multiple Renames in Same Version

```typescript
// Problem: Plik został przemianowany wielokrotnie w jednej wersji
// old.js → temp.js → new.js

// Rozwiązanie: Record only final rename (old.js → new.js)
// Intermediate renames are not recorded separately
```

### Case 3: Rename + Modify in Same Version

```typescript
// Problem: Plik został przemianowany I zmodyfikowany
// old.js (content: "A") → new.js (content: "B")

// Rozwiązanie: Record as 'renamed' + 'modified'
// Both changes are tracked
```

### Case 4: Circular Rename

```typescript
// Problem: old.js → new.js, new.js → old.js (w różnych wersjach)
// Rozwiązanie: Każdy rename jest recordowany osobno
// System radzi sobie z tym poprawnie
```

---

## Rename Log Structure

```typescript
interface RenameLog {
  entries: RenameEntry[];
}

// Przykład:
{
  "renameLog": [
    {
      "inodeId": "abc123-def456-...",
      "fromPath": "src/index.js",
      "toPath": "src/main.js",
      "versionId": "v5-xyz789",
      "timestamp": "2025-01-15T10:30:00Z"
    },
    {
      "inodeId": "abc123-def456-...",
      "fromPath": "src/main.js",
      "toPath": "lib/entry.js",
      "versionId": "v8-uvw012",
      "timestamp": "2025-01-20T14:20:00Z"
    }
  ]
}
```

**Interpretacja**: Plik z inodeId `abc123-def456-...` był:
1. `src/index.js` (v1-v4)
2. `src/main.js` (v5-v7)
3. `lib/entry.js` (v8-HEAD)

---

## Performance Considerations

### Inode Lookup Optimization

```typescript
class InodeIndex {
  private inodeToPath = new Map<string, string>();
  private pathToInode = new Map<string, string>();
  
  buildIndex(manifest: Manifest): void {
    for (const [path, fileEntry] of Object.entries(manifest.fileMap)) {
      this.inodeToPath.set(fileEntry.inodeId, path);
      this.pathToInode.set(path, fileEntry.inodeId);
    }
  }
  
  getPathForInode(inodeId: string): string | null {
    return this.inodeToPath.get(inodeId) || null;
  }
  
  getInodeForPath(path: string): string | null {
    return this.pathToInode.get(path) || null;
  }
}
```

**Korzyść**: O(1) lookup zamiast O(N) przeszukiwania.

---

## Best Practices

### Do's ✅

1. **Zawsze używaj inodeId** do śledzenia historii pliku
2. **Record wszystkie renames** w renameLog
3. **Update fileStates** w nowych wersjach
4. **Query przez inodeId** dla pełnej historii
5. **Validate renames** przed zapisem

### Don'ts ❌

1. **Nie używaj path** do śledzenia historii (zmienia się)
2. **Nie ignoruj renames** - tracisz historię
3. **Nie duplikuj inodeId** - każdy plik = unikalny inodeId
4. **Nie modyfikuj inodeId** - jest niezmienny

---

## Integration

```typescript
// Move file operation
async function moveFile(fromPath: string, toPath: string): Promise<void> {
  await this.handleRename(fromPath, toPath);
}

// Get file history (handles renames automatically)
async function getFileHistory(path: string): Promise<FileHistoryEntry[]> {
  const fileEntry = this.manifest.fileMap[path];
  if (!fileEntry) {
    throw new FileNotFoundError(path);
  }
  
  // Get history by inodeId (includes all renames)
  return await this.getFileHistoryByInode(fileEntry.inodeId);
}
```

---

## Testing

```typescript
describe('File Rename Tracking', () => {
  it('should track rename across versions', async () => {
    // Add file
    await manager.addFile('old.js', 'content');
    const v1 = await manager.saveCheckpoint('Add file');
    
    // Rename file
    await manager.moveFile('old.js', 'new.js');
    const v2 = await manager.saveCheckpoint('Rename file');
    
    // Get history - should include both paths
    const history = await manager.getFileHistory('new.js');
    expect(history.length).toBe(2);
    expect(history[0].path).toBe('old.js');
    expect(history[1].path).toBe('new.js');
  });
  
  it('should preserve inodeId across rename', async () => {
    await manager.addFile('old.js', 'content');
    const fileEntry1 = await manager.getFileInfo('old.js');
    
    await manager.moveFile('old.js', 'new.js');
    const fileEntry2 = await manager.getFileInfo('new.js');
    
    expect(fileEntry1.inodeId).toBe(fileEntry2.inodeId);
  });
});
```

---

## Kompletny Przykład: Rename Detection w Save Checkpoint

```typescript
class FileManager {
  async saveCheckpoint(message: string): Promise<string> {
    // 1. Identify changes
    const currentFiles = await this.listWorkingCopyFiles();
    // currentFiles = ['src/main.js', 'assets/logo.png']
    
    const headVersion = this.getVersion(this.manifest.refs.head);
    const headFiles = new Set(Object.keys(headVersion.fileStates));
    // headFiles = Set(['src/index.js', 'assets/logo.png'])
    
    // 2. Detect renames
    const changes: FileChange[] = [];
    
    // Sprawdź nowe pliki
    for (const path of currentFiles) {
      if (!headFiles.has(path)) {
        // Nowy plik lub rename?
        const fileEntry = this.manifest.fileMap[path];
        const inodeId = fileEntry?.inodeId;
        
        // Szukaj tego inodeId w historii
        const existingPath = this.findInodeInHistory(inodeId);
        
        if (existingPath) {
          // To jest rename!
          changes.push({
            type: 'renamed',
            inodeId,
            fromPath: existingPath,  // 'src/index.js'
            toPath: path             // 'src/main.js'
          });
        } else {
          // Nowy plik
          changes.push({ type: 'added', path, inodeId });
        }
      }
    }
    
    // 3. Record rename w renameLog
    for (const change of changes.filter(c => c.type === 'renamed')) {
      const newVersionId = uuidv4();
      await this.recordRename(
        change.inodeId,
        change.fromPath,
        change.toPath,
        newVersionId
      );
      
      // Update fileMap
      const oldEntry = this.manifest.fileMap[change.fromPath];
      delete this.manifest.fileMap[change.fromPath];
      this.manifest.fileMap[change.toPath] = {
        ...oldEntry,
        path: change.toPath,
        modified: new Date().toISOString()
      };
    }
    
    // 4. Create version
    const versionId = await this.createVersion(message, changes);
    return versionId;
  }
  
  private findInodeInHistory(inodeId: string): string | null {
    // Szukaj w wszystkich wersjach
    for (const version of this.manifest.versionHistory) {
      for (const [path, fileState] of Object.entries(version.fileStates)) {
        const fileEntry = this.manifest.fileMap[path];
        if (fileEntry?.inodeId === inodeId) {
          return path;
        }
      }
    }
    return null;
  }
}
```

**Przykład wykonania**:
```typescript
// v1: Utworzenie pliku
await manager.addFile('src/index.js', 'console.log("Hello");');
const v1 = await manager.saveCheckpoint('Add index.js');
// manifest.fileMap['src/index.js'] = { inodeId: 'abc123', ... }

// v2: Rename
await manager.moveFile('src/index.js', 'src/main.js');
const v2 = await manager.saveCheckpoint('Rename to main.js');
// 1. Wykryto: 'src/main.js' nie istnieje w v1, ale inodeId 'abc123' istnieje
// 2. Zidentyfikowano jako rename: 'src/index.js' → 'src/main.js'
// 3. Zapisano w renameLog:
//    { inodeId: 'abc123', fromPath: 'src/index.js', toPath: 'src/main.js', versionId: 'v2' }
// 4. Zaktualizowano fileMap:
//    - Usunięto: fileMap['src/index.js']
//    - Dodano: fileMap['src/main.js'] = { inodeId: 'abc123', path: 'src/main.js', ... }
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
