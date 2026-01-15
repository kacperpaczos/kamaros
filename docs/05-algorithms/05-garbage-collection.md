# Garbage Collection

> **Szczegółowy algorytm Mark & Sweep dla czyszczenia nieużywanych blobów i delt**

[← Back: Patch Application](04-patch-application.md) | [Next: File Rename Tracking →](06-file-rename-tracking.md)

---

## Overview

Algorytm **Garbage Collection (GC)** odpowiada za identyfikację i usuwanie **orphaned** blobów i delt - czyli plików przechowywanych w `.store/`, które nie są już referencowane przez żadną wersję w historii.

**Kluczowe założenie**: GC używa strategii **Mark & Sweep** - najpierw oznacza używane pliki (Mark), potem usuwa nieoznaczone (Sweep).

---

## Algorithm: Mark & Sweep

```
INPUT:
  - manifest: Manifest        // Aktualny manifest z historią wersji
  - options: GCOptions        // Opcje GC (grace period, dry run, etc.)

OUTPUT:
  - report: GCReport          // Raport z wyników czyszczenia

PRECONDITION:
  - Manifest jest załadowany i walidowany
  - Wszystkie wersje są dostępne

POSTCONDITION:
  - Orphaned bloby i delty są usunięte
  - ZIP jest zrekompresowany (repacked)
  - Manifest jest zaktualizowany
```

### Phase 1: Mark - Identify Used Resources

**Purpose**: Zbierz wszystkie hashe blobów i ścieżki delt, które są używane przez jakąkolwiek wersję.

Dla **delt**: Jeśli wersja `vN` istnieje i ma parenta `v(N-1)`, to patch `vN_{filehash}.patch` jest POTRZEBNY do przywrócenia `v(N-1)` (Reverse Delta). Więc każda wersja (oprócz najstarszej) implikuje istnienie patchy, które pozwalają z niej "wyjść" w dół.

```typescript
async function markUsedResources(manifest: Manifest): Promise<{
  usedBlobs: Set<string>;
  usedDeltas: Set<string>;
}> {
  const usedBlobs = new Set<string>();
  const usedDeltas = new Set<string>();
  
  // Iterate through all versions
  for (const version of manifest.versionHistory) {
    // Check all file states in this version
    for (const [filePath, fileState] of Object.entries(version.fileStates)) {
      if (fileState.deleted) {
        continue; // Deleted files don't reference anything
      }
      
      // Mark blob if binary file
      if (fileState.hash) {
        usedBlobs.add(fileState.hash);
      }
      
      // Mark delta if text file
      // In Reverse Delta, a text file in version 'vN' might explicitly reference a contentRef
      // OR implicitely rely on a patch named 'vN_...patch' to go to parent.
      // If we use explicit contentRef in FileState:
      if (fileState.contentRef && fileState.contentRef.startsWith('.store/deltas/')) {
        usedDeltas.add(fileState.contentRef);
      }
      
      // Implicit check (safety fallback if contentRef is not strictly used):
      // If this version has a parent, it likely created a patch to go back to it.
      // But only if the file existed in parent and was changed.
      // Generally, relying on explicit contentRef in version object is safer if implemented correctly.
      // If not, we might need to scan .store/deltas/ and check if filename matches any {version.id}_{hashPath(filePath)}.patch
    }
  }
  
  // Also mark blobs/deltas referenced in current HEAD (content/)
  // HEAD files are in /content/, so they don't use blobs directly unless deduplicated?
  // Actually, HEAD state is fully represented in /content/, but it matches the last version in history.
  // The last version in history is already processed above.
  
  // If we have Snapshots (in .store/snapshots/), we should mark them too.
  // Assuming snapshots are tracked or we iterate them.
  
  console.log(`Marked ${usedBlobs.size} used blobs`);
  console.log(`Marked ${usedDeltas.size} used deltas`);
  
  return { usedBlobs, usedDeltas };
}
```

**Złożoność**: O(V × F) gdzie V = versions, F = files per version

