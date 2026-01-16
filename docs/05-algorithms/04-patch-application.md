# Patch Application

> **Szczegółowy algorytm aplikowania reverse delta patches podczas restore version**

[← Back: Diff Generation](03-diff-generation.md) | [Next: Garbage Collection →](05-garbage-collection.md)

---

## Overview

Algorytm `patch-application` odpowiada za aplikowanie reverse delta patches podczas operacji `restoreVersion`. Patche są aplikowane **wstecz** (od HEAD do starszej wersji), co jest kluczowe dla Reverse Delta Strategy.

**Kluczowe założenie**: Aplikujemy reverse patches (NEW → OLD), więc zaczynamy od HEAD i cofamy się w czasie.

---

## Algorithm: Step-by-Step

```
INPUT:
  - currentText: string    // Aktualna zawartość (HEAD lub intermediate)
  - patchText: string       // Serialized patch (reverse: NEW → OLD)

OUTPUT:
  - oldText: string         // Zrekonstruowana starsza wersja

PRECONDITION:
  - currentText jest znormalizowany
  - patchText jest poprawnym formatem diff-match-patch
  - Patch reprezentuje transformację: currentText → oldText

POSTCONDITION:
  - oldText jest znormalizowany
  - oldText reprezentuje poprzednią wersję
```

### Step 1: Parse Patch

```typescript
import { diff_match_patch } from 'diff-match-patch';

function parsePatch(patchText: string): Patch[] {
  const dmp = new diff_match_patch();
  
  // Parse text format to Patch objects
  const patches = dmp.patch_fromText(patchText);
  
  if (!patches || patches.length === 0) {
    throw new InvalidPatchError('Empty or invalid patch');
  }
  
  return patches;
}
```

**Format patch**:
```
@@ -start,count +start,count @@
 kontekst
-linia do usunięcia
+linia do dodania
 kontekst
```

**Konkretny przykład parsowania**:
```typescript
const patchText = `@@ -1,3 +1,2 @@
 console.log('Hello');
-console.log('World');
 console.log('End');
`;

const dmp = new diff_match_patch();
const patches = dmp.patch_fromText(patchText);

// Wynik:
// patches = [
//   {
//     diffs: [
//       [0, "console.log('Hello');\n"],      // bez zmian (kontekst)
//       [-1, "console.log('World');\n"],     // usuń tę linię
//       [0, "console.log('End');\n"]         // bez zmian (kontekst)
//     ],
//     start1: 1,   // Pozycja w starym tekście (1-based)
//     start2: 1,   // Pozycja w nowym tekście (1-based)
//     length1: 3,  // Długość starego fragmentu (3 linie)
//     length2: 2   // Długość nowego fragmentu (2 linie)
//   }
// ]
```

### Step 2: Normalize Current Text

```typescript
function normalizeText(text: string): string {
  // Identyczna normalizacja jak w diff-generation
  text = text.replace(/\r\n/g, '\n');
  text = text.replace(/\r/g, '\n');
  text = text.normalize('NFC');
  
  if (!text.endsWith('\n') && text.length > 0) {
    text += '\n';
  }
  
  return text;
}
```

**Ważne**: Normalizacja musi być identyczna jak przy generowaniu patchy, inaczej aplikacja się nie powiedzie.

**Przykład problemu z różną normalizacją**:
```typescript
// Podczas generowania patchy (saveCheckpoint):
const newText = 'Hello\r\nWorld';  // Windows line endings
const oldText = 'Hello';
const patch = await computeDiff(newText, oldText);
// patch używa znormalizowanego tekstu (LF)

// Podczas aplikowania patchy (restoreVersion):
const currentText = 'Hello\r\nWorld';  // Windows line endings (nie znormalizowane!)
const result = await applyPatch(currentText, patch);
// ❌ Błąd! Patch nie może się zastosować bo line endings się nie zgadzają

// Poprawne:
const normalizedCurrent = normalizeText(currentText);  // 'Hello\nWorld'
const result = await applyPatch(normalizedCurrent, patch);
// ✅ Działa! Oba teksty używały tej samej normalizacji
```

### Step 3: Apply Patch

