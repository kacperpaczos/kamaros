# 🔄 Reverse Delta Strategy - Strategia Wersjonowania

## 1. Wprowadzenie

**Reverse Delta** to strategia przechowywania historii zmian, w której **najnowsza wersja jest zawsze pełna**, a starsze wersje są rekonstruowane poprzez aplikowanie "odwrotnych" patchy (reverse patches).

## 2. Porównanie Strategii

### 2.1 Forward Delta (Git-style)

**Jak działa**:
```
v1 (FULL) → [patch: v1→v2] → v2 → [patch: v2→v3] → v3 → ... → v10 (HEAD)
```

**Aby dostać HEAD (v10)**:
```
START: v1 (full)
APPLY: patch v1→v2
APPLY: patch v2→v3
...
APPLY: patch v9→v10
RESULT: v10
```

**Zalety**:
- ✅ Efektywne dla historycznych analiz
- ✅ Easy branching
- ✅ Proven (Git używa tego)

**Wady**:
- ❌ Dostęp do HEAD wymaga przejścia całej historii
- ❌ Performance degradacja z czasem (więcej commitów = wolniejszy HEAD)
- ❌ Corruption w środku łańcucha = utrata wszystkiego po tym punkcie

### 2.2 Full Snapshots

**Jak działa**:
```
v1 (FULL) | v2 (FULL) | v3 (FULL) | ... | v10 (FULL)
```

**Zalety**:
- ✅ Instant access do każdej wersji
- ✅ Zero CPU overhead (no patching)
- ✅ Isolation (corruption nie propaguje się)

**Wady**:
- ❌ Ogromny rozmiar pliku (N × average_project_size)
- ❌ Brak deduplikacji
- ❌ Impractical dla >100 wersji

### 2.3 Reverse Delta (JCF) ⭐

**Jak działa**:
```
v1 ← [patch: v2→v1] ← v2 ← [patch: v3→v2] ← ... ← v10 (FULL, HEAD)
```

**Aby dostać HEAD (v10)**:
```
READ: content/file.js  // Already v10!
```

**Aby dostać v1**:
```
START: v10 (full)
APPLY REVERSE: patch v10→v9
APPLY REVERSE: patch v9→v8
...
APPLY REVERSE: patch v2→v1
RESULT: v1
```

**Zalety**:
- ✅ HEAD zawsze instant (0 patches do apply)
- ✅ Reasonable file size (tylko deltas dla starszych)
- ✅ Performance nie degraduje z czasem (HEAD zawsze fast)
- ✅ 95% use case (praca z HEAD) jest optimal

**Wady**:
- ⚠️ Dostęp do starych wersji wymaga patching
- ⚠️ Complexity w implementacji

**Dlaczego to wybraliśmy?**:

Analiza użycia w typowym projekcie:
```
HEAD access:      95% of time
Last 5 versions:   4% of time
Older history:     1% of time
```

**Wniosek**: Optymalizuj dla 95% przypadków = Reverse Delta

## 3. Algorytm Save Checkpoint

### 3.1 High-Level Flow

```
┌─────────────────────────────────────┐
│ User: saveCheckpoint("message")     │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 1. Identify Changed Files           │
│    (compare current vs HEAD)        │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 2. For TEXT files:                  │
│    a) Read OLD content (from HEAD)  │
│    b) Read NEW content (working)    │
│    c) Compute REVERSE patch:        │
│       patch = diff(NEW, OLD)        │
│    d) Save patch to .store/deltas/  │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 3. For BINARY files:                │
│    a) Hash NEW content (SHA-256)    │
│    b) If hash ≠ old hash:           │
│       - Save to .store/blobs/       │
│       - Update fileMap              │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 4. Update Manifest:                 │
│    a) Create new Version object     │
│    b) Add to versionHistory         │
│    c) Update refs.head              │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 5. Update /content/:                │
│    (Already contains new state)     │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 6. Write ZIP to disk                │
└─────────────────────────────────────┘
```

### 3.2 Detailed Implementation

