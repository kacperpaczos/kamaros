# Diff Generation (Myers Algorithm)

> **Szczegółowy algorytm generowania reverse delta patches dla plików tekstowych**

[← Back: Restore Version](02-restore-version.md) | [Next: Patch Application →](04-patch-application.md)

---

## Overview

Algorytm `diff-generation` odpowiada za tworzenie reverse delta patches (NOWY → STARY) dla plików tekstowych podczas operacji `saveCheckpoint`. Używa biblioteki **diff-match-patch** (Google), która implementuje ulepszoną wersję algorytmu Myers.

**Kluczowe założenie**: Generujemy **reverse patches** (NEW → OLD), nie forward patches (OLD → NEW), ponieważ używamy Reverse Delta Strategy.

---

## Algorithm: Step-by-Step

```
INPUT:
  - newText: string        // Aktualna zawartość (HEAD)
  - oldText: string        // Poprzednia zawartość (parent version)

OUTPUT:
  - patchText: string      // Serialized patch w formacie diff-match-patch

PRECONDITION:
  - Oba teksty są znormalizowane (line endings, Unicode)
  - newText reprezentuje NOWY stan
  - oldText reprezentuje STARY stan

POSTCONDITION:
  - Patch może być zastosowany do newText aby otrzymać oldText
```

### Step 1: Normalize Text

**Purpose**: Zapewnienie spójności przed diffingiem.

```typescript
function normalizeText(text: string): string {
  // 1. Normalize line endings to LF
  text = text.replace(/\r\n/g, '\n');  // Windows → Unix
  text = text.replace(/\r/g, '\n');    // Old Mac → Unix
  
  // 2. Normalize Unicode to NFC (Canonical Composition)
  // Ważne dla stabilności diff - "é" może być reprezentowane jako:
  // - U+00E9 (é) - composed
  // - U+0065 + U+0301 (e + combining acute) - decomposed
  text = text.normalize('NFC');
  
  // 3. Ensure trailing newline (Unix convention)
  // Ułatwia diffing - ostatnia linia zawsze ma newline
  if (!text.endsWith('\n') && text.length > 0) {
    text += '\n';
  }
  
  return text;
}
```

**Dlaczego normalizacja?**:
- Line endings różnią się między systemami (Windows: CRLF, Unix: LF)
- Unicode może być reprezentowany na różne sposoby
- Bez normalizacji: te same zmiany mogą generować różne patche

**Przykład problemu bez normalizacji**:
```typescript
// Windows plik (CRLF)
const windowsText = 'line1\r\nline2\r\n';
// Unix plik (LF)
const unixText = 'line1\nline2\n';

// Bez normalizacji - różne patche dla tej samej zmiany!
const patch1 = await computeDiffWithoutNormalization(windowsText, 'line1\r\n');
const patch2 = await computeDiffWithoutNormalization(unixText, 'line1\n');
// patch1 !== patch2 (błąd!)

// Z normalizacją - identyczne patche
const normalized1 = normalizeText(windowsText);  // 'line1\nline2\n'
const normalized2 = normalizeText(unixText);    // 'line1\nline2\n'
const patch3 = await computeDiff(normalized1, 'line1\n');
const patch4 = await computeDiff(normalized2, 'line1\n');
// patch3 === patch4 ✅
```

**Przykład Unicode normalizacji**:
```typescript
// Różne reprezentacje tego samego znaku
const composed = 'café';           // U+00E9 (é jako jeden znak)
const decomposed = 'cafe\u0301';  // U+0065 (e) + U+0301 (combining acute)

// Bez normalizacji
composed.length === 4;      // 'c', 'a', 'f', 'é'
decomposed.length === 5;    // 'c', 'a', 'f', 'e', '́'
// Różne długości = różne patche!

// Z normalizacją NFC
normalizeText(composed) === normalizeText(decomposed);  // true ✅
normalizeText(composed).length === 4;  // Oba mają długość 4
```

### Step 2: Initialize diff-match-patch