**Konkretny przykład**:
```typescript
// Przykład manifestu:
const manifest = {
  versionHistory: [
    {
      id: 'v1',
      fileStates: {
        'src/index.js': { hash: 'abc123', inodeId: 'inode1' },
        'assets/logo.png': { hash: 'def456', inodeId: 'inode2' }
      }
    },
    {
      id: 'v2',
      fileStates: {
        'src/index.js': { hash: 'abc123', inodeId: 'inode1', contentRef: '.store/deltas/v2_hash.patch' }, 
        'assets/logo.png': { hash: 'ghi789', inodeId: 'inode2' }  // Nowy hash
      }
    },
    {
      id: 'v3',
      fileStates: {
        'src/index.js': { hash: 'abc123', inodeId: 'inode1', contentRef: '.store/deltas/v3_hash.patch' },
        'assets/logo.png': { hash: 'def456', inodeId: 'inode2' }  // Wrócono do starego
      }
    }
  ]
};

// Mark phase:
const usedBlobs = new Set<string>();
// v1: dodaj 'abc123', 'def456'
// v2: dodaj 'abc123', 'ghi789' (abc123 już jest, więc tylko ghi789)
// v3: dodaj 'abc123', 'def456' (oba już są)
// Wynik: usedBlobs = Set(['abc123', 'def456', 'ghi789'])

// W storage mamy:
// .store/blobs/abc123  ← używany przez v1, v2, v3
// .store/blobs/def456  ← używany przez v1, v3
// .store/blobs/ghi789  ← używany tylko przez v2
// .store/blobs/xyz999  ← NIE używany przez żadną wersję (orphaned!)
```

### Phase 2: Sweep - Find Orphans

**Purpose**: Znajdź wszystkie bloby i delty w storage, które nie są oznaczone jako używane.

```typescript
async function sweepOrphans(
  usedBlobs: Set<string>,
  usedDeltas: Set<string>
): Promise<{
  orphanedBlobs: string[];
  orphanedDeltas: string[];
}> {
  // List all blobs in storage
  const allBlobs = await this.listBlobs();
  const orphanedBlobs: string[] = [];
  
  for (const blobHash of allBlobs) {
    if (!usedBlobs.has(blobHash)) {
      orphanedBlobs.push(blobHash);
    }
  }
  
  // List all deltas in storage
  const allDeltas = await this.listDeltas();
  const orphanedDeltas: string[] = [];
  
  for (const deltaPath of allDeltas) {
    if (!usedDeltas.has(deltaPath)) {
      orphanedDeltas.push(deltaPath);
    }
  }
  
  console.log(`Found ${orphanedBlobs.length} orphaned blobs`);
  console.log(`Found ${orphanedDeltas.length} orphaned deltas`);
  
  return { orphanedBlobs, orphanedDeltas };
}
```

**Złożoność**: O(B + D) gdzie B = blobs, D = deltas

**Konkretny przykład**:
```typescript
// List all blobs w storage
const allBlobs = [
  'abc123def456...',  // 64-char SHA-256 hash
  'def456ghi789...',
  'ghi789jkl012...',
  'xyz999aaa111...'   // Orphaned!
];

const usedBlobs = new Set(['abc123def456...', 'def456ghi789...', 'ghi789jkl012...']);

// Sweep phase:
const orphanedBlobs: string[] = [];
for (const blobHash of allBlobs) {
  if (!usedBlobs.has(blobHash)) {
    orphanedBlobs.push(blobHash);
  }
}
// Wynik: orphanedBlobs = ['xyz999aaa111...']

// Analogicznie dla deltas:
const allDeltas = [
  '.store/deltas/v2_src_index.js.patch',
  '.store/deltas/v3_src_index.js.patch',
  '.store/deltas/v5_old_file.js.patch'  // Orphaned! (v5 został usunięty)
];

const usedDeltas = new Set([
  '.store/deltas/v2_src_index.js.patch',
  '.store/deltas/v3_src_index.js.patch'
]);

const orphanedDeltas = allDeltas.filter(d => !usedDeltas.has(d));
// Wynik: orphanedDeltas = ['.store/deltas/v5_old_file.js.patch']
```

