# 🗄️ Content Addressable Storage (CAS) - Zarządzanie Plikami Binarnymi

## 1. Wprowadzenie

**Content Addressable Storage (CAS)** to system przechowywania, gdzie **nazwa pliku = hash jego zawartości**. W JCF używamy CAS do efektywnego zarządzania plikami binarnymi (obrazy, wideo, archiwia, itp.).

## 2. Podstawowe Koncepcje

### 2.1 Co to jest Content Addressing?

**Tradycyjny system**:
```
filename: logo.png
content: [binary data]
```
Problem: Ten sam content może być duplikowany pod różnymi nazwami

**Content Addressable**:
```
hash: a3f5e8d9c1b2e4f67890abcdef123456...
content: [binary data]
```
Zaleta: Ten sam content = ten sam hash = jeden plik

### 2.2 Dlaczego SHA-256?

**Porównanie algorytmów**:

| Algorytm | Size | Collision Prob | Performance | Używany przez |
|----------|------|----------------|-------------|---------------|
| MD5 | 128-bit | ⚠️ Vulnerable | ⚡⚡⚡ | (deprecated) |
| SHA-1 | 160-bit | ⚠️ Broken | ⚡⚡ | Git (legacy) |
| **SHA-256** | **256-bit** | **✅ Extremely low** | **⚡** | **Bitcoin, Git (new)** |
| SHA-512 | 512-bit | ✅ Overkill | 🐌 | - |

**SHA-256 dla JCF**:
- ✅ Collision resistant (2^256 possible hashes)
- ✅ Native WebCrypto API support
- ✅ Wide ecosystem support
- ✅ Balance: security vs performance

**Prawdopodobieństwo kolizji**:
```
Aby mieć 50% szansy na kolizję, potrzebujesz:
2^128 = 340,282,366,920,938,463,463,374,607,431,768,211,456 plików

Czyli praktycznie: ZERO RISK
```

## 3. Struktura `.store/blobs/`

### 3.1 Organizacja Plików

```
.store/blobs/
├── a3f5e8d9c1b2e4f67890abcdef1234567890abcdef1234567890abcdef123456
├── 9d4c1e2b3a4f5e6d7c8b9a0f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8f9e0
├── f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8f9e0d1c2b3a4f5e6d7c8b9a0f
└── ...
```

**Naming**: 
- Filename = SHA-256 hex (64 characters)
- No file extension
- Flat structure (no subdirectories)

**Dlaczego flat structure?**:
- ✅ Simple implementation
- ✅ No path traversal issues
- ✅ Easy to list and garbage collect

**Alternatywa (Git-style sharding)**:
```
.store/blobs/
├── a3/
│   └── f5e8d9c1b2e4f67890abcdef1234567890abcdef1234567890abcdef123456
├── 9d/
│   └── 4c1e2b3a4f5e6d7c8b9a0f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8f9e0
```
Problem: Complexity, performance na niektórych filesystemach

**Decyzja**: Flat structure (prostsze, wystarczające)

### 3.2 Compression Policy

```typescript
// Binary files w CAS: STORE (no compression)
// Dlaczego?
// 1. Większość binary files jest już skompresowana (PNG, JPG, MP4, ZIP)
// 2. Recompression = CPU waste + marginal gains
// 3. Integrity: SHA-256 hash musi pasować do original content
```

**Test compressibility**:
```typescript
function shouldCompressBlob(data: Uint8Array): boolean {
  // Quick sample test
  const sample = data.slice(0, 4096);
  const compressed = deflate(sample, { level: 1 });
  
  const ratio = compressed.length / sample.length;
  
  // If compression saves <10%, don't bother
  return ratio < 0.9;
}
```

## 4. Algorytm Save Blob

### 4.1 High-Level Flow