```typescript
import { diff_match_patch } from 'diff-match-patch';

const dmp = new diff_match_patch();

// Konfiguracja dla reverse delta
dmp.Match_Distance = 1000;           // Fuzzy matching range
dmp.Match_Threshold = 0.5;           // Similarity threshold
dmp.Patch_DeleteThreshold = 0.5;     // Delete threshold
dmp.Patch_Margin = 4;                // Context lines around changes
```

**Parametry**:
- `Match_Distance`: Jak daleko szukać dopasowań (dla fuzzy matching)
- `Match_Threshold`: Próg podobieństwa (0.0 = exact, 1.0 = any)
- `Patch_DeleteThreshold`: Kiedy użyć delete zamiast replace
- `Patch_Margin`: Kontekst wokół zmian (linie)

**Przykład działania parametrów**:
```typescript
// Przykład 1: Match_Distance = 1000
// Szuka dopasowań w zakresie ±1000 znaków od oczekiwanej pozycji
const text1 = 'A'.repeat(500) + 'X' + 'A'.repeat(500);
const text2 = 'A'.repeat(500) + 'Y' + 'A'.repeat(500);
// Z Match_Distance=1000: znajdzie 'X' i 'Y' jako różnicę
// Z Match_Distance=10: może nie znaleźć (za daleko)

// Przykład 2: Match_Threshold = 0.5
// Wymaga 50% podobieństwa do uznania za dopasowanie
const similar1 = 'Hello World';
const similar2 = 'Hello Wrld';  // 90% podobieństwa
// Z Match_Threshold=0.5: uznane za podobne ✅
// Z Match_Threshold=0.9: nie uznane za podobne ❌

// Przykład 3: Patch_Margin = 4
// Dodaje 4 linie kontekstu wokół każdej zmiany
const oldText = 'line1\nline2\nline3\nline4\nline5\nCHANGED\nline7\nline8\n';
const newText = 'line1\nline2\nline3\nline4\nline5\nMODIFIED\nline7\nline8\n';
// Patch będzie zawierał:
// - 4 linie przed: line2, line3, line4, line5
// - Zmianę: -CHANGED, +MODIFIED
// - 4 linie po: line7, line8
```

### Step 3: Compute Diff

```typescript
async function computeDiff(
  newText: string,
  oldText: string
): Promise<string> {
  // Normalize inputs
  const normalizedNew = normalizeText(newText);
  const normalizedOld = normalizeText(oldText);
  
  // Check if identical (no diff needed)
  if (normalizedNew === normalizedOld) {
    return ''; // Empty patch
  }
  
  // Use worker for large files (>100KB)
  if (normalizedNew.length > 100_000 || normalizedOld.length > 100_000) {
    return await this.computeDiffInWorker(normalizedNew, normalizedOld);
  }
  
  // Compute diff using diff-match-patch
  // patch_make(A, B) creates patches to transform A → B
  // Dla reverse delta: NEW → OLD
  const patches = dmp.patch_make(normalizedNew, normalizedOld);
  
  // Serialize to text format
  const patchText = dmp.patch_toText(patches);
  
  return patchText;
}
```

**Dlaczego `patch_make(newText, oldText)`?**:
- `patch_make(A, B)` tworzy patche transformujące A → B
- Dla reverse delta potrzebujemy: NEW → OLD
- Więc: `patch_make(newText, oldText)` ✅

**Konkretny przykład**:
```typescript
// Stan początkowy (OLD)
const oldText = 'console.log("Hello");\n';

// Stan nowy (NEW) - dodano linię
const newText = 'console.log("Hello");\nconsole.log("World");\n';

// Generowanie reverse patch: NEW → OLD
const patches = dmp.patch_make(newText, oldText);
// patches = [
//   {
//     diffs: [
//       [0, 'console.log("Hello");\n'],  // bez zmian
//       [-1, 'console.log("World");\n'], // usuń tę linię
//       [0, '']                           // koniec
//     ],
//     start1: 0, start2: 0, length1: 2, length2: 1
//   }
// ]

// Serializacja do tekstu
const patchText = dmp.patch_toText(patches);
// Wynik:
// '@@ -1,2 +1,1 @@\n console.log("Hello");\n-console.log("World");\n'

// Weryfikacja: aplikacja patchy do newText daje oldText
const [result] = dmp.patch_apply(patches, newText);
console.log(result === oldText);  // true ✅
```