### Phase 3: Grace Period Filter

**Purpose**: Nie usuwaj plików, które zostały niedawno "orphaned" (możliwe, że są w trakcie użycia).

```typescript
async function filterByGracePeriod(
  orphanedBlobs: string[],
  orphanedDeltas: string[],
  gracePeriodDays: number
): Promise<{
  safeToDeleteBlobs: string[];
  safeToDeleteDeltas: string[];
  inGracePeriodBlobs: string[];
  inGracePeriodDeltas: string[];
}> {
  const now = Date.now();
  const graceMs = gracePeriodDays * 24 * 60 * 60 * 1000;
  
  const safeToDeleteBlobs: string[] = [];
  const inGracePeriodBlobs: string[] = [];
  
  for (const blobHash of orphanedBlobs) {
    const lastUsed = await this.findLastBlobUsage(blobHash);
    
    if (now - lastUsed > graceMs) {
      safeToDeleteBlobs.push(blobHash);
    } else {
      inGracePeriodBlobs.push(blobHash);
    }
  }
  
  const safeToDeleteDeltas: string[] = [];
  const inGracePeriodDeltas: string[] = [];
  
  for (const deltaPath of orphanedDeltas) {
    const lastUsed = await this.findLastDeltaUsage(deltaPath);
    
    if (now - lastUsed > graceMs) {
      safeToDeleteDeltas.push(deltaPath);
    } else {
      inGracePeriodDeltas.push(deltaPath);
    }
  }
  
  return {
    safeToDeleteBlobs,
    safeToDeleteDeltas,
    inGracePeriodBlobs,
    inGracePeriodDeltas
  };
}
```

**Dlaczego grace period?**:
- Chroni przed usunięciem plików w trakcie operacji
- Daje czas na rollback jeśli GC był błędny
- Domyślnie: 7 dni

**Konkretny przykład**:
```typescript
// Obecna data: 2025-01-20
const now = new Date('2025-01-20').getTime();
const gracePeriodDays = 7;
const graceMs = gracePeriodDays * 24 * 60 * 60 * 1000;  // 604800000 ms

// Blob został ostatnio użyty w wersji v10 (2025-01-10)
const lastUsed = new Date('2025-01-10').getTime();
const age = now - lastUsed;  // 10 dni

if (age > graceMs) {
  // 10 dni > 7 dni → safe to delete ✅
  safeToDeleteBlobs.push('abc123');
} else {
  // W grace period → nie usuwaj jeszcze
  inGracePeriodBlobs.push('abc123');
}

// Przykład 2: Blob użyty wczoraj
const lastUsed2 = new Date('2025-01-19').getTime();
const age2 = now - lastUsed2;  // 1 dzień

if (age2 > graceMs) {
  // 1 dzień < 7 dni → w grace period ⏳
  inGracePeriodBlobs.push('def456');
}
```

### Phase 4: Calculate Space to Free

```typescript
async function calculateSpaceToFree(
  safeToDeleteBlobs: string[],
  safeToDeleteDeltas: string[]
): Promise<number> {
  let totalSize = 0;
  
  // Calculate blob sizes
  for (const blobHash of safeToDeleteBlobs) {
    const blobPath = `.store/blobs/${blobHash}`;
    const size = await this.getFileSizeInZip(blobPath);
    totalSize += size;
  }
  
  // Calculate delta sizes
  for (const deltaPath of safeToDeleteDeltas) {
    const size = await this.getFileSizeInZip(deltaPath);
    totalSize += size;
  }
  
  return totalSize;
}
```

### Phase 5: Delete Orphans