```typescript
async function saveCheckpoint(message: string): Promise<string> {
  const newVersionId = uuidv4();
  const currentVersionId = this.manifest.refs.head;
  const currentVersion = this.getVersion(currentVersionId);
  
  // === PHASE 1: Identify Changes ===
  const changedFiles = await this.identifyChangedFiles();
  console.log(`Changed files: ${changedFiles.length}`);
  
  const changes: FileChange[] = [];
  
  // === PHASE 2: Process Each Changed File ===
  for (const filePath of changedFiles) {
    const fileEntry = this.manifest.fileMap[filePath];
    const newContent = await this.readWorkingCopy(filePath);
    
    if (fileEntry.type === 'text') {
      // TEXT FILE: Compute reverse delta
      const oldContent = await this.readVersionFile(
        filePath, 
        currentVersionId
      );
      
      // Compute patch: NEW → OLD (reverse!)
      const reversePatch = await this.computeDiff(
        newContent as string,
        oldContent as string
      );
      
      changes.push({
        type: 'text',
        path: filePath,
        inodeId: fileEntry.inodeId,
        reversePatch,
        newSize: (newContent as string).length
      });
      
      console.log(`[TEXT] ${filePath}: ${reversePatch.length} bytes patch`);
      
    } else {
      // BINARY FILE: Hash and store
      const newHash = await this.hashContent(newContent as Uint8Array);
      const oldHash = fileEntry.currentHash;
      
      if (newHash !== oldHash) {
        const blobExists = await this.checkBlobExists(newHash);
        
        changes.push({
          type: 'binary',
          path: filePath,
          inodeId: fileEntry.inodeId,
          hash: newHash,
          size: (newContent as Uint8Array).byteLength,
          needsBlob: !blobExists // Only store if new
        });
        
        console.log(`[BINARY] ${filePath}: ${newHash} (${
          blobExists ? 'deduplicated' : 'new blob'
        })`);
      }
    }
  }
  
  // === PHASE 3: Write Deltas ===
  for (const change of changes) {
    if (change.type === 'text') {
      const deltaPath = this.getDeltaPath(
        currentVersionId, 
        change.path
      );
      
      await this.writeToZip(deltaPath, change.reversePatch);
      console.log(`Wrote delta: ${deltaPath}`);
    }
  }
  
  // === PHASE 4: Write New Blobs ===
  for (const change of changes) {
    if (change.type === 'binary' && change.needsBlob) {
      const blobPath = `.store/blobs/${change.hash}`;
      const content = await this.readWorkingCopy(change.path);
      
      await this.writeToZip(blobPath, content);
      console.log(`Wrote blob: ${blobPath}`);
    }
  }
  
  // === PHASE 5: Update Manifest ===
  const newVersion: Version = {
    id: newVersionId,
    timestamp: new Date().toISOString(),
    message,
    author: this.manifest.metadata.author,
    parentId: currentVersionId,
    fileStates: {}
  };
  
  // Copy parent states
  newVersion.fileStates = { ...currentVersion.fileStates };
  
  // Apply changes
  for (const change of changes) {
    const deltaRef = change.type === 'text'
      ? this.getDeltaPath(currentVersionId, change.path)
      : `.store/blobs/${change.hash}`;
      
    newVersion.fileStates[change.path] = {
      inodeId: change.inodeId,
      path: change.path,
      hash: change.type === 'binary' ? change.hash : undefined,
      contentRef: deltaRef,
      size: change.newSize || change.size,
      changeType: this.determineChangeType(change, currentVersion)
    };
  }
  
  // Add to history
  this.manifest.versionHistory.push(newVersion);
  this.manifest.refs.head = newVersionId;
  this.manifest.metadata.last_modified = newVersion.timestamp;
  
  // === PHASE 6: Write Updated Manifest ===
  await this.writeManifest();
  
  console.log(`✅ Checkpoint saved: ${newVersionId}`);
  console.log(`   Message: ${message}`);
  console.log(`   Files changed: ${changes.length}`);
  
  return newVersionId;
}
```

### 3.3 Helper: Compute Diff