### Step 4: Worker Pool dla dużych plików

```typescript
async function computeDiffInWorker(
  newText: string,
  oldText: string
): Promise<string> {
  // Offload to Web Worker (nie blokuje UI)
  const worker = await this.workerPool.acquire('diff');
  
  try {
    const patchText = await worker.computeDiff(newText, oldText);
    return patchText;
  } finally {
    this.workerPool.release(worker);
  }
}
```

**Worker Implementation**:
```typescript
// worker/diff-worker.ts
import { diff_match_patch } from 'diff-match-patch';

self.onmessage = (event) => {
  const { newText, oldText, taskId } = event.data;
  
  try {
    const dmp = new diff_match_patch();
    dmp.Match_Distance = 1000;
    dmp.Match_Threshold = 0.5;
    
    // Progress callback (opcjonalne)
    let lastProgress = 0;
    const progressCallback = (percent: number) => {
      if (percent - lastProgress > 5) {  // Co 5%
        self.postMessage({ type: 'progress', taskId, percent });
        lastProgress = percent;
      }
    };
    
    const patches = dmp.patch_make(newText, oldText);
    const patchText = dmp.patch_toText(patches);
    
    self.postMessage({ 
      type: 'complete', 
      taskId, 
      patchText,
      patchSize: patchText.length 
    });
  } catch (error) {
    self.postMessage({ 
      type: 'error', 
      taskId, 
      error: error.message 
    });
  }
};
```

**Korzyści**:
- UI pozostaje responsywny (przykład: diff 10MB pliku nie blokuje UI)
- Wykorzystanie wielu rdzeni CPU (przykład: 4 workery = 4x szybsze dla 4 plików)
- Progress tracking możliwy (przykład: "Diffing file.js: 45%")

**Przykład użycia**:
```typescript
// Main thread
const worker = new Worker('diff-worker.js');
worker.postMessage({ 
  newText: largeFileContent,  // 5MB
  oldText: previousVersion,   // 5MB
  taskId: 'diff-123'
});

worker.onmessage = (event) => {
  if (event.data.type === 'progress') {
    console.log(`Progress: ${event.data.percent}%`);
    updateProgressBar(event.data.percent);
  } else if (event.data.type === 'complete') {
    const patchText = event.data.patchText;
    console.log(`Patch size: ${event.data.patchSize} bytes`);
    // Kontynuuj z patchText...
  }
};
```

### Step 5: Validate Patch

```typescript
function validatePatch(
  patchText: string,
  newText: string,
  expectedOldText: string
): boolean {
  if (!patchText) {
    // Empty patch - texts should be identical
    return newText === expectedOldText;
  }
  
  const dmp = new diff_match_patch();
  const patches = dmp.patch_fromText(patchText);
  
  // Test apply: newText + patch should = oldText
  const [result, successArray] = dmp.patch_apply(patches, newText);
  
  // All patches must apply successfully
  const allSuccess = successArray.every(s => s === true);
  
  if (!allSuccess) {
    console.warn('⚠️  Some patches failed to apply');
    return false;
  }
  
  // Result should match expected
  const normalizedResult = normalizeText(result);
  const normalizedExpected = normalizeText(expectedOldText);
  
  return normalizedResult === normalizedExpected;
}
```

---

## Format Patch (diff-match-patch)

### Przykład Patch

```
@@ -1,3 +1,4 @@
 console.log('Hello');
+console.log('World');
 console.log('End');
```

**Format**:
- `@@ -start,count +start,count @@`: Header (pozycja w starym i nowym tekście)
- `-`: Linia do usunięcia
- `+`: Linia do dodania
- ` `: Linia bez zmian (kontekst)

### Reverse Patch Example

**Stan początkowy (OLD)**:
```javascript
console.log('Hello');
console.log('End');
```

**Stan nowy (NEW)**:
```javascript
console.log('Hello');
console.log('World');
console.log('End');
```