```typescript
async function deleteOrphans(
  safeToDeleteBlobs: string[],
  safeToDeleteDeltas: string[]
): Promise<void> {
  // Delete blobs
  for (const blobHash of safeToDeleteBlobs) {
    const blobPath = `.store/blobs/${blobHash}`;
    await this.deleteFromZip(blobPath);
    
    this.emit('gc:progress', {
      phase: 'deleting-blobs',
      current: safeToDeleteBlobs.indexOf(blobHash) + 1,
      total: safeToDeleteBlobs.length
    });
  }
  
  // Delete deltas
  for (const deltaPath of safeToDeleteDeltas) {
    await this.deleteFromZip(deltaPath);
    
    this.emit('gc:progress', {
      phase: 'deleting-deltas',
      current: safeToDeleteDeltas.indexOf(deltaPath) + 1,
      total: safeToDeleteDeltas.length
    });
  }
}
```

### Phase 6: Repack ZIP

**Purpose**: ZIP nie zmniejsza się automatycznie po usunięciu plików - trzeba go zrekompresować.

```typescript
async function repackZip(): Promise<void> {
  console.log('Repacking ZIP to reclaim space...');
  
  // Read all remaining files from ZIP
  const files = await this.listAllFilesInZip();
  const fileData = new Map<string, Uint8Array>();
  
  for (const filePath of files) {
    const data = await this.readFromZip(filePath);
    fileData.set(filePath, data);
  }
  
  // Create new ZIP with only remaining files
  const newZip = await this.createNewZip();
  
  for (const [filePath, data] of fileData) {
    await newZip.addFile(filePath, data);
  }
  
  // Replace old ZIP with new one
  await this.replaceZip(newZip);
  
  console.log('ZIP repacked successfully');
}
```

**Dlaczego repack?**:
- ZIP przechowuje usunięte pliki jako "deleted entries"
- Repack usuwa te wpisy i zmniejsza rozmiar pliku
- Może zająć dużo czasu dla dużych projektów

---

## Complete Algorithm

```typescript
async function runGC(options: GCOptions = {}): Promise<GCReport> {
  const startTime = Date.now();
  
  try {
    this.emit('gc:start', { options });
    
    // Phase 1: Mark
    this.emit('gc:progress', { phase: 'marking', percent: 10 });
    const { usedBlobs, usedDeltas } = await this.markUsedResources(
      this.manifest
    );
    
    // Phase 2: Sweep
    this.emit('gc:progress', { phase: 'sweeping', percent: 30 });
    const { orphanedBlobs, orphanedDeltas } = await this.sweepOrphans(
      usedBlobs,
      usedDeltas
    );
    
    if (orphanedBlobs.length === 0 && orphanedDeltas.length === 0) {
      return {
        blobsRemoved: 0,
        deltasRemoved: 0,
        spaceFreed: 0,
        duration: Date.now() - startTime
      };
    }
    
    // Phase 3: Grace Period Filter
    this.emit('gc:progress', { phase: 'filtering', percent: 50 });
    const gracePeriodDays = options.gracePeriodDays ?? 7;
    const {
      safeToDeleteBlobs,
      safeToDeleteDeltas,
      inGracePeriodBlobs,
      inGracePeriodDeltas
    } = await this.filterByGracePeriod(
      orphanedBlobs,
      orphanedDeltas,
      gracePeriodDays
    );
    
    // Phase 4: Calculate Space
    this.emit('gc:progress', { phase: 'calculating', percent: 60 });
    const spaceFreed = await this.calculateSpaceToFree(
      safeToDeleteBlobs,
      safeToDeleteDeltas
    );
    
    // Dry run check
    if (options.dryRun) {
      return {
        blobsRemoved: safeToDeleteBlobs.length,
        deltasRemoved: safeToDeleteDeltas.length,
        spaceFreed,
        inGracePeriodBlobs: inGracePeriodBlobs.length,
        inGracePeriodDeltas: inGracePeriodDeltas.length,
        duration: Date.now() - startTime,
        dryRun: true
      };
    }
    
    // Phase 5: Delete
    this.emit('gc:progress', { phase: 'deleting', percent: 70 });
    await this.deleteOrphans(safeToDeleteBlobs, safeToDeleteDeltas);
    
    // Phase 6: Repack
    this.emit('gc:progress', { phase: 'repacking', percent: 85 });
    await this.repackZip();
    
    // Phase 7: Update manifest (remove references)
    this.emit('gc:progress', { phase: 'updating', percent: 95 });
    await this.updateManifestAfterGC();
    
    const report: GCReport = {
      blobsRemoved: safeToDeleteBlobs.length,
      deltasRemoved: safeToDeleteDeltas.length,
      spaceFreed,
      inGracePeriodBlobs: inGracePeriodBlobs.length,
      inGracePeriodDeltas: inGracePeriodDeltas.length,
      duration: Date.now() - startTime
    };
    
    this.emit('gc:complete', report);
    
    return report;
    
  } catch (error) {
    this.emit('gc:error', { error });
    throw error;
  }
}
```