```typescript
async function saveBlob(
  filePath: string,
  content: Uint8Array
): Promise<string> {
  // 1. Hash content
  const hash = await sha256(content);
  
  // 2. Check if blob already exists (deduplication!)
  const blobPath = `.store/blobs/${hash}`;
  
  if (await this.blobExists(blobPath)) {
    console.log(`✅ Blob deduplicated: ${hash}`);
    return hash;
  }
  
  // 3. Write new blob
  await this.writeToZip(blobPath, content);
  console.log(`💾 New blob saved: ${hash}`);
  
  // 4. Update file map
  this.manifest.fileMap[filePath].currentHash = hash;
  
  return hash;
}
```

### 4.2 Streaming for Large Files

**Problem**: 500MB file nie może być loaded do RAM w przeglądarce

**Rozwiązanie**: Stream hashing + writing

```typescript
async function saveBlobStreaming(
  filePath: string,
  stream: ReadableStream
): Promise<string> {
  // Tee stream: one for hashing, one for writing
  const [hashStream, writeStream] = stream.tee();
  
  // Hash in parallel with writing
  const hashPromise = sha256Stream(hashStream);
  
  // Write to temp location first
  const tempPath = `.store/temp/${uuidv4()}`;
  await this.writeToZipStream(tempPath, writeStream);
  
  // Get hash
  const hash = await hashPromise;
  const finalPath = `.store/blobs/${hash}`;
  
  // Check if blob exists
  if (await this.blobExists(finalPath)) {
    // Delete temp (deduplicated)
    await this.deleteFromZip(tempPath);
    console.log(`✅ Large blob deduplicated: ${hash}`);
  } else {
    // Rename temp to final
    await this.renameInZip(tempPath, finalPath);
    console.log(`💾 Large blob saved: ${hash}`);
  }
  
  return hash;
}
```

**SHA-256 Streaming** (using WebCrypto):
```typescript
async function sha256Stream(stream: ReadableStream): Promise<string> {
  // WebCrypto API doesn't support streaming directly
  // Use chunked approach with hash-wasm library
  
  const hasher = await createSHA256();
  const reader = stream.getReader();
  
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      
      hasher.update(value);
    }
    
    const hash = hasher.digest('hex');
    return hash;
    
  } finally {
    reader.releaseLock();
  }
}
```

## 5. Algorytm Load Blob

### 5.1 Basic Load

```typescript
async function loadBlob(hash: string): Promise<Uint8Array> {
  const blobPath = `.store/blobs/${hash}`;
  
  if (!await this.blobExists(blobPath)) {
    throw new BlobNotFoundError(hash);
  }
  
  return await this.readFromZip(blobPath);
}
```

### 5.2 Stream Load

```typescript
async function loadBlobStream(hash: string): Promise<ReadableStream> {
  const blobPath = `.store/blobs/${hash}`;
  
  if (!await this.blobExists(blobPath)) {
    throw new BlobNotFoundError(hash);
  }
  
  return await this.readFromZipStream(blobPath);
}
```

### 5.3 Verification on Load

```typescript
async function loadBlobWithVerification(
  hash: string
): Promise<Uint8Array> {
  const content = await this.loadBlob(hash);
  
  // Verify integrity
  const actualHash = await sha256(content);
  
  if (actualHash !== hash) {
    throw new BlobCorruptionError(
      `Expected: ${hash}, Got: ${actualHash}`
    );
  }
  
  return content;
}
```

## 6. Deduplication Examples

### 6.1 Same File in Multiple Versions

```typescript
// Scenario: logo.png nie zmienia się przez 10 wersji

// v1: Add logo.png
await manager.addFile('logo.png', logoData);
// → Blob saved: a3f5e8...

// v2-v10: logo.png unchanged
// → No new blobs saved!
// All versions point to the same blob: a3f5e8...

// Result: 1 blob storage, 10 version references
```

### 6.2 File Revert