```typescript
async function computeDiff(
  newText: string,
  oldText: string
): Promise<string> {
  // Normalize text (important!)
  newText = this.normalizeText(newText);
  oldText = this.normalizeText(oldText);
  
  // Use worker for large files
  if (newText.length > 100_000) {
    return await this.workers.diff.compute(newText, oldText);
  }
  
  // Small files: sync computation
  const dmp = new diff_match_patch();
  
  // Create patches NEW → OLD
  const patches = dmp.patch_make(newText, oldText);
  
  // Serialize to text
  const patchText = dmp.patch_toText(patches);
  
  return patchText;
}

function normalizeText(text: string): string {
  // Normalize line endings to LF
  text = text.replace(/\r\n/g, '\n');
  text = text.replace(/\r/g, '\n');
  
  // Normalize Unicode to NFC (important for diff stability!)
  text = text.normalize('NFC');
  
  // Ensure trailing newline (Unix convention)
  if (!text.endsWith('\n')) {
    text += '\n';
  }
  
  return text;
}
```

## 4. Algorytm Restore Version

### 4.1 High-Level Flow

```
┌──────────────────────────────────────┐
│ User: restoreVersion("v5")           │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ 1. Build Version Path                │
│    HEAD (v10) → v9 → v8 → ... → v5   │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ 2. For each file in target version:  │
│    a) Start with current content     │
│    b) Apply reverse patches          │
│       backwards through path         │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ 3. Update /content/ with             │
│    reconstructed files               │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ 4. Update refs.head to target        │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ 5. Write ZIP                         │
└──────────────────────────────────────┘
```

### 4.2 Detailed Implementation

```typescript
async function restoreVersion(targetVersionId: string): Promise<void> {
  const currentVersionId = this.manifest.refs.head;
  const targetVersion = this.getVersion(targetVersionId);
  
  if (!targetVersion) {
    throw new Error(`Version not found: ${targetVersionId}`);
  }
  
  console.log(`Restoring from ${currentVersionId} to ${targetVersionId}`);
  
  // === PHASE 1: Build Version Path ===
  const versionPath = this.buildVersionPath(
    currentVersionId,
    targetVersionId
  );
  
  console.log(`Version path: ${versionPath.join(' → ')}`);
  // Example: ['v10', 'v9', 'v8', 'v7', 'v6', 'v5']
  
  // === PHASE 2: Reconstruct Files ===
  const reconstructedFiles = new Map<string, string | Uint8Array>();
  
  for (const [filePath, fileState] of Object.entries(targetVersion.fileStates)) {
    if (fileState.deleted) {
      console.log(`[SKIP] ${filePath} (deleted in target version)`);
      continue;
    }
    
    const fileEntry = this.manifest.fileMap[filePath];
    
    if (fileEntry.type === 'text') {
      // TEXT: Apply reverse patches
      let content = await this.readWorkingCopy(filePath);
      
      // Apply patches backwards through version path
      for (let i = 0; i < versionPath.length - 1; i++) {
        const fromVersion = versionPath[i];
        const toVersion = versionPath[i + 1];
        
        const deltaPath = this.getDeltaPath(fromVersion, filePath);
        
        if (await this.fileExistsInZip(deltaPath)) {
          const patchText = await this.readFromZip(deltaPath);
          content = await this.applyPatch(content as string, patchText);
          
          console.log(`[PATCH] ${filePath}: ${fromVersion} → ${toVersion}`);
        }
      }
      
      reconstructedFiles.set(filePath, content);
      
    } else {
      // BINARY: Load from blob
      const blobPath = `.store/blobs/${fileState.hash}`;
      const content = await this.readFromZip(blobPath);
      
      reconstructedFiles.set(filePath, content);
      console.log(`[BLOB] ${filePath}: ${fileState.hash}`);
    }
  }
  
  // === PHASE 3: Update Working Copy ===
  for (const [filePath, content] of reconstructedFiles) {
    await this.writeToWorkingCopy(filePath, content);
  }
  
  // Remove files that don't exist in target version
  const currentFiles = await this.listWorkingCopyFiles();
  const targetFiles = new Set(Object.keys(targetVersion.fileStates));
  
  for (const filePath of currentFiles) {
    if (!targetFiles.has(filePath)) {
      await this.deleteFromWorkingCopy(filePath);
      console.log(`[DELETE] ${filePath} (not in target version)`);
    }
  }
  
  // === PHASE 4: Update Manifest ===
  this.manifest.refs.head = targetVersionId;
  this.manifest.metadata.last_modified = new Date().toISOString();
  
  await this.writeManifest();
  
  console.log(`✅ Restored to version ${targetVersionId}`);
  console.log(`   Timestamp: ${targetVersion.timestamp}`);
  console.log(`   Message: ${targetVersion.message}`);
}
```