---

## Helper Functions

### Find Last Usage

```typescript
async function findLastBlobUsage(blobHash: string): Promise<number> {
  let lastUsed = 0;
  
  // Find last version that uses this blob
  for (const version of this.manifest.versionHistory) {
    for (const fileState of Object.values(version.fileStates)) {
      if (fileState.hash === blobHash) {
        const timestamp = new Date(version.timestamp).getTime();
        lastUsed = Math.max(lastUsed, timestamp);
      }
    }
  }
  
  // If never used, return 0 (safe to delete)
  return lastUsed;
}
```

### List Blobs/Deltas

```typescript
async function listBlobs(): Promise<string[]> {
  const blobs: string[] = [];
  const entries = await this.listZipEntries();
  
  for (const entry of entries) {
    if (entry.startsWith('.store/blobs/')) {
      const hash = entry.replace('.store/blobs/', '');
      // Validate hash format (SHA-256 = 64 hex chars)
      if (/^[a-f0-9]{64}$/i.test(hash)) {
        blobs.push(hash);
      }
    }
  }
  
  return blobs;
}

async function listDeltas(): Promise<string[]> {
  const deltas: string[] = [];
  const entries = await this.listZipEntries();
  
  for (const entry of entries) {
    if (entry.startsWith('.store/deltas/') && entry.endsWith('.patch')) {
      deltas.push(entry);
    }
  }
  
  return deltas;
}
```

---

## Complexity Analysis

### Time Complexity

| Phase | Complexity | Note |
|-------|------------|------|
| Mark | O(V × F) | V = versions, F = files per version |
| Sweep | O(B + D) | B = blobs, D = deltas |
| Grace Period | O(B + D) | Check last usage |
| Delete | O(B + D) | Delete operations |
| Repack | O(Z) | Z = ZIP size |
| **Total** | **O(V × F + B + D + Z)** | |

### Space Complexity

| Component | Space | Note |
|-----------|-------|------|
| Used sets | O(B + D) | Sets of hashes/paths |
| Orphaned arrays | O(B + D) | Arrays of hashes/paths |
| **Total** | **O(B + D)** | |

---

## Edge Cases

### Case 1: No Orphans

```typescript
if (orphanedBlobs.length === 0 && orphanedDeltas.length === 0) {
  return { blobsRemoved: 0, deltasRemoved: 0, spaceFreed: 0 };
}
```

### Case 2: All Files in Grace Period

```typescript
// Jeśli wszystkie orphaned files są w grace period,
// GC nie usunie niczego (bezpieczne)
```

### Case 3: Corrupted References

```typescript
// Problem: Manifest referencuje blob, który nie istnieje w ZIP
// Rozwiązanie: GC nie usuwa referencowanych plików,
// ale może zgłosić warning o missing references
```

### Case 4: Concurrent GC

```typescript
// Problem: GC uruchomiony podczas saveCheckpoint
// Rozwiązanie: Lock mechanism lub queue operations
```

---