```typescript
// v1: logo.png (hash: abc123)
// v2: logo.png changed (hash: def456)
// v3: logo.png reverted to v1 (hash: abc123)

// Storage:
// .store/blobs/abc123  ← Used by v1 and v3
// .store/blobs/def456  ← Used by v2

// Total: 2 blobs (not 3!)
```

### 6.3 Multiple Files with Same Content

```typescript
// User accidentally copies logo.png to logo-copy.png

await manager.addFile('assets/logo.png', logoData);
// → Blob: a3f5e8...

await manager.addFile('assets/logo-copy.png', logoData);
// → Blob: a3f5e8... (same! deduplicated)

// Storage: 1 blob
// References: 2 files pointing to same blob
```

## 7. Garbage Collection

### 7.1 Identifying Orphaned Blobs

**Orphaned blob** = blob nie używany przez żaden version

```typescript
async function identifyOrphanedBlobs(): Promise<string[]> {
  // Step 1: Collect all referenced blobs
  const referencedBlobs = new Set<string>();
  
  for (const version of this.manifest.versionHistory) {
    for (const fileState of Object.values(version.fileStates)) {
      if (fileState.hash) {
        referencedBlobs.add(fileState.hash);
      }
    }
  }
  
  console.log(`Referenced blobs: ${referencedBlobs.size}`);
  
  // Step 2: List all blobs in storage
  const allBlobs = await this.listBlobs();
  console.log(`Total blobs: ${allBlobs.length}`);
  
  // Step 3: Find orphans
  const orphaned: string[] = [];
  
  for (const blobHash of allBlobs) {
    if (!referencedBlobs.has(blobHash)) {
      orphaned.push(blobHash);
    }
  }
  
  console.log(`Orphaned blobs: ${orphaned.length}`);
  return orphaned;
}
```

### 7.2 Deleting Orphaned Blobs

```typescript
async function runGC(): Promise<GCReport> {
  const orphaned = await this.identifyOrphanedBlobs();
  
  if (orphaned.length === 0) {
    return { blobsRemoved: 0, spaceFreed: 0 };
  }
  
  // Calculate space to be freed
  let spaceFreed = 0;
  for (const hash of orphaned) {
    const blobPath = `.store/blobs/${hash}`;
    const size = await this.getFileSize(blobPath);
    spaceFreed += size;
  }
  
  console.log(`Will free: ${formatBytes(spaceFreed)}`);
  
  // Delete orphaned blobs
  for (const hash of orphaned) {
    const blobPath = `.store/blobs/${hash}`;
    await this.deleteFromZip(blobPath);
  }
  
  // Repack ZIP to reclaim space
  await this.repackZip();
  
  return {
    blobsRemoved: orphaned.length,
    spaceFreed
  };
}
```

### 7.3 Safe GC with Grace Period

```typescript
async function safeGC(gracePeriodDays: number = 7): Promise<GCReport> {
  const orphaned = await this.identifyOrphanedBlobs();
  const now = Date.now();
  const graceMs = gracePeriodDays * 24 * 60 * 60 * 1000;
  
  const safeToDelete: string[] = [];
  
  for (const hash of orphaned) {
    // Find when blob was last referenced
    const lastUsed = await this.findLastBlobUsage(hash);
    
    if (now - lastUsed > graceMs) {
      safeToDelete.push(hash);
    } else {
      console.log(`⏳ Blob ${hash} in grace period`);
    }
  }
  
  // Delete only safe blobs
  for (const hash of safeToDelete) {
    await this.deleteFromZip(`.store/blobs/${hash}`);
  }
  
  return {
    blobsRemoved: safeToDelete.length,
    blobsInGracePeriod: orphaned.length - safeToDelete.length,
    spaceFreed: await this.calculateTotalSize(safeToDelete)
  };
}
```

## 8. Performance Optimizations

### 8.1 Blob Caching