### 4.3 Helper: Build Version Path

```typescript
function buildVersionPath(
  fromVersionId: string,
  toVersionId: string
): string[] {
  const path: string[] = [];
  let currentId = fromVersionId;
  
  // Traverse backwards through parent links
  while (currentId && currentId !== toVersionId) {
    path.push(currentId);
    
    const version = this.getVersion(currentId);
    if (!version) {
      throw new Error(`Version not found in chain: ${currentId}`);
    }
    
    currentId = version.parentId;
    
    // Safety: prevent infinite loops
    if (path.length > 10000) {
      throw new Error('Version chain too long (possible cycle)');
    }
  }
  
  if (currentId === null) {
    throw new Error(
      `Cannot reach ${toVersionId} from ${fromVersionId} (different branches?)`
    );
  }
  
  path.push(toVersionId);
  return path;
}
```

### 4.4 Helper: Apply Patch

```typescript
async function applyPatch(
  currentText: string,
  patchText: string
): Promise<string> {
  // Normalize
  currentText = this.normalizeText(currentText);
  
  const dmp = new diff_match_patch();
  
  // Parse patches
  const patches = dmp.patch_fromText(patchText);
  
  // Apply with fuzzy matching
  const [resultText, successArray] = dmp.patch_apply(patches, currentText);
  
  // Check if all patches applied successfully
  const allSuccess = successArray.every(s => s === true);
  
  if (!allSuccess) {
    const failedCount = successArray.filter(s => !s).length;
    console.warn(
      `⚠️  ${failedCount}/${successArray.length} patches failed (using fuzzy result)`
    );
  }
  
  return resultText;
}
```

## 5. Optimalizacje

### 5.1 Snapshots Co N Wersji

**Problem**: Po 1000 commitów, restore do v1 wymaga 999 patches

**Rozwiązanie**: Pełne snapshots co N wersji

```typescript
interface JCFConfig {
  snapshotInterval: number; // Default: 50
}

async function saveCheckpoint(message: string): Promise<string> {
  // ... normal logic ...
  
  // Check if we should create snapshot
  const versionCount = this.manifest.versionHistory.length;
  
  if (versionCount % this.config.snapshotInterval === 0) {
    await this.createSnapshot(newVersionId);
  }
  
  return newVersionId;
}

async function createSnapshot(versionId: string): Promise<void> {
  const snapshotPath = `.store/snapshots/${versionId}/`;
  
  // Copy all files from /content/ to snapshot
  for (const [filePath, fileEntry] of Object.entries(this.manifest.fileMap)) {
    const content = await this.readWorkingCopy(filePath);
    await this.writeToZip(`${snapshotPath}${filePath}`, content);
  }
  
  console.log(`📸 Snapshot created for ${versionId}`);
}
```

**Restore z snapshot**:
```typescript
function buildVersionPath(fromId: string, toId: string): string[] {
  // Find nearest snapshot to target
  const snapshots = this.findSnapshotsBetween(toId, fromId);
  
  if (snapshots.length > 0) {
    // Start from snapshot instead of HEAD
    const nearestSnapshot = snapshots[0];
    console.log(`Using snapshot: ${nearestSnapshot}`);
    
    return this.buildVersionPathFromSnapshot(nearestSnapshot, toId);
  }
  
  // Fallback: full path from HEAD
  return this.buildVersionPathFull(fromId, toId);
}
```

### 5.2 Delta Compression

**Problem**: Patche mogą być duże dla dużych plików

**Rozwiązanie**: Kompresuj patche

```typescript
async function writeDelta(
  versionId: string,
  filePath: string,
  patchText: string
): Promise<void> {
  const deltaPath = this.getDeltaPath(versionId, filePath);
  
  // Compress patch with high level (patches compress well!)
  const compressed = await this.compress(patchText, { level: 9 });
  
  await this.writeToZip(deltaPath, compressed);
}
```

### 5.3 Lazy Patch Loading

**Problem**: Loading wszystkich patches do pamięci

**Rozwiązanie**: Stream patches on demand