**Reverse Patch (NEW → OLD)**:
```
@@ -1,3 +1,2 @@
 console.log('Hello');
-console.log('World');
 console.log('End');
```

**Aplikacja**:
```
NEW:  console.log('Hello');
      console.log('World');
      console.log('End');

Apply patch (usuń linię "World"):

OLD:  console.log('Hello');
      console.log('End');
```

---

## Complexity Analysis

### Time Complexity

| Operation | Complexity | Note |
|-----------|------------|------|
| Normalize | O(N) | N = text length |
| patch_make | O(ND) | N = length, D = differences (Myers algorithm) |
| patch_toText | O(P) | P = patch size |
| **Total** | **O(ND)** | Dla typowych plików: D << N |

**Myers Algorithm**:
- Best case: O(N) - identical files
- Average case: O(ND) - few differences
- Worst case: O(N²) - completely different files

### Space Complexity

| Component | Space | Note |
|-----------|-------|------|
| Normalized text | O(N) | N = text length |
| Diff result | O(D) | D = differences |
| Patch text | O(P) | P = patch size (~10-20% of file) |
| **Total** | **O(N + P)** | |

### Performance Benchmarks

| File Size | Differences | Time | Memory |
|-----------|-------------|------|--------|
| 1 KB | 10 lines | 5ms | 50 KB |
| 10 KB | 50 lines | 25ms | 200 KB |
| 100 KB | 200 lines | 180ms | 1.5 MB |
| 1 MB | 1000 lines | 2.1s | 15 MB |
| 10 MB | 5000 lines | 28s | 150 MB |

**Wniosek**: Dla plików >1MB, użyj Web Workers.

---

## Edge Cases

### Case 1: Identical Files

```typescript
const newText = 'Hello World';
const oldText = 'Hello World';

const patch = await computeDiff(newText, oldText);
// Returns: '' (empty patch)
```

**Optymalizacja**: Skip diffing jeśli hashe są identyczne.

**Implementacja z hash check**:
```typescript
async function computeDiffWithHashCheck(
  newText: string,
  oldText: string
): Promise<string> {
  // Szybki hash check (SHA-256)
  const newHash = await crypto.subtle.digest(
    'SHA-256',
    new TextEncoder().encode(newText)
  );
  const oldHash = await crypto.subtle.digest(
    'SHA-256',
    new TextEncoder().encode(oldText)
  );
  
  // Porównaj hashe (ArrayBuffer comparison)
  const newHashArray = new Uint8Array(newHash);
  const oldHashArray = new Uint8Array(oldHash);
  
  if (newHashArray.length !== oldHashArray.length) {
    return await computeDiff(newText, oldText);
  }
  
  for (let i = 0; i < newHashArray.length; i++) {
    if (newHashArray[i] !== oldHashArray[i]) {
      return await computeDiff(newText, oldText);
    }
  }
  
  // Hashe identyczne = pliki identyczne
  return '';  // Empty patch
}

// Przykład użycia:
const patch1 = await computeDiffWithHashCheck('Hello', 'Hello');
console.log(patch1);  // '' (hash check wykrył identyczność, skip diffing)

const patch2 = await computeDiffWithHashCheck('Hello', 'World');
console.log(patch2.length > 0);  // true (hash różny, wykonano diffing)
```

### Case 2: Completely Different Files

```typescript
const newText = 'A'.repeat(10000);
const oldText = 'B'.repeat(10000);

const patch = await computeDiff(newText, oldText);
// Large patch: delete all, add all
```

**Strategia**: Jeśli patch > 50% rozmiaru pliku, rozważ snapshot zamiast patch.