```typescript
class BlobManager {
  private cache = new LRUCache<string, Uint8Array>({
    max: 100, // Cache up to 100 blobs
    maxSize: 100 * 1024 * 1024, // 100MB total
    sizeCalculation: (value) => value.byteLength
  });
  
  async loadBlob(hash: string): Promise<Uint8Array> {
    // Check cache first
    const cached = this.cache.get(hash);
    if (cached) {
      console.log(`✅ Cache hit: ${hash}`);
      return cached;
    }
    
    // Load from storage
    const content = await this.loadBlobFromStorage(hash);
    
    // Cache for future
    this.cache.set(hash, content);
    
    return content;
  }
}
```

### 8.2 Parallel Blob Operations

```typescript
async function saveManyBlobs(
  files: Map<string, Uint8Array>
): Promise<Map<string, string>> {
  const results = new Map<string, string>();
  
  // Process blobs in parallel (use worker pool)
  const promises = Array.from(files.entries()).map(
    async ([path, content]) => {
      const hash = await this.workers.hash.compute(content);
      return [path, hash] as const;
    }
  );
  
  const hashes = await Promise.all(promises);
  
  // Save unique blobs
  const uniqueHashes = new Set(hashes.map(([, hash]) => hash));
  
  for (const hash of uniqueHashes) {
    if (!await this.blobExists(`.store/blobs/${hash}`)) {
      const content = files.get(
        hashes.find(([, h]) => h === hash)![0]
      )!;
      await this.saveBlob(hash, content);
    }
  }
  
  return new Map(hashes);
}
```

### 8.3 Blob Prefetching

```typescript
async function prefetchBlobsForVersion(versionId: string): Promise<void> {
  const version = this.getVersion(versionId);
  
  // Collect all blob hashes for this version
  const blobHashes = Object.values(version.fileStates)
    .filter(fs => fs.hash)
    .map(fs => fs.hash!);
  
  // Prefetch in background
  for (const hash of blobHashes) {
    this.loadBlob(hash).catch(err => {
      console.warn(`Prefetch failed for ${hash}:`, err);
    });
  }
}
```

## 9. Security Considerations

### 9.1 Hash Verification

**Always verify** na load (szczególnie po network transfer):

```typescript
async function loadBlobSecure(hash: string): Promise<Uint8Array> {
  const content = await this.loadBlob(hash);
  
  // Compute actual hash
  const actualHash = await sha256(content);
  
  if (actualHash !== hash) {
    // Possible corruption or tampering
    throw new SecurityError(
      `Blob integrity check failed!\n` +
      `Expected: ${hash}\n` +
      `Got: ${actualHash}`
    );
  }
  
  return content;
}
```

### 9.2 Size Limits

**Prevent resource exhaustion**:

```typescript
async function saveBlob(
  filePath: string,
  content: Uint8Array
): Promise<string> {
  // Enforce size limits
  const MAX_BLOB_SIZE = 500 * 1024 * 1024; // 500MB
  
  if (content.byteLength > MAX_BLOB_SIZE) {
    throw new BlobTooLargeError(
      `File ${filePath} is ${formatBytes(content.byteLength)}, ` +
      `max allowed is ${formatBytes(MAX_BLOB_SIZE)}`
    );
  }
  
  // ... rest of save logic
}
```

### 9.3 Path Traversal Protection

**Blobs są read-only i content-addressed** = bezpieczne, ale:

```typescript
function sanitizeBlobPath(hash: string): string {
  // Ensure hash is valid SHA-256 hex
  if (!/^[a-f0-9]{64}$/i.test(hash)) {
    throw new InvalidHashError(`Invalid hash format: ${hash}`);
  }
  
  return `.store/blobs/${hash.toLowerCase()}`;
}
```

## 10. Monitoring and Metrics

### 10.1 Blob Statistics