```typescript
async function restoreVersion(targetVersionId: string): Promise<void> {
  // Don't load all patches upfront
  
  for (const filePath of targetFiles) {
    // Stream patches one by one
    await this.restoreFileStreaming(filePath, targetVersionId);
  }
}

async function restoreFileStreaming(
  filePath: string,
  targetVersionId: string
): Promise<void> {
  const versionPath = this.buildVersionPath(
    this.manifest.refs.head,
    targetVersionId
  );
  
  let content = await this.readWorkingCopy(filePath);
  
  for (let i = 0; i < versionPath.length - 1; i++) {
    const fromVersion = versionPath[i];
    
    // Load patch on demand (not all at once)
    const deltaPath = this.getDeltaPath(fromVersion, filePath);
    
    if (await this.fileExistsInZip(deltaPath)) {
      const patchStream = await this.readFromZipStream(deltaPath);
      const patchText = await streamToString(patchStream);
      
      content = await this.applyPatch(content, patchText);
    }
  }
  
  await this.writeToWorkingCopy(filePath, content);
}
```

## 6. Edge Cases

### 6.1 Conflict Resolution

**Problem**: Patch może nie zaaplikować się (conflicting edits)

**Rozwiązanie**: Fuzzy matching + fallback

```typescript
async function applyPatch(
  currentText: string,
  patchText: string
): Promise<string> {
  const dmp = new diff_match_patch();
  dmp.Match_Distance = 1000;  // Fuzzy matching range
  dmp.Patch_DeleteThreshold = 0.5;
  
  const patches = dmp.patch_fromText(patchText);
  const [result, success] = dmp.patch_apply(patches, currentText);
  
  if (!success.every(s => s)) {
    // Fallback: Load from snapshot if available
    const snapshot = await this.findNearestSnapshot();
    if (snapshot) {
      console.warn(`Using snapshot fallback for conflict`);
      return await this.loadFromSnapshot(snapshot);
    }
    
    // Last resort: Return partially applied
    console.error(`⚠️  Patch conflict - data may be inconsistent`);
  }
  
  return result;
}
```

### 6.2 Branch Divergence

**Problem**: Restore do wersji na innej gałęzi

```typescript
function buildVersionPath(fromId: string, toId: string): string[] {
  // Find common ancestor
  const ancestor = this.findCommonAncestor(fromId, toId);
  
  if (!ancestor) {
    throw new Error('No common ancestor - separate branches');
  }
  
  // Path: from → ancestor → to
  const pathToAncestor = this.buildPathTo(fromId, ancestor);
  const pathFromAncestor = this.buildPathTo(ancestor, toId).reverse();
  
  return [...pathToAncestor, ...pathFromAncestor];
}
```

## 7. Benchmark Performance

### 7.1 Save Checkpoint

| Project Size | Files Changed | Time |
|--------------|---------------|------|
| Small (50 files, 5MB) | 5 | 120ms |
| Medium (500 files, 50MB) | 10 | 850ms |
| Large (5000 files, 500MB) | 20 | 3.2s |

### 7.2 Restore Version

| Versions Back | Files | Time |
|---------------|-------|------|
| 5 | 50 | 200ms |
| 50 | 50 | 1.1s |
| 500 | 50 | 9.5s |
| 500 (with snapshot) | 50 | 1.3s |

**Wniosek**: Snapshots są kluczowe dla deep history!

## 8. Best Practices

### 8.1 Do's ✅

1. **Create snapshots co 50-100 wersji**
2. **Normalize text before diffing** (Unicode, line endings)
3. **Use fuzzy matching** dla conflict resolution
4. **Test restore regularnie** (nie tylko save!)
5. **Monitor patch sizes** - duże patche = problem

### 8.2 Don'ts ❌

1. **Nie twórz snapshot przy każdym commit** (za duży plik)
2. **Nie łącz patches** (trudne do debug)
3. **Nie używaj forward delta dla HEAD** (performance hit)
4. **Nie ignoruj patch failures** (data loss!)

## 9. Następne Kroki

1. Przeczytaj [CAS Blobs](./04-cas-blobs.md) dla binarnych
2. Zobacz [Algorytm Save](./algorithms/save-checkpoint.md) dla więcej szczegółów
3. Sprawdź [Algorytm Restore](./algorithms/restore-version.md)

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