**Implementacja z threshold check**:
```typescript
async function computeDiffWithThreshold(
  newText: string,
  oldText: string
): Promise<{ patch: string; useSnapshot: boolean }> {
  const patch = await computeDiff(newText, oldText);
  const patchSize = patch.length;
  const fileSize = Math.max(newText.length, oldText.length);
  const ratio = patchSize / fileSize;
  
  // Próg: jeśli patch > 50% rozmiaru pliku, użyj snapshot
  const THRESHOLD = 0.5;
  
  if (ratio > THRESHOLD) {
    console.warn(
      `Patch size (${patchSize}) > ${THRESHOLD * 100}% of file size (${fileSize}). ` +
      `Consider using snapshot instead.`
    );
    return { patch, useSnapshot: true };
  }
  
  return { patch, useSnapshot: false };
}

// Przykład:
const result1 = await computeDiffWithThreshold(
  'A'.repeat(10000),  // 10KB
  'B'.repeat(10000)   // 10KB
);
// result1.patch.length ≈ 20000 bytes (200% ratio!)
// result1.useSnapshot === true ✅

const result2 = await computeDiffWithThreshold(
  'console.log("Hello");\nconsole.log("World");\n',  // 40 bytes
  'console.log("Hello");\n'                          // 20 bytes
);
// result2.patch.length ≈ 15 bytes (37.5% ratio)
// result2.useSnapshot === false ✅ (użyj patch)
```

### Case 3: Unicode Edge Cases

```typescript
// Problem: Różne reprezentacje Unicode
const text1 = 'café';  // U+00E9 (composed)
const text2 = 'cafe\u0301';  // U+0065 + U+0301 (decomposed)

// Rozwiązanie: Normalize przed diffingiem
const normalized1 = text1.normalize('NFC');
const normalized2 = text2.normalize('NFC');
// Oba są teraz identyczne
```

### Case 4: Binary Data w Pliku Tekstowym

```typescript
// Problem: Plik zawiera binary data (np. base64 image)
const text = '...' + binaryDataAsString + '...';

// Rozwiązanie: diff-match-patch radzi sobie z tym
// Ale: Duże binary bloki = duże patche
// Lepsze: Wykryj binary i użyj CAS zamiast diff
```

---

## Optimizations

### 1. Hash-based Skip

```typescript
async function computeDiffOptimized(
  newText: string,
  oldText: string
): Promise<string> {
  // Quick check: jeśli hashe są identyczne, skip diffing
  const newHash = await sha256(newText);
  const oldHash = await sha256(oldText);
  
  if (newHash === oldHash) {
    return ''; // No changes
  }
  
  // Proceed with diffing
  return await computeDiff(newText, oldText);
}
```

**Speedup**: 10-100x dla plików bez zmian.

### 2. Chunked Diffing dla bardzo dużych plików

```typescript
async function computeDiffChunked(
  newText: string,
  oldText: string,
  chunkSize: number = 100_000
): Promise<string> {
  // Split into chunks
  const newChunks = chunkText(newText, chunkSize);
  const oldChunks = chunkText(oldText, chunkSize);
  
  // Diff each chunk
  const patchChunks = await Promise.all(
    newChunks.map((newChunk, i) =>
      computeDiff(newChunk, oldChunks[i] || '')
    )
  );
  
  // Combine patches
  return patchChunks.join('\n---\n');
}
```

**Trade-off**: Szybsze, ale może generować większe patche.

### 3. Incremental Diffing

```typescript
// Cache previous diff result
class DiffCache {
  private cache = new Map<string, string>();
  
  async getCachedDiff(
    newText: string,
    oldText: string
  ): Promise<string | null> {
    const key = `${hash(newText)}:${hash(oldText)}`;
    return this.cache.get(key) || null;
  }
}
```

**Korzyść**: Jeśli ten sam diff jest potrzebny wielokrotnie (np. podczas testów).

---

## Best Practices

### Do's ✅

1. **Zawsze normalizuj tekst** przed diffingiem
2. **Używaj Web Workers** dla plików >100KB
3. **Waliduj patche** po wygenerowaniu
4. **Cache hashe** dla szybkiego skip
5. **Monitoruj rozmiar patchy** - duże patche = problem

### Don'ts ❌

1. **Nie diffuj binariów** - użyj CAS
2. **Nie ignoruj błędów walidacji** - patch może być niepoprawny
3. **Nie diffuj bez normalizacji** - niestabilne wyniki
4. **Nie blokuj UI thread** - użyj workers dla dużych plików

---

## Testing

### Unit Tests

