# 🎯 JCF Manager - Kompletna Specyfikacja Implementacyjna

> **Dokument Finalny**: Master specification dla implementacji biblioteki JCF Manager
> 
> **Status**: Ready for Implementation  
> **Wersja**: 1.0.0  
> **Data**: 2025-12-18

---

## 📋 Spis Treści

1. [Executive Summary](#1-executive-summary)
2. [Stack Technologiczny](#2-stack-technologiczny)
3. [Wzorce Projektowe](#3-wzorce-projektowe)
4. [Struktury Danych](#4-struktury-danych)
5. [Algorytmy](#5-algorytmy)
6. [Metody i Funkcje](#6-metody-i-funkcje)
7. [Pipeline Implementacji](#7-pipeline-implementacji)
8. [Decyzje Architektoniczne](#8-decyzje-architektoniczne)

---

## 1. Executive Summary

### 1.1 Czym jest JCF Manager?

**JCF (JSON Container Format) Manager** to izomorficzna biblioteka JavaScript/TypeScript do zarządzania plikami projektowymi z wbudowanym systemem wersjonowania (Time-Travel Versioning).

### 1.2 Kluczowe Cechy

- ✅ **Reverse Delta Versioning**: HEAD zawsze pełny, historia jako patche
- ✅ **Content Addressable Storage**: Automatyczna deduplikacja binarnych
- ✅ **Izomorficzność**: Browser + Node.js + Tauri + Deno
- ✅ **Streaming Support**: Obsługa plików >500MB bez RAM overflow
- ✅ **Multi-threading**: Web Workers dla CPU-intensive operacji
- ✅ **Production Ready**: Error handling, validation, testing

### 1.3 Use Cases

1. **Code Editors**: VSCode-like apps z historią zmian
2. **Design Tools**: Figma-like apps z version control
3. **Game Editors**: Unity-like editors z asset versioning
4. **Document Editors**: Google Docs-like z offline-first
5. **Project Management**: CAD/3D modeling tools

---

## 2. Stack Technologiczny

### 2.1 Core Dependencies

#### 2.1.1 fflate (ZIP Compression)

**Dlaczego fflate?**

| Kryterium | fflate | JSZip | Waga | Wynik |
|-----------|--------|-------|------|-------|
| Performance (compression) | 9/10 | 3/10 | 40% | fflate: 3.6 |
| Bundle Size | 10/10 | 5/10 | 20% | fflate: 2.0 |
| Streaming API | 10/10 | 4/10 | 25% | fflate: 2.5 |
| Ease of Use | 6/10 | 10/10 | 15% | fflate: 0.9 |

**Total Score**: fflate 9.0 vs JSZip 5.5 → **fflate wins**

**Funkcje używane**:
```typescript
import {
  Zip,                    // Synchronous ZIP creation
  AsyncZipDeflate,        // Async compression stream
  Unzip,                  // Synchronous extraction
  UnzipInflate,           // Decompression codec
  strFromU8,              // Uint8Array → string
  strToU8                 // string → Uint8Array
} from 'fflate';
```

**Powód wyboru**:
- 20x szybsza kompresja vs JSZip
- 5x szybsza dekompresja vs JSZip
- Natywny streaming (AsyncZipDeflate)
- Automatyczny multi-core (Web Workers)
- Tree-shakeable (8kb min+gzip)

---

#### 2.1.2 diff-match-patch (Text Diffing)

**Dlaczego Google's diff-match-patch?**

**Alternatywy**:
- `jsdiff`: Popularna, ale brak patch application
- `fast-myers-diff`: Szybka, ale only diff (no patch)
- `patience-diff`: Dobra dla code, ale mało popularna

**Wybrano diff-match-patch bo**:
- ✅ Battle-tested (Google Docs)
- ✅ Diff + Patch + Fuzzy Match
- ✅ Unicode support
- ✅ Text format (debuggable)
- ✅ Works in browser + Node.js

**API używane**:
```typescript
import { diff_match_patch } from 'diff-match-patch';

const dmp = new diff_match_patch();

// Create patches
const patches = dmp.patch_make(text1, text2);
const patchText = dmp.patch_toText(patches);

// Apply patches
const patches2 = dmp.patch_fromText(patchText);
const [result, success] = dmp.patch_apply(patches2, text1);
```

---

#### 2.1.3 WebCrypto API (Hashing)

**Dlaczego WebCrypto?**

**Alternatywy**:
- Node.js `crypto`: Only server-side
- `crypto-js`: Pure JS, ale wolniejsze
- `hash-wasm`: Fastest, ale wymaga WASM

**Wybrano WebCrypto bo**:
- ✅ Native (fast)
- ✅ Browser + Node.js 15+
- ✅ SHA-256 out of the box
- ✅ No dependencies
- ❌ No streaming (workaround: hash-wasm for large files)

**Implementacja**:
```typescript
async function sha256(data: Uint8Array): Promise<string> {
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  return hashHex;
}
```

**Fallback dla streaming**:
```typescript
import { createSHA256 } from 'hash-wasm';

async function sha256Stream(stream: ReadableStream): Promise<string> {
  const hasher = await createSHA256();
  const reader = stream.getReader();
  
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      hasher.update(value);
    }
    return hasher.digest('hex');
  } finally {
    reader.releaseLock();
  }
}
```

---

#### 2.1.4 uuid (ID Generation)

```typescript
import { v4 as uuidv4 } from 'uuid';

const versionId = uuidv4(); // "550e8400-e29b-41d4-a716-446655440000"
```

**Dlaczego UUID v4?**
- Collision resistant (2^122 probability)
- No server dependency
- Standard format
- 36 chars (readable)

---

### 2.2 Dev Dependencies

```json
{
  "devDependencies": {
    "typescript": "^5.3.0",
    "vitest": "^1.0.0",           // Testing
    "playwright": "^1.40.0",      // E2E browser tests
    "@types/node": "^20.0.0",
    "tsup": "^8.0.0",             // Bundler
    "prettier": "^3.1.0",
    "eslint": "^8.55.0"
  }
}
```

---

## 3. Wzorce Projektowe

### 3.1 Adapter Pattern 🔌

**Zastosowanie**: Abstrakcja systemu plików per platforma

**Problem**: 
Różne platformy mają różne API:
- Browser: IndexedDB + File API
- Node.js: fs/promises
- Tauri: @tauri-apps/api/fs

**Rozwiązanie**:
```typescript
interface FileSystemAdapter {
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  // ...
}

class BrowserAdapter implements FileSystemAdapter { /* ... */ }
class NodeAdapter implements FileSystemAdapter { /* ... */ }
class TauriAdapter implements FileSystemAdapter { /* ... */ }
```

**Korzyści**:
- Core logic platform-agnostic
- Easy testing (MockAdapter)
- Future-proof (nowe platformy bez refactoringu)

**Implementacja**: `src/adapters/`

---

### 3.2 Strategy Pattern 🎯

**Zastosowanie**: Compression policy per file type

**Problem**:
Różne typy plików wymagają różnych strategii kompresji:
- PNG: Already compressed → STORE
- JS: Text → DEFLATE level 6
- ZIP: Nested compression → STORE

**Rozwiązanie**:
```typescript
interface CompressionStrategy {
  shouldCompress(filepath: string, data: Uint8Array): boolean;
  getLevel(): number;
}

class TextCompressionStrategy implements CompressionStrategy {
  shouldCompress() { return true; }
  getLevel() { return 6; }
}

class ImageCompressionStrategy implements CompressionStrategy {
  shouldCompress() { return false; }
  getLevel() { return 0; }
}
```

**Implementacja**: `src/core/compression/`

---

### 3.3 Factory Pattern 🏭

**Zastosowanie**: Tworzenie ZIP readers/writers

**Kod**:
```typescript
class ZipFactory {
  static createWriter(adapter: FileSystemAdapter): ZipWriter {
    return new FflateZipWriter(adapter);
  }
  
  static createReader(
    source: Uint8Array | ReadableStream,
    adapter: FileSystemAdapter
  ): ZipReader {
    return new FflateZipReader(source, adapter);
  }
}
```

**Implementacja**: `src/core/zip/ZipFactory.ts`

---

### 3.4 Repository Pattern 📚

**Zastosowanie**: Abstrakcja dostępu do danych

**Struktura**:
```typescript
class BlobRepository {
  constructor(private adapter: FileSystemAdapter) {}
  
  async save(hash: string, data: Uint8Array): Promise<void> {
    await this.adapter.writeFile(`.store/blobs/${hash}`, data);
  }
  
  async load(hash: string): Promise<Uint8Array> {
    return await this.adapter.readFile(`.store/blobs/${hash}`);
  }
  
  async exists(hash: string): Promise<boolean> {
    return await this.adapter.fileExists(`.store/blobs/${hash}`);
  }
}

class DeltaRepository { /* similar */ }
class ManifestRepository { /* similar */ }
```

**Implementacja**: `src/repositories/`

---

### 3.5 Observer Pattern 👁️

**Zastosowanie**: Event emitters dla progress tracking

**Kod**:
```typescript
class JCFManager extends EventEmitter {
  async saveCheckpoint(message: string): Promise<string> {
    this.emit('checkpoint:start', { message });
    
    // ... save logic ...
    
    this.emit('checkpoint:progress', { percent: 50 });
    
    // ... more logic ...
    
    this.emit('checkpoint:complete', { versionId });
    return versionId;
  }
}

// Usage
manager.on('checkpoint:progress', (e) => {
  console.log(`Progress: ${e.percent}%`);
});
```

**Implementacja**: `src/core/JCFManager.ts`

---

### 3.6 Chain of Responsibility ⛓️

**Zastosowanie**: Error handling pipeline

**Kod**:
```typescript
class ErrorHandler {
  private next?: ErrorHandler;
  
  setNext(handler: ErrorHandler): ErrorHandler {
    this.next = handler;
    return handler;
  }
  
  handle(error: Error): void {
    if (this.canHandle(error)) {
      this.process(error);
    } else if (this.next) {
      this.next.handle(error);
    } else {
      throw error;
    }
  }
  
  protected abstract canHandle(error: Error): boolean;
  protected abstract process(error: Error): void;
}

class StorageErrorHandler extends ErrorHandler {
  canHandle(error: Error) {
    return error instanceof StorageError;
  }
  
  process(error: StorageError) {
    // Retry logic, cleanup, etc.
  }
}
```

**Implementacja**: `src/core/errors/`

---

### 3.7 Command Pattern 📝

**Zastosowanie**: Undo/Redo (future feature)

**Sketch**:
```typescript
interface Command {
  execute(): Promise<void>;
  undo(): Promise<void>;
}

class AddFileCommand implements Command {
  constructor(
    private manager: JCFManager,
    private path: string,
    private content: Uint8Array
  ) {}
  
  async execute() {
    await this.manager.addFile(this.path, this.content);
  }
  
  async undo() {
    await this.manager.deleteFile(this.path);
  }
}
```

**Implementacja**: `src/commands/` (future)

---

## 4. Struktury Danych

### 4.1 Manifest (JSON)

**Struktura**:
```typescript
interface Manifest {
  formatVersion: string;        // "1.0.0"
  metadata: ProjectMetadata;
  fileMap: Map<string, FileEntry>;      // Fast lookup
  versionHistory: Version[];            // Append-only log
  refs: Map<string, string>;            // head, branches
  renameLog: RenameEntry[];             // Tracking
}
```

**Dlaczego Map zamiast Object?**
- ✅ Iteration order preserved
- ✅ Any key type (nie tylko string)
- ✅ Size property
- ✅ Better performance dla > 100 keys

**Serializacja**:
```typescript
function serializeManifest(manifest: Manifest): string {
  return JSON.stringify({
    ...manifest,
    fileMap: Object.fromEntries(manifest.fileMap),
    refs: Object.fromEntries(manifest.refs)
  }, null, 2);
}
```

---

### 4.2 Version Graph (DAG)

**Struktura**:
```
v1 (root)
 │
 ├─ v2
 │   │
 │   ├─ v3
 │   │   │
 │   │   └─ v5 (HEAD)
 │   │
 │   └─ v4 (branch)
 │
 └─ v6 (orphaned)
```

**Reprezentacja**:
```typescript
class VersionGraph {
  private nodes = new Map<string, VersionNode>();
  
  addVersion(version: Version): void {
    const node: VersionNode = {
      id: version.id,
      parentId: version.parentId,
      children: []
    };
    
    this.nodes.set(version.id, node);
    
    if (version.parentId) {
      const parent = this.nodes.get(version.parentId);
      parent?.children.push(version.id);
    }
  }
  
  findPath(fromId: string, toId: string): string[] {
    // BFS algorithm
    const queue: Array<{id: string, path: string[]}> = [
      { id: fromId, path: [fromId] }
    ];
    
    while (queue.length > 0) {
      const { id, path } = queue.shift()!;
      
      if (id === toId) return path;
      
      const node = this.nodes.get(id);
      if (node?.parentId) {
        queue.push({
          id: node.parentId,
          path: [...path, node.parentId]
        });
      }
    }
    
    throw new Error('No path found');
  }
}
```

**Implementacja**: `src/core/VersionGraph.ts`

---

### 4.3 LRU Cache (dla Blobów)

**Dlaczego LRU?**
- Frequently accessed blobs in memory
- Automatic eviction (oldest)
- Fixed memory limit

**Implementacja**:
```typescript
class LRUCache<K, V> {
  private cache = new Map<K, V>();
  private maxSize: number;
  
  constructor(maxSize: number) {
    this.maxSize = maxSize;
  }
  
  get(key: K): V | undefined {
    const value = this.cache.get(key);
    if (value !== undefined) {
      // Move to end (most recently used)
      this.cache.delete(key);
      this.cache.set(key, value);
    }
    return value;
  }
  
  set(key: K, value: V): void {
    // Remove if exists (to reorder)
    this.cache.delete(key);
    
    // Add to end
    this.cache.set(key, value);
    
    // Evict oldest if over limit
    if (this.cache.size > this.maxSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
  }
}
```

**Usage**:
```typescript
class BlobManager {
  private cache = new LRUCache<string, Uint8Array>(100); // 100 blobs
  
  async loadBlob(hash: string): Promise<Uint8Array> {
    const cached = this.cache.get(hash);
    if (cached) return cached;
    
    const data = await this.repository.load(hash);
    this.cache.set(hash, data);
    return data;
  }
}
```

**Implementacja**: `src/utils/LRUCache.ts`

---

### 4.4 Trie (dla Path Lookup)

**Use Case**: Fast file path search

**Przykład**:
```
src/
├── index.js
├── utils.js
└── components/
    ├── Button.jsx
    └── Input.jsx
```

**Trie**:
```
root
 └─ src/
     ├─ index.js (leaf)
     ├─ utils.js (leaf)
     └─ components/
         ├─ Button.jsx (leaf)
         └─ Input.jsx (leaf)
```

**Implementacja**:
```typescript
class PathTrie {
  private root = new TrieNode();
  
  insert(path: string, data: FileEntry): void {
    const parts = path.split('/');
    let node = this.root;
    
    for (const part of parts) {
      if (!node.children.has(part)) {
        node.children.set(part, new TrieNode());
      }
      node = node.children.get(part)!;
    }
    
    node.data = data;
    node.isLeaf = true;
  }
  
  search(path: string): FileEntry | null {
    const parts = path.split('/');
    let node = this.root;
    
    for (const part of parts) {
      node = node.children.get(part);
      if (!node) return null;
    }
    
    return node.isLeaf ? node.data : null;
  }
  
  findByPrefix(prefix: string): string[] {
    // DFS to collect all paths with prefix
    // ...
  }
}
```

**Implementacja**: `src/utils/PathTrie.ts` (optional optimization)

---

## 5. Algorytmy

### 5.1 Save Checkpoint (Reverse Delta)

**Algorytm Krok po Kroku**:

```
INPUT: message (string), author (string)
OUTPUT: versionId (string)

STEP 1: Identify Changed Files
  ├─ Compare current working copy vs HEAD
  ├─ Track: added, modified, deleted
  └─ Store in changedFiles[]

STEP 2: Process Text Files
  FOR EACH text file in changedFiles:
    ├─ Read NEW content (working copy)
    ├─ Read OLD content (HEAD version)
    ├─ Compute REVERSE patch: NEW → OLD
    │   └─ Using diff-match-patch
    ├─ Serialize patch to text
    └─ Save to .store/deltas/{currentVersionId}_{pathHash}.patch

STEP 3: Process Binary Files
  FOR EACH binary file in changedFiles:
    ├─ Hash NEW content (SHA-256)
    ├─ Compare with OLD hash
    ├─ IF different:
    │   ├─ Check if blob exists (deduplication)
    │   ├─ IF NOT exists:
    │   │   └─ Save to .store/blobs/{hash}
    │   └─ Update fileMap[path].currentHash

STEP 4: Create New Version Object
  ├─ Generate new versionId (UUID v4)
  ├─ Create Version object:
  │   ├─ id: versionId
  │   ├─ timestamp: now (ISO 8601)
  │   ├─ message: user message
  │   ├─ author: user author
  │   ├─ parentId: current HEAD
  │   └─ fileStates: copy from parent + changes
  └─ Add to manifest.versionHistory[]

STEP 5: Update References
  ├─ Set manifest.refs.head = newVersionId
  └─ Update manifest.metadata.last_modified

STEP 6: Update Working Copy
  └─ /content/ already contains new state (no action needed)

STEP 7: Write Manifest
  ├─ Serialize manifest to JSON
  ├─ Compress (DEFLATE level 6)
  └─ Write to ZIP at /manifest.json

STEP 8: Finalize ZIP
  ├─ Ensure /mimetype is first (STORE)
  ├─ Close ZIP stream
  └─ Write to adapter

STEP 9: Cleanup
  ├─ Clear dirty files tracking
  ├─ Emit 'checkpoint:complete' event
  └─ RETURN versionId

ERROR HANDLING:
  ├─ IF no changes: THROW NoChangesError
  ├─ IF adapter write fails: THROW StorageError
  ├─ IF patch computation fails: Log warning, create full snapshot
  └─ IF critical error: Rollback (restore previous state)
```

**Complexity**:
- **Time**: O(F × L) gdzie F = changed files, L = avg file length
- **Space**: O(F × P) gdzie P = avg patch size (~10-20% of file)

**Optimizations**:
1. Parallel hashing (Web Workers)
2. Streaming dla large files (>50MB)
3. Incremental patching (process files as ready)

**Implementacja**: `src/core/VersionManager.ts::saveCheckpoint()`

---

### 5.2 Restore Version (Time Travel)

**Algorytm Krok po Kroku**:

```
INPUT: targetVersionId (string)
OUTPUT: void (updates working copy)

STEP 1: Validate Target Version
  ├─ Check if targetVersionId exists in manifest
  └─ IF NOT: THROW VersionNotFoundError

STEP 2: Build Version Path
  ├─ currentId = manifest.refs.head
  ├─ targetId = targetVersionId
  ├─ path = []
  ├─ WHILE currentId != targetId:
  │   ├─ path.push(currentId)
  │   ├─ version = getVersion(currentId)
  │   ├─ currentId = version.parentId
  │   └─ IF currentId == null: THROW "No path found"
  └─ path.push(targetId)
  
  EXAMPLE: HEAD=v10, target=v5
  RESULT: path = [v10, v9, v8, v7, v6, v5]

STEP 3: Reconstruct Files
  targetVersion = getVersion(targetVersionId)
  
  FOR EACH (filepath, fileState) in targetVersion.fileStates:
    IF fileState.deleted:
      └─ SKIP (will delete later)
    
    fileEntry = manifest.fileMap[filepath]
    
    IF fileEntry.type == 'text':
      ├─ content = readWorkingCopy(filepath)  // v10 content
      │
      ├─ Apply patches backwards:
      │   FOR i = 0 to path.length - 2:
      │     fromVersion = path[i]
      │     toVersion = path[i + 1]
      │     deltaPath = .store/deltas/{fromVersion}_{pathHash}.patch
      │     
      │     IF deltaPath exists:
      │       ├─ patchText = readFromZip(deltaPath)
      │       ├─ patches = parsePatch(patchText)
      │       └─ content = applyPatch(patches, content)
      │
      └─ writeToWorkingCopy(filepath, content)
    
    ELSE IF fileEntry.type == 'binary':
      ├─ hash = fileState.hash
      ├─ blobPath = .store/blobs/{hash}
      ├─ content = readFromZip(blobPath)
      └─ writeToWorkingCopy(filepath, content)

STEP 4: Delete Removed Files
  currentFiles = listWorkingCopyFiles()
  targetFiles = keys(targetVersion.fileStates)
  
  FOR EACH file in currentFiles:
    IF file NOT IN targetFiles:
      └─ deleteFromWorkingCopy(file)

STEP 5: Update Manifest
  ├─ manifest.refs.head = targetVersionId
  └─ manifest.metadata.last_modified = now()

STEP 6: Write Changes
  ├─ Save manifest
  └─ Rebuild ZIP

STEP 7: Emit Event
  ├─ Emit 'restore:complete' event
  └─ RETURN

ERROR HANDLING:
  ├─ IF patch apply fails: Use fuzzy matching
  ├─ IF blob missing: THROW BlobNotFoundError
  ├─ IF path broken: Try finding via snapshots
  └─ On critical error: ROLLBACK to previous HEAD
```

**Complexity**:
- **Time**: O(F × D) gdzie F = files, D = version distance
- **Space**: O(F × S) gdzie S = avg file size (buffering)

**Optimizations**:
1. Snapshot every N versions (skip long paths)
2. Lazy loading (load patches on demand)
3. Parallel file reconstruction

**Implementacja**: `src/core/VersionManager.ts::restoreVersion()`

---

### 5.3 Garbage Collection

**Algorytm Mark & Sweep**:

```
INPUT: options (GCOptions)
OUTPUT: GCReport

STEP 1: MARK Phase - Identify Used Blobs
  usedBlobs = new Set<string>()
  usedDeltas = new Set<string>()
  
  FOR EACH version in manifest.versionHistory:
    FOR EACH fileState in version.fileStates:
      IF fileState.hash:
        └─ usedBlobs.add(fileState.hash)
      IF fileState.contentRef starts with '.store/deltas/':
        └─ usedDeltas.add(fileState.contentRef)

STEP 2: SWEEP Phase - Find Orphans
  allBlobs = listFilesInZip('.store/blobs/')
  allDeltas = listFilesInZip('.store/deltas/')
  
  orphanedBlobs = []
  FOR EACH blob in allBlobs:
    hash = extractHashFromPath(blob)
    IF hash NOT IN usedBlobs:
      └─ orphanedBlobs.push(blob)
  
  orphanedDeltas = []
  FOR EACH delta in allDeltas:
    IF delta NOT IN usedDeltas:
      └─ orphanedDeltas.push(delta)

STEP 3: Apply Grace Period (Optional)
  IF options.gracePeriodDays > 0:
    now = Date.now()
    graceMs = options.gracePeriodDays * 24 * 60 * 60 * 1000
    
    safeToDelete = []
    FOR EACH blob in orphanedBlobs:
      lastUsed = findLastBlobUsage(blob)
      IF now - lastUsed > graceMs:
        └─ safeToDelete.push(blob)
    
    orphanedBlobs = safeToDelete

STEP 4: Calculate Space
  spaceFreed = 0
  FOR EACH blob in orphanedBlobs:
    └─ spaceFreed += getFileSize(blob)
  FOR EACH delta in orphanedDeltas:
    └─ spaceFreed += getFileSize(delta)

STEP 5: DELETE Phase
  FOR EACH blob in orphanedBlobs:
    └─ deleteFromZip(blob)
  FOR EACH delta in orphanedDeltas:
    └─ deleteFromZip(delta)

STEP 6: COMPACT Phase
  └─ repackZip()  // Remove deleted entries, compact

STEP 7: Return Report
  RETURN {
    blobsRemoved: orphanedBlobs.length,
    deltasRemoved: orphanedDeltas.length,
    spaceFreed: spaceFreed,
    duration: elapsed
  }
```

**Complexity**:
- **Time**: O(V × F + B + D) gdzie V=versions, F=files, B=blobs, D=deltas
- **Space**: O(B + D) dla tracking sets

**Implementacja**: `src/core/GarbageCollector.ts::runGC()`

---

### 5.4 File Rename Tracking (Inode System)

**Problem**: Jak zachować historię po rename?

**Rozwiązanie**: Unix-like inode system

```
STEP 1: File Creation
  ├─ Generate unique inodeId (UUID v4)
  ├─ Store in fileMap[path].inodeId
  └─ This ID NEVER changes

STEP 2: File Rename
  oldPath = 'old-name.js'
  newPath = 'new-name.js'
  
  ├─ fileEntry = manifest.fileMap[oldPath]
  ├─ inodeId = fileEntry.inodeId  // Preserve!
  │
  ├─ Create new entry:
  │   manifest.fileMap[newPath] = {
  │     ...fileEntry,
  │     inodeId: inodeId  // SAME ID!
  │   }
  │
  ├─ Delete old entry:
  │   delete manifest.fileMap[oldPath]
  │
  ├─ Log rename:
  │   manifest.renameLog.push({
  │     inodeId: inodeId,
  │     fromPath: oldPath,
  │     toPath: newPath,
  │     versionId: currentVersion,
  │     timestamp: now()
  │   })
  │
  └─ Move physical file in /content/

STEP 3: History Reconstruction
  INPUT: currentPath (string)
  OUTPUT: history (FileHistoryEntry[])
  
  fileEntry = manifest.fileMap[currentPath]
  inodeId = fileEntry.inodeId
  
  history = []
  
  FOR EACH version in versionHistory (reverse):
    ├─ Find file by inodeId in version.fileStates
    ├─ path = findPathByInode(version, inodeId)
    │
    └─ history.push({
        versionId: version.id,
        path: path,  // May be different!
        message: version.message,
        timestamp: version.timestamp,
        changeType: detectChangeType(version, inodeId)
      })
  
  RETURN history
```

**Implementacja**: `src/core/FileManager.ts::moveFile()`

---

## 6. Metody i Funkcje

### 6.1 Publiczne API (JCFManager)

#### Core Operations

```typescript
class JCFManager {
  // === Initialization ===
  
  /**
   * Inicjalizuje manager z adapterem
   * @param adapter - Adapter dla platformy (Browser/Node/Tauri)
   * @param source - Opcjonalnie: istniejący plik JCF
   */
  async init(
    adapter: FileSystemAdapter,
    source?: Uint8Array | ReadableStream
  ): Promise<void>
  
  /**
   * Zamyka manager i zwalnia zasoby
   */
  async dispose(): Promise<void>
  
  // === File Operations ===
  
  /**
   * Dodaje lub aktualizuje plik
   * @param path - Ścieżka relatywna (Unix-style)
   * @param content - Zawartość (string/Uint8Array/Blob/Stream)
   * @param metadata - Opcjonalne metadane
   */
  async addFile(
    path: string,
    content: string | Uint8Array | Blob | ReadableStream,
    metadata?: FileMetadata
  ): Promise<void>
  
  /**
   * Pobiera zawartość pliku
   * @param path - Ścieżka do pliku
   * @param versionId - Opcjonalnie: konkretna wersja
   * @returns Zawartość jako Uint8Array
   */
  async getFile(
    path: string,
    versionId?: string
  ): Promise<Uint8Array>
  
  /**
   * Pobiera plik jako stream (dla dużych plików)
   */
  getFileStream(
    path: string,
    versionId?: string
  ): ReadableStream
  
  /**
   * Usuwa plik (soft delete - zachowane w historii)
   */
  async deleteFile(path: string): Promise<void>
  
  /**
   * Zmienia nazwę/przenosi plik (zachowuje historię)
   */
  async moveFile(
    oldPath: string,
    newPath: string
  ): Promise<void>
  
  /**
   * Lista wszystkich plików
   */
  async listFiles(versionId?: string): Promise<FileInfo[]>
  
  /**
   * Sprawdza czy plik istnieje
   */
  async fileExists(
    path: string,
    versionId?: string
  ): Promise<boolean>
  
  // === Versioning ===
  
  /**
   * Tworzy nowy checkpoint (commit)
   * @param message - Opis zmian
   * @param author - Opcjonalnie: nadpisuje domyślnego autora
   * @returns ID nowej wersji
   */
  async saveCheckpoint(
    message: string,
    author?: string
  ): Promise<string>
  
  /**
   * Przywraca projekt do określonej wersji
   * UWAGA: Destructive operation!
   */
  async restoreVersion(versionId: string): Promise<void>
  
  /**
   * Zwraca pełną historię wersji
   */
  getVersionHistory(): Version[]
  
  /**
   * Zwraca historię konkretnego pliku
   */
  async getFileHistory(
    filePath: string
  ): Promise<FileHistoryEntry[]>
  
  /**
   * Porównuje dwie wersje
   */
  async compareVersions(
    versionId1: string,
    versionId2: string
  ): Promise<VersionDiff>
  
  /**
   * Zwraca info o konkretnej wersji
   */
  getVersion(versionId: string): Version | null
  
  // === Maintenance ===
  
  /**
   * Uruchamia garbage collection
   */
  async runGC(options?: GCOptions): Promise<GCReport>
  
  /**
   * Weryfikuje integralność pliku JCF
   */
  async verify(): Promise<VerificationReport>
  
  /**
   * Zwraca statystyki projektu
   */
  async getStats(): Promise<ProjectStats>
  
  // === Export/Import ===
  
  /**
   * Eksportuje projekt jako stream
   */
  async export(): Promise<ReadableStream>
  
  /**
   * Eksportuje snapshot wersji (bez historii)
   */
  async exportSnapshot(
    versionId?: string,
    format?: 'zip' | 'tar'
  ): Promise<ReadableStream>
  
  /**
   * Importuje projekt z danych
   */
  async import(
    data: Uint8Array | ReadableStream
  ): Promise<void>
  
  // === Utilities ===
  
  /**
   * Pobiera manifest
   */
  getManifest(): Manifest
  
  /**
   * Aktualizuje konfigurację
   */
  updateConfig(config: Partial<JCFConfig>): void
  
  /**
   * Pobiera bieżący HEAD
   */
  getCurrentVersion(): Version
}
```

---

### 6.2 Internal Classes

#### VersionManager

```typescript
class VersionManager {
  constructor(
    private manifest: Manifest,
    private adapter: FileSystemAdapter,
    private deltaManager: DeltaManager,
    private blobManager: BlobManager
  ) {}
  
  async saveCheckpoint(
    message: string,
    author: string,
    changedFiles: FileChange[]
  ): Promise<string>
  
  async restoreVersion(targetVersionId: string): Promise<void>
  
  buildVersionPath(fromId: string, toId: string): string[]
  
  getVersion(id: string): Version | null
  
  findCommonAncestor(id1: string, id2: string): string | null
}
```

#### DeltaManager

```typescript
class DeltaManager {
  constructor(
    private adapter: FileSystemAdapter,
    private workers: WorkerPool
  ) {}
  
  async computeDelta(
    newText: string,
    oldText: string
  ): Promise<string>
  
  async applyDelta(
    currentText: string,
    patchText: string
  ): Promise<string>
  
  async saveDelta(
    versionId: string,
    filePath: string,
    patchText: string
  ): Promise<void>
  
  async loadDelta(
    versionId: string,
    filePath: string
  ): Promise<string>
}
```

#### BlobManager

```typescript
class BlobManager {
  constructor(
    private adapter: FileSystemAdapter,
    private workers: WorkerPool,
    private cache: LRUCache<string, Uint8Array>
  ) {}
  
  async saveBlob(
    content: Uint8Array
  ): Promise<string>  // Returns hash
  
  async loadBlob(hash: string): Promise<Uint8Array>
  
  async blobExists(hash: string): Promise<boolean>
  
  async hashContent(
    content: Uint8Array
  ): Promise<string>
  
  async listBlobs(): Promise<string[]>
}
```

#### FileManager

```typescript
class FileManager {
  constructor(
    private manifest: Manifest,
    private adapter: FileSystemAdapter
  ) {}
  
  async addFile(
    path: string,
    content: string | Uint8Array | Blob | ReadableStream,
    metadata?: FileMetadata
  ): Promise<void>
  
  async getFile(
    path: string,
    versionId?: string
  ): Promise<Uint8Array>
  
  async deleteFile(path: string): Promise<void>
  
  async moveFile(
    oldPath: string,
    newPath: string
  ): Promise<void>
  
  async listFiles(versionId?: string): Promise<FileInfo[]>
  
  detectFileType(
    path: string,
    content: Uint8Array
  ): 'text' | 'binary'
}
```

#### GarbageCollector

```typescript
class GarbageCollector {
  constructor(
    private manifest: Manifest,
    private adapter: FileSystemAdapter
  ) {}
  
  async runGC(options?: GCOptions): Promise<GCReport>
  
  private async markPhase(): Promise<{
    usedBlobs: Set<string>;
    usedDeltas: Set<string>;
  }>
  
  private async sweepPhase(
    usedBlobs: Set<string>,
    usedDeltas: Set<string>
  ): Promise<{
    orphanedBlobs: string[];
    orphanedDeltas: string[];
  }>
  
  private async deletePhase(
    orphaned: string[]
  ): Promise<void>
  
  private async compactPhase(): Promise<void>
}
```

---

### 6.3 Worker Pool

```typescript
class WorkerPool {
  private hashWorkers: Worker[];
  private diffWorkers: Worker[];
  private availableWorkers: Worker[];
  private taskQueue: Task[];
  
  constructor(workerCount: number) {
    // Initialize workers
  }
  
  async hash(data: Uint8Array): Promise<string>
  
  async diff(
    text1: string,
    text2: string
  ): Promise<string>
  
  async compress(
    data: Uint8Array,
    level: number
  ): Promise<Uint8Array>
  
  terminate(): void
}
```

**Worker Scripts**:
```typescript
// hash-worker.ts
self.onmessage = async (e) => {
  const { type, data, id } = e.data;
  
  if (type === 'hash') {
    const hash = await sha256(data);
    self.postMessage({ id, result: hash });
  }
};

// diff-worker.ts
self.onmessage = (e) => {
  const { type, oldText, newText, id } = e.data;
  
  if (type === 'diff') {
    const dmp = new diff_match_patch();
    const patches = dmp.patch_make(newText, oldText);
    const patchText = dmp.patch_toText(patches);
    
    self.postMessage({ id, result: patchText });
  }
};
```

---

## 7. Pipeline Implementacji

### Phase 1: Foundation (Tydzień 1)

**Zadania**:
1. ✅ Setup projektu (TypeScript, tsup, vitest)
2. ✅ Struktury danych (Manifest, Version, FileEntry)
3. ✅ Adapter interfejs + MemoryAdapter
4. ✅ ZIP utils (wrapper dla fflate)
5. ✅ Basic file operations (add, get, list)

**Deliverable**: Można dodawać/pobierać pliki w pamięci

---

### Phase 2: Versioning Core (Tydzień 2)

**Zadania**:
1. ✅ DeltaManager (diff-match-patch integration)
2. ✅ BlobManager (SHA-256, CAS)
3. ✅ VersionManager (save checkpoint - podstawowa wersja)
4. ✅ Restore version (bez optimization)
5. ✅ Tests jednostkowe

**Deliverable**: Działający save/restore z reverse delta

---

### Phase 3: Platform Adapters (Tydzień 3)

**Zadania**:
1. ✅ BrowserAdapter (IndexedDB)
2. ✅ NodeAdapter (fs/promises)
3. ✅ TauriAdapter (@tauri-apps/api/fs)
4. ✅ Streaming support
5. ✅ Integration tests per platform

**Deliverable**: Działa na Browser, Node.js, Tauri

---

### Phase 4: Performance (Tydzień 4)

**Zadania**:
1. ✅ Worker Pool implementation
2. ✅ LRU Cache dla blobów
3. ✅ Streaming dla large files (>50MB)
4. ✅ Parallel operations
5. ✅ Benchmarking

**Deliverable**: Obsługa plików >500MB, 5x faster

---

### Phase 5: Advanced Features (Tydzień 5)

**Zadania**:
1. ✅ Garbage Collection
2. ✅ File rename tracking (inode system)
3. ✅ Verification system
4. ✅ Export/Import
5. ✅ Event emitters

**Deliverable**: Production-ready library

---

### Phase 6: Polish & Release (Tydzień 6)

**Zadania**:
1. ✅ Dokumentacja API (JSDoc)
2. ✅ E2E tests
3. ✅ Error handling refinement
4. ✅ Performance profiling
5. ✅ npm publish

**Deliverable**: v1.0.0 release!

---

## 8. Decyzje Architektoniczne

### 8.1 Dlaczego Reverse Delta zamiast Forward?

**Analiza**:

| Metric | Forward Delta | Reverse Delta | Waga |
|--------|---------------|---------------|------|
| HEAD access speed | O(n) | O(1) | 50% |
| Old version access | O(1) | O(n) | 10% |
| Storage size | Equivalent | Equivalent | 20% |
| Implementation complexity | Medium | Medium | 10% |
| Corruption resilience | Low | High | 10% |

**Score**:
- Forward: 3.5/5
- Reverse: 4.6/5 → **Reverse wins**

**Uzasadnienie**:
95% czasu użytkownik pracuje z HEAD. Optymalizujemy dla common case.

---

### 8.2 Dlaczego ZIP zamiast Custom Binary?

**Pros**:
- ✅ Universal (każdy OS rozumie)
- ✅ Tooling (unzip, 7-zip, etc.)
- ✅ Recovery (standard tools mogą odzyskać)
- ✅ Compression built-in

**Cons**:
- ⚠️ Overhead (~22 bytes per file header)
- ⚠️ No streaming modification (must repack)

**Decyzja**: ZIP wins dzięki kompatybilności i tooling

---

### 8.3 Dlaczego SHA-256 zamiast MD5?

**Bezpieczeństwo**:
- MD5: Broken (collisions found)
- SHA-1: Deprecated (SHAttered attack)
- SHA-256: Secure

**Performance**:
- MD5: ~500 MB/s
- SHA-256: ~300 MB/s (native crypto)
- SHA-256: ~800 MB/s (WASM)

**Decyzja**: SHA-256 dla bezpieczeństwa + native API

---

### 8.4 Dlaczego Map zamiast Object dla fileMap?

**Performance** (1000 keys):
- Object: ~0.5ms lookup
- Map: ~0.1ms lookup

**Features**:
- Object: Only string keys
- Map: Any key, iteration order, size property

**Memory**:
- Equivalent dla small (<100 keys)
- Map 10-20% overhead dla large

**Decyzja**: Map dla performance + features

---

### 8.5 Dlaczego Web Workers zamiast Main Thread?

**Benchmark** (100MB file hash):
- Main thread: 2.5s (UI freeze)
- Web Worker: 2.5s (non-blocking)

**User Experience**:
- Main thread: App hangs
- Worker: Smooth UI + progress bar

**Decyzja**: Workers dla UX (critical!)

---

### 8.6 Dlaczego LRU Cache zamiast LFU?

**LRU** (Least Recently Used):
- Simple implementation
- Good for temporal locality
- O(1) get/set

**LFU** (Least Frequently Used):
- Complex (need frequency counter)
- Good for access patterns
- O(log n) operations

**Use case**: Często edytowane pliki = temporal locality

**Decyzja**: LRU (simplicity + good enough)

---

### 8.7 Dlaczego Adapter Pattern zamiast Conditional Logic?

**Without Adapter**:
```typescript
async readFile(path: string) {
  if (typeof window !== 'undefined') {
    // Browser code
  } else if (typeof process !== 'undefined') {
    // Node code
  } else if (typeof __TAURI__ !== 'undefined') {
    // Tauri code
  }
}
```
Problem: Unmaintainable spaghetti!

**With Adapter**:
```typescript
async readFile(path: string) {
  return this.adapter.readFile(path);
}
```
Win: Clean, testable, extensible

**Decyzja**: Adapter (design principle)

---

## 9. Podsumowanie

### 9.1 Co zostało określone?

- ✅ **Stack**: fflate, diff-match-patch, WebCrypto, uuid
- ✅ **Wzorce**: 7 design patterns (Adapter, Strategy, Factory, Repository, Observer, Chain of Responsibility, Command)
- ✅ **Struktury danych**: Manifest, Version Graph (DAG), LRU Cache, Trie
- ✅ **Algorytmy**: Save Checkpoint, Restore Version, GC (Mark & Sweep), Rename Tracking
- ✅ **Metody**: 30+ public methods, 15+ internal classes
- ✅ **Pipeline**: 6-week implementation plan
- ✅ **Decyzje**: 7 architectural decisions z uzasadnieniem

### 9.2 Ready for Implementation?

**TAK!** Ten dokument zawiera wszystko co potrzebne do rozpoczęcia kodowania:

1. ✅ Dokładna specyfikacja każdej metody
2. ✅ Algorytmy krok po kroku (pseudo-kod)
3. ✅ Struktury danych z uzasadnieniem
4. ✅ Wzorce projektowe z przykładami
5. ✅ Harmonogram implementacji
6. ✅ Analiza decyzji architektonicznych

### 9.3 Następne Kroki

1. **Setup projektu**:
   ```bash
   mkdir jcf-manager
   cd jcf-manager
   npm init -y
   npm install -D typescript vitest tsup
   npm install fflate diff-match-patch uuid
   ```

2. **Struktura folderów**:
   ```
   src/
   ├── core/
   │   ├── JCFManager.ts
   │   ├── VersionManager.ts
   │   ├── DeltaManager.ts
   │   ├── BlobManager.ts
   │   ├── FileManager.ts
   │   └── GarbageCollector.ts
   ├── adapters/
   │   ├── FileSystemAdapter.ts
   │   ├── BrowserAdapter.ts
   │   ├── NodeAdapter.ts
   │   └── TauriAdapter.ts
   ├── workers/
   │   ├── hash-worker.ts
   │   └── diff-worker.ts
   ├── types/
   │   └── index.ts
   └── index.ts
   ```

3. **Rozpocznij od Phase 1**: Foundation (tydzień 1)

---

## 10. Apendix: Pytania i Odpowiedzi

### Q: Czy obsługujemy branching?

**A**: Nie w v1.0. Linear history only. Branching w v2.0.

### Q: Jak duże projekty obsługujemy?

**A**: 
- Browser: do 500MB (storage limit)
- Node.js: Unlimited (disk dependent)
- Files: Max 500MB per file (streaming)

### Q: Czy będzie CLI?

**A**: Nie w core library. CLI jako osobny package (`jcf-cli`).

### Q: Merge conflicts?

**A**: Nie w v1.0 (linear history). Conflict resolution w v2.0.

### Q: Encryption?

**A**: Nie built-in. User może encrypt przed addFile(). Encryption layer w v2.0.

### Q: Real-time collaboration?

**A**: Nie. To jest offline-first format. Sync można dodać zewnętrznie (WebRTC/WebSocket).

---

**Document End**

---

**Autorzy**: Zespół Architektury JCF  
**Ostatnia aktualizacja**: 2025-12-18  
**Wersja**: 1.0.0  
**Status**: ✅ READY FOR IMPLEMENTATION