## Safety Features

### 1. Dry Run Mode

```typescript
const report = await manager.runGC({ dryRun: true });
console.log(`Would remove ${report.blobsRemoved} blobs`);
console.log(`Would free ${report.spaceFreed} bytes`);
```

### 2. Grace Period

```typescript
// Domyślnie 7 dni - pliki orphaned <7 dni temu nie są usuwane
await manager.runGC({ gracePeriodDays: 7 });
```

### 3. Backup Before GC

```typescript
if (options.createBackup) {
  await this.createBackup('pre-gc');
}
```

### 4. Verification After GC

```typescript
// Po GC, zweryfikuj że wszystkie referencje są nadal ważne
await this.verifyIntegrity();
```

---

## Best Practices

### Do's ✅

1. **Uruchamiaj GC regularnie** (co tydzień lub po delete operations)
2. **Używaj dry run** przed pierwszym GC
3. **Ustaw grace period** (7-14 dni)
4. **Twórz backup** przed GC
5. **Monitoruj space freed** - powinien być >0 dla aktywnych projektów

### Don'ts ❌

1. **Nie uruchamiaj GC podczas saveCheckpoint** - może usunąć potrzebne pliki
2. **Nie ustawiaj grace period = 0** - ryzyko usunięcia potrzebnych plików
3. **Nie ignoruj błędów GC** - może prowadzić do corrupted data
4. **Nie usuwaj blobów ręcznie** - użyj GC

---

## Performance Benchmarks

| Project Size | Blobs | Deltas | GC Time | Space Freed |
|--------------|-------|--------|---------|-------------|
| Small (50 files) | 20 | 30 | 200ms | 5 MB |
| Medium (500 files) | 200 | 500 | 1.2s | 50 MB |
| Large (5000 files) | 2000 | 5000 | 8.5s | 500 MB |

**Wniosek**: GC jest szybkie dla typowych projektów, ale może zająć czas dla bardzo dużych.

---

## Integration

```typescript
// Auto GC after checkpoint (opcjonalne)
async function saveCheckpoint(message: string): Promise<string> {
  const versionId = await this.createVersion(message);
  
  // Auto GC if enabled
  if (this.config.autoGC) {
    await this.runGC({ gracePeriodDays: 7 });
  }
  
  return versionId;
}
```

---

## Kompletny Przykład Wykonania GC

```typescript
// Przed GC:
// Storage: 100 blobów, 200 delt
// Total size: 500 MB

// 1. Mark phase
const { usedBlobs, usedDeltas } = await markUsedResources(manifest);
// usedBlobs.size = 85
// usedDeltas.size = 180

// 2. Sweep phase
const { orphanedBlobs, orphanedDeltas } = await sweepOrphans(usedBlobs, usedDeltas);
// orphanedBlobs.length = 15 (100 - 85)
// orphanedDeltas.length = 20 (200 - 180)

// 3. Grace period filter
const gracePeriodDays = 7;
const { safeToDeleteBlobs, safeToDeleteDeltas } = await filterByGracePeriod(
  orphanedBlobs,
  orphanedDeltas,
  gracePeriodDays
);
// safeToDeleteBlobs.length = 12 (3 są w grace period)
// safeToDeleteDeltas.length = 18 (2 są w grace period)

// 4. Calculate space
const spaceFreed = await calculateSpaceToFree(safeToDeleteBlobs, safeToDeleteDeltas);
// spaceFreed = 75 MB

// 5. Delete
await deleteOrphans(safeToDeleteBlobs, safeToDeleteDeltas);
// Usunięto 12 blobów i 18 delt

// 6. Repack ZIP
await repackZip();
// ZIP zmniejszony z 500 MB do 425 MB

// Wynik:
// {
//   blobsRemoved: 12,
//   deltasRemoved: 18,
//   spaceFreed: 75_000_000,  // 75 MB
//   inGracePeriodBlobs: 3,
//   inGracePeriodDeltas: 2
// }
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