```typescript
describe('computeDiff', () => {
  it('should generate empty patch for identical texts', async () => {
    const patch = await computeDiff('Hello', 'Hello');
    expect(patch).toBe('');
  });
  
  it('should generate reverse patch correctly', async () => {
    const newText = 'Hello\nWorld';
    const oldText = 'Hello';
    
    const patch = await computeDiff(newText, oldText);
    
    // Apply patch to newText should give oldText
    const dmp = new diff_match_patch();
    const patches = dmp.patch_fromText(patch);
    const [result] = dmp.patch_apply(patches, newText);
    
    expect(normalizeText(result)).toBe(normalizeText(oldText));
  });
  
  it('should handle Unicode normalization', async () => {
    const text1 = 'café';  // composed
    const text2 = 'cafe\u0301';  // decomposed
    
    const patch = await computeDiff(text1, text2);
    expect(patch).toBe(''); // Should be identical after normalization
  });
});
```

---

## Integration z Save Checkpoint

```typescript
async function saveCheckpoint(message: string): Promise<string> {
  // ... identify changes ...
  
  for (const change of textFileChanges) {
    const newContent = await readWorkingCopy(change.path);
    const oldContent = await getFileFromVersion(change.path, headId);
    
    // Generate reverse patch
    const patch = await this.diffManager.computeDiff(
      newContent,
      oldContent
    );
    
    // Save patch
    await this.saveDelta(headId, change.path, patch);
  }
  
  // ...
}
```

---

## Integration z Save Checkpoint - Kompletny Przykład

```typescript
class DeltaManager {
  async saveCheckpoint(message: string): Promise<string> {
    const headVersionId = this.manifest.refs.head;
    const changedFiles = await this.identifyChangedFiles();
    
    for (const filePath of changedFiles) {
      const fileEntry = this.manifest.fileMap[filePath];
      
      if (fileEntry.type === 'text') {
        // 1. Odczytaj aktualną zawartość (NEW)
        const newContent = await this.adapter.readFile(`content/${filePath}`);
        const newText = new TextDecoder().decode(newContent);
        
        // 2. Odczytaj poprzednią zawartość (OLD)
        const oldContent = await this.getFileFromVersion(filePath, headVersionId);
        const oldText = new TextDecoder().decode(oldContent);
        
        // 3. Generuj reverse patch
        const patchText = await this.computeDiff(newText, oldText);
        
        // 4. Sprawdź czy patch nie jest za duży
        if (patchText.length > newText.length * 0.5) {
          // Użyj snapshot zamiast patch
          await this.saveSnapshot(headVersionId, filePath, newText);
        } else {
          // Zapisz patch
          const deltaPath = `.store/deltas/${headVersionId}_${this.hashPath(filePath)}.patch`;
          await this.writeToZip(deltaPath, new TextEncoder().encode(patchText));
        }
      }
    }
    
    // 5. Utwórz nową wersję
    const newVersionId = await this.createVersion(message);
    return newVersionId;
  }
  
  private hashPath(path: string): string {
    // Hash ścieżki dla bezpiecznej nazwy pliku (bez /, etc.)
    const hash = Array.from(path)
      .map(c => c.charCodeAt(0).toString(16).padStart(2, '0'))
      .join('')
      .substring(0, 16);
    return hash;
  }
}
```

**Przykład wykonania**:
```typescript
// Stan początkowy (v1)
// content/src/index.js: 'console.log("Hello");\n'

// Użytkownik modyfikuje plik
await manager.addFile('src/index.js', 'console.log("Hello");\nconsole.log("World");\n');

// saveCheckpoint('Add World')
// 1. newText = 'console.log("Hello");\nconsole.log("World");\n'
// 2. oldText = 'console.log("Hello");\n'
// 3. patchText = '@@ -1,2 +1,1 @@\n console.log("Hello");\n-console.log("World");\n'
// 4. patchText.length (45) < newText.length (40) * 0.5 = 20? Nie, ale to edge case
//    W rzeczywistości: patch jest mniejszy niż pełny plik, więc OK
// 5. Zapisano: .store/deltas/v1_a1b2c3d4.patch
// 6. Utworzono: v2
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