```typescript
async function applyPatch(
  currentText: string,
  patchText: string
): Promise<string> {
  // Normalize
  const normalizedCurrent = normalizeText(currentText);
  
  // Parse
  const dmp = new diff_match_patch();
  const patches = dmp.patch_fromText(patchText);
  
  // Configure for fuzzy matching
  dmp.Match_Distance = 1000;
  dmp.Patch_DeleteThreshold = 0.5;
  
  // Apply patches
  const [resultText, successArray] = dmp.patch_apply(
    patches,
    normalizedCurrent
  );
  
  // Check if all patches applied successfully
  const allSuccess = successArray.every(s => s === true);
  
  if (!allSuccess) {
    // Try fuzzy matching fallback
    return await applyPatchWithFuzzy(
      normalizedCurrent,
      patches,
      successArray
    );
  }
  
  return normalizeText(resultText);
}
```

**Return value**:
- `resultText`: Zrekonstruowany tekst
- `successArray`: Tablica boolean - czy każdy patch się powiódł

### Step 4: Fuzzy Matching Fallback

```typescript
async function applyPatchWithFuzzy(
  currentText: string,
  patches: Patch[],
  successArray: boolean[]
): Promise<string> {
  const failedCount = successArray.filter(s => !s).length;
  
  console.warn(
    `⚠️  ${failedCount}/${patches.length} patches failed, ` +
    `attempting fuzzy matching`
  );
  
  // Increase fuzzy matching tolerance
  const dmp = new diff_match_patch();
  dmp.Match_Distance = 2000;  // Większy zakres
  dmp.Match_Threshold = 0.3;   // Niższy próg (bardziej tolerancyjny)
  dmp.Patch_DeleteThreshold = 0.3;
  
  // Retry with more aggressive fuzzy matching
  const [resultText, retrySuccess] = dmp.patch_apply(
    patches,
    currentText
  );
  
  const retryFailedCount = retrySuccess.filter(s => !s).length;
  
  if (retryFailedCount > 0) {
    // Still failed - try snapshot fallback
    console.error(
      `❌ ${retryFailedCount} patches still failed after fuzzy matching`
    );
    
    return await applyPatchWithSnapshotFallback(
      currentText,
      patches
    );
  }
  
  console.log('✅ Fuzzy matching succeeded');
  return normalizeText(resultText);
}
```

**Kiedy fuzzy matching pomaga?**:
- Plik został zmodyfikowany po utworzeniu patchy
- Kontekst wokół zmian się zmienił
- Line endings różnią się (choć powinny być znormalizowane)

**Konkretny przykład fuzzy matching**:
```typescript
// Oryginalny patch został wygenerowany dla:
const originalText = 'line1\nline2\nline3\nCHANGE_ME\nline5\n';
const targetText = 'line1\nline2\nline3\nCHANGED\nline5\n';
const patch = await computeDiff(originalText, targetText);
// patch: usuń 'CHANGE_ME', dodaj 'CHANGED'

// Ale plik został dodatkowo zmodyfikowany:
const modifiedText = 'line1\nline2\nEXTRA_LINE\nline3\nCHANGE_ME\nline5\n';
// Dodano 'EXTRA_LINE' przed linią 3

// Bez fuzzy matching:
const [result1, success1] = dmp.patch_apply(patches, modifiedText);
// success1 = [false] ❌ - patch nie może się zastosować (pozycja się zmieniła)

// Z fuzzy matching (Match_Distance = 1000):
dmp.Match_Distance = 1000;  // Szukaj w zakresie ±1000 znaków
const [result2, success2] = dmp.patch_apply(patches, modifiedText);
// success2 = [true] ✅ - fuzzy matching znalazł 'CHANGE_ME' pomimo przesunięcia
// result2 = 'line1\nline2\nEXTRA_LINE\nline3\nCHANGED\nline5\n'
```

### Step 5: Snapshot Fallback

```typescript
async function applyPatchWithSnapshotFallback(
  currentText: string,
  patches: Patch[]
): Promise<string> {
  // Jeśli dostępny snapshot dla tej wersji, użyj go
  const snapshotVersion = await this.findNearestSnapshot(patches);
  
  if (snapshotVersion) {
    console.log(`📸 Using snapshot fallback for version ${snapshotVersion}`);
    return await this.loadFromSnapshot(snapshotVersion);
  }
  
  // Last resort: Return partially applied result
  // (może być niepoprawny, ale lepsze niż crash)
  console.error(
    '⚠️  Patch application failed - returning partial result. ' +
    'Data may be inconsistent.'
  );
  
  const dmp = new diff_match_patch();
  const [resultText] = dmp.patch_apply(patches, currentText);
  return normalizeText(resultText);
}
```