```typescript
interface BlobStats {
  totalBlobs: number;
  totalSize: number;
  averageBlobSize: number;
  largestBlob: { hash: string; size: number };
  deduplicationRatio: number;
}

async function getBlobStats(): Promise<BlobStats> {
  const blobs = await this.listBlobs();
  
  let totalSize = 0;
  let largestBlob = { hash: '', size: 0 };
  
  for (const hash of blobs) {
    const size = await this.getBlobSize(hash);
    totalSize += size;
    
    if (size > largestBlob.size) {
      largestBlob = { hash, size };
    }
  }
  
  // Calculate deduplication savings
  const totalReferences = this.countBlobReferences();
  const deduplicationRatio = totalReferences / blobs.length;
  
  return {
    totalBlobs: blobs.length,
    totalSize,
    averageBlobSize: totalSize / blobs.length,
    largestBlob,
    deduplicationRatio
  };
}
```

### 10.2 Deduplication Report

```typescript
async function getDeduplicationReport(): Promise<string> {
  const stats = await this.getBlobStats();
  
  const naiveSize = stats.totalSize * stats.deduplicationRatio;
  const actualSize = stats.totalSize;
  const savedBytes = naiveSize - actualSize;
  const savedPercent = (savedBytes / naiveSize) * 100;
  
  return `
📊 Deduplication Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total unique blobs:    ${stats.totalBlobs}
Total references:      ${Math.round(stats.totalBlobs * stats.deduplicationRatio)}
Deduplication ratio:   ${stats.deduplicationRatio.toFixed(2)}x

Storage used:          ${formatBytes(actualSize)}
Without dedup:         ${formatBytes(naiveSize)}
Space saved:           ${formatBytes(savedBytes)} (${savedPercent.toFixed(1)}%)
  `.trim();
}
```

## 11. Best Practices

### 11.1 Do's ✅

1. **Zawsze weryfikuj hash** po network transfer
2. **Używaj streaming** dla plików >50MB
3. **Uruchamiaj GC regularnie** (co tydzień lub po delete operations)
4. **Cache frequently accessed blobs** (LRU cache)
5. **Monitor deduplication ratio** (powinien być >1.5x)

### 11.2 Don'ts ❌

1. **Nie kompresuj już skompresowanych** (PNG, JPG, MP4)
2. **Nie load wszystkich blobów** do pamięci naraz
3. **Nie delete blobów ręcznie** (używaj GC)
4. **Nie modify blob content** (immutable by design)
5. **Nie store małych plików** jako blobs (<1KB = overhead)

## 12. Troubleshooting

### 12.1 Missing Blob Error

```
Error: Blob not found: a3f5e8d9...
```

**Możliwe przyczyny**:
1. GC usunął blob przez błąd
2. Korupcja ZIP
3. Incomplete transfer

**Rozwiązanie**:
```typescript
// Enable blob reference tracking
this.config.trackBlobUsage = true;

// Before GC, verify references
await this.verifyBlobReferences();
```

### 12.2 Slow Blob Operations

**Symptom**: saveBlob() trwa >5s dla 100MB pliku

**Diagnoza**:
```typescript
const start = performance.now();
const hash = await sha256(content);
console.log(`Hash: ${performance.now() - start}ms`);

const writeStart = performance.now();
await this.writeToZip(path, content);
console.log(`Write: ${performance.now() - writeStart}ms`);
```

**Rozwiązanie**: Use Web Worker dla hashing

### 12.3 High Deduplication Ratio

**Symptom**: deduplicationRatio = 10x (suspicious!)

**Możliwa przyczyna**: Te same dane w wielu plikach

**Sprawdź**:
```typescript
const topBlobs = await this.getTopReferencedBlobs(10);
// Example output:
// a3f5e8... → 50 references (assets/logo.png across versions)
```

## 13. Następne Kroki

1. Przeczytaj [Adapters](./05-adapters.md) dla różnych platform
2. Zobacz [Workers](./06-workers.md) dla performance
3. Sprawdź [API Reference](../api/BlobManager.md)

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