**Kiedy snapshot fallback?**:
- Wszystkie metody aplikacji patchy zawiodły
- Dostępny jest snapshot dla tej wersji
- Lepsze niż zwrócenie błędnych danych

---

## Complete Algorithm: Restore File

```typescript
async function restoreFile(
  filePath: string,
  targetVersionId: string
): Promise<string> {
  // Start with HEAD content
  let content = await this.readWorkingCopy(filePath);
  let text = new TextDecoder().decode(content);
  
  // Build version path: HEAD → target
  const versionPath = this.buildVersionPath(
    this.manifest.refs.head,
    targetVersionId
  );
  
  // Apply patches backwards through path
  for (let i = 0; i < versionPath.length - 1; i++) {
    const fromVersion = versionPath[i];
    const toVersion = versionPath[i + 1];
    
    // Load reverse patch: from → to
    const deltaPath = `.store/deltas/${fromVersion}_${hashPath(filePath)}.patch`;
    
    if (await this.fileExistsInZip(deltaPath)) {
      const patchData = await this.readFromZip(deltaPath);
      const patchText = new TextDecoder().decode(patchData);
      
      // Apply patch
      text = await this.applyPatch(text, patchText);
      
      this.emit('restore:progress', {
        file: filePath,
        fromVersion,
        toVersion,
        percent: ((i + 1) / (versionPath.length - 1)) * 100
      });
    } else {
      // Patch missing - try snapshot or error
      console.warn(`Patch missing for ${filePath} at ${fromVersion}`);
      text = await this.fallbackReconstruction(filePath, toVersion);
    }
  }
  
  return text;
}
```

---

## Error Handling

### Case 1: Patch Not Found

```typescript
if (!await this.fileExistsInZip(deltaPath)) {
  // Try to find patch in alternative location
  const altPath = await this.findAlternativePatchPath(
    fromVersion,
    filePath
  );
  
  if (altPath) {
    patchText = await this.readFromZip(altPath);
  } else {
    // No patch available - use snapshot or error
    throw new PatchNotFoundError(
      `Patch not found: ${deltaPath}`,
      { fromVersion, filePath }
    );
  }
}
```

### Case 2: Patch Application Failed

```typescript
try {
  text = await this.applyPatch(text, patchText);
} catch (error) {
  if (error instanceof PatchApplicationError) {
    // Try fuzzy matching
    text = await this.applyPatchWithFuzzy(text, patchText);
  } else {
    // Unexpected error
    throw error;
  }
}
```

### Case 3: Corruption Detection

```typescript
// After applying patch, verify result
const expectedHash = targetVersion.fileStates[filePath].hash;

if (expectedHash) {
  const actualHash = await sha256(text);
  
  if (actualHash !== expectedHash) {
    console.warn(
      `⚠️  Hash mismatch for ${filePath} after patch application. ` +
      `Expected: ${expectedHash}, Got: ${actualHash}`
    );
    
    // Try snapshot fallback
    text = await this.loadFromSnapshot(targetVersionId);
  }
}
```

---

## Performance Optimizations

### 1. Batch Patch Loading

```typescript
async function restoreFileOptimized(
  filePath: string,
  targetVersionId: string
): Promise<string> {
  const versionPath = this.buildVersionPath(
    this.manifest.refs.head,
    targetVersionId
  );
  
  // Load all patches upfront (if small)
  const patches = await Promise.all(
    versionPath.slice(0, -1).map(async (fromVersion) => {
      const deltaPath = `.store/deltas/${fromVersion}_${hashPath(filePath)}.patch`;
      const patchData = await this.readFromZip(deltaPath);
      return new TextDecoder().decode(patchData);
    })
  );
  
  // Apply all patches sequentially
  let text = await this.readWorkingCopy(filePath);
  for (const patchText of patches) {
    text = await this.applyPatch(text, patchText);
  }
  
  return text;
}
```

**Trade-off**: Szybsze dla małych plików, ale więcej pamięci.

### 2. Streaming Patch Application

```typescript
async function restoreFileStreaming(
  filePath: string,
  targetVersionId: string
): Promise<ReadableStream> {
  // Stream patches one by one (dla bardzo dużych plików)
  const versionPath = this.buildVersionPath(
    this.manifest.refs.head,
    targetVersionId
  );
  
  let text = await this.readWorkingCopy(filePath);
  
  for (const fromVersion of versionPath.slice(0, -1)) {
    const patchStream = await this.readFromZipStream(
      `.store/deltas/${fromVersion}_${hashPath(filePath)}.patch`
    );
    
    const patchText = await streamToString(patchStream);
    text = await this.applyPatch(text, patchText);
    
    // Yield intermediate result (dla progress tracking)
    yield text;
  }
  
  return text;
}
```

### 3. Patch Caching

```typescript
class PatchCache {
  private cache = new LRUCache<string, string>({ max: 100 });
  
  async getCachedPatch(
    versionId: string,
    filePath: string
  ): Promise<string | null> {
    const key = `${versionId}:${filePath}`;
    return this.cache.get(key) || null;
  }
  
  async cachePatch(
    versionId: string,
    filePath: string,
    patchText: string
  ): Promise<void> {
    const key = `${versionId}:${filePath}`;
    this.cache.set(key, patchText);
  }
}
```

**Korzyść**: Jeśli ten sam patch jest używany wielokrotnie (np. podczas testów restore).

---

## Complexity Analysis

### Time Complexity

| Operation | Complexity | Note |
|-----------|------------|------|
| Parse patch | O(P) | P = patch size |
| Apply patch | O(N + P) | N = text length, P = patch size |
| Fuzzy matching | O(N + P + M) | M = match distance |
| **Total (single patch)** | **O(N + P)** | |
| **Total (k patches)** | **O(k × (N + P))** | k = number of versions back |

### Space Complexity

| Component | Space | Note |
|-----------|-------|------|
| Current text | O(N) | N = text length |
| Patch data | O(P) | P = patch size |
| Result text | O(N) | |
| **Total** | **O(N + P)** | |

---

## Edge Cases

### Case 1: Empty Patch

```typescript
const patchText = '';
const result = await applyPatch('Hello', '');
// Returns: 'Hello' (no change)
```

### Case 2: Patch for Different File

```typescript
// Problem: Patch został wygenerowany dla innego pliku
// (np. plik został zmieniony po utworzeniu patchy)

// Rozwiązanie: Fuzzy matching powinien to wykryć
// Jeśli nie - snapshot fallback
```

### Case 3: Conflicting Patches

```typescript
// Problem: Dwa patche modyfikują tę samą linię
// (rzadkie, ale możliwe przy concurrent edits)

// Rozwiązanie: diff-match-patch używa fuzzy matching
// Jeśli zawiedzie - snapshot fallback
```

### Case 4: Very Large Patches

```typescript
// Problem: Patch jest większy niż oryginalny plik
// (oznacza, że plik został całkowicie przepisany)

// Rozwiązanie: Rozważ snapshot zamiast patch
// (optymalizacja w saveCheckpoint)
```

---

## Best Practices

### Do's ✅

1. **Zawsze normalizuj tekst** przed aplikacją patchy
2. **Waliduj wynik** (hash check jeśli dostępny)
3. **Używaj fuzzy matching** jako fallback
4. **Loguj nieudane aplikacje** dla debugowania
5. **Używaj snapshot fallback** gdy patch zawodzi

### Don'ts ❌

1. **Nie ignoruj błędów aplikacji** - może prowadzić do corrupted data
2. **Nie aplikuj patchy bez normalizacji** - niestabilne wyniki
3. **Nie używaj patchy dla binariów** - użyj CAS
4. **Nie cache patchy bez limitów** - może wyczerpać pamięć

---

## Testing

### Unit Tests

```typescript
describe('applyPatch', () => {
  it('should apply empty patch correctly', async () => {
    const result = await applyPatch('Hello', '');
    expect(result).toBe('Hello');
  });
  
  it('should apply reverse patch correctly', async () => {
    const newText = 'Hello\nWorld';
    const oldText = 'Hello';
    
    // Generate patch
    const patch = await computeDiff(newText, oldText);
    
    // Apply patch
    const result = await applyPatch(newText, patch);
    
    expect(normalizeText(result)).toBe(normalizeText(oldText));
  });
  
  it('should handle fuzzy matching', async () => {
    const currentText = 'Hello\nModified\nWorld';
    const patch = '@@ -1,2 +1,1 @@\n Hello\n-Modified\n'; // Remove "Modified"
    
    // Apply with fuzzy matching
    const result = await applyPatch(currentText, patch);
    
    expect(result).toContain('Hello');
    expect(result).not.toContain('Modified');
  });
  
  it('should handle patch application failure gracefully', async () => {
    const currentText = 'Completely different';
    const patch = '@@ -1,1 +1,1 @@\n-Old\n+New\n'; // Patch for different text
    
    // Should use fuzzy matching or snapshot fallback
    const result = await applyPatch(currentText, patch);
    
    // Should not crash
    expect(result).toBeDefined();
  });
});
```

---

## Integration z Restore Version

```typescript
async function restoreVersion(targetVersionId: string): Promise<void> {
  // ... build version path ...
  
  for (const filePath of targetFiles) {
    if (isTextFile(filePath)) {
      // Reconstruct using patches
      const text = await this.restoreFile(filePath, targetVersionId);
      await this.writeToWorkingCopy(filePath, text);
    } else {
      // Binary: Load from CAS
      const content = await this.loadBlob(filePath, targetVersionId);
      await this.writeToWorkingCopy(filePath, content);
    }
  }
  
  // ...
}
```

---

## Integration z Restore Version - Kompletny Przykład

```typescript
class VersionManager {
  async restoreVersion(targetVersionId: string): Promise<void> {
    const currentVersionId = this.manifest.refs.head;
    
    // 1. Zbuduj ścieżkę wersji (HEAD → target)
    const versionPath = this.buildVersionPath(currentVersionId, targetVersionId);
    // Przykład: ['v10', 'v9', 'v8', 'v7', 'v6', 'v5']
    
    // 2. Dla każdego pliku w target version
    const targetVersion = this.getVersion(targetVersionId);
    for (const [filePath, fileState] of Object.entries(targetVersion.fileStates)) {
      const fileEntry = this.manifest.fileMap[filePath];
      
      if (fileEntry.type === 'text') {
        // 3. Zacznij od HEAD content
        let content = await this.adapter.readFile(`content/${filePath}`);
        let text = new TextDecoder().decode(content);
        
        // 4. Aplikuj patche wstecz przez version path
        for (let i = 0; i < versionPath.length - 1; i++) {
          const fromVersion = versionPath[i];      // v10
          const toVersion = versionPath[i + 1];    // v9
          
          // Załaduj reverse patch: v10 → v9
          const deltaPath = `.store/deltas/${fromVersion}_${this.hashPath(filePath)}.patch`;
          
          if (await this.fileExistsInZip(deltaPath)) {
            const patchData = await this.readFromZip(deltaPath);
            const patchText = new TextDecoder().decode(patchData);
            
            // Aplikuj patch
            text = await this.applyPatch(text, patchText);
            
            console.log(`Applied patch: ${fromVersion} → ${toVersion}`);
          }
        }
        
        // 5. Zapisz zrekonstruowany plik
        await this.adapter.writeFile(
          `content/${filePath}`,
          new TextEncoder().encode(text)
        );
      }
    }
    
    // 6. Zaktualizuj HEAD
    this.manifest.refs.head = targetVersionId;
    await this.writeManifest();
  }
}
```

**Przykład wykonania**:
```typescript
// Historia: v1 → v2 → v3 → v4 → v5 (HEAD)
// content/src/index.js w v5: 'console.log("Hello");\nconsole.log("World");\n'
// Chcemy przywrócić do v2: 'console.log("Hello");\n'

// restoreVersion('v2')
// 1. versionPath = ['v5', 'v4', 'v3', 'v2']
// 2. Dla src/index.js:
//    - Start: text = 'console.log("Hello");\nconsole.log("World");\n' (v5)
//    - Apply patch v5→v4: text = 'console.log("Hello");\n' (usuń "World")
//    - Apply patch v4→v3: text = 'console.log("Hello");\n' (bez zmian)
//    - Apply patch v3→v2: text = 'console.log("Hello");\n' (bez zmian)
// 3. Zapisano: content/src/index.js = 'console.log("Hello");\n'
// 4. HEAD = v2
```

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0
