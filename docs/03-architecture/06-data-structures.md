# Struktury danych w Kamaros

Kamaros wykorzystuje specjalistyczne struktury danych zoptymalizowane pod konkretne przypadki użycia.

## Manifest - Centralna baza danych

### Struktura

```typescript
interface Manifest {
  formatVersion: string;
  metadata: ProjectMetadata;
  fileMap: Map<string, FileEntry>;
  versionHistory: Version[];
  refs: Map<string, string>;
  renameLog: RenameEntry[];
}

interface FileEntry {
  inodeId: string;
  type: 'text' | 'binary';
  currentHash?: string;
  created: string;
  modified: string;
}

interface Version {
  id: string;
  parentId: string | null;
  timestamp: string;
  message: string;
  author: string;
  fileStates: Map<string, FileState>;
}

interface FileState {
  inodeId: string;
  hash?: string;
  contentRef?: string;
  deleted?: boolean;
}
```

### Dlaczego Map zamiast Object?

- Gwarantowana kolejność iteracji
- Dowolne typy kluczy
- Wbudowana właściwość size
- Zoptymalizowana dla częstych operacji add/remove
- Szybsza dla >100 kluczy

## Blob Index - Metadane plików binarnych

### Struktura

```typescript
interface BlobIndex {
  [hash: string]: BlobMetadata;
}

interface BlobMetadata {
  mimeType: string;       // np. "image/png"
  originalName: string;   // np. "logo.png"
  size: number;           // bytes
  addedAt: string;        // ISO timestamp
  refCount: number;       // Reference counting dla GC
  tags?: string[];        // Optional tags
}
```

### Cel
Przechowywanie metadanych dla blobów (CAS) poza głównym plikiem `manifest.json`, aby zachować lekkość manifestu (Opcja C).


## Version Graph - DAG nawigacji wersji

### Struktura

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
 └─ v6 (orphaned - candidate for GC)
```

### Reprezentacja

```typescript
interface VersionNode {
  id: string;
  parentId: string | null;
  children: string[];
}

class VersionGraph {
  private nodes = new Map<string, VersionNode>();

  findPath(fromId: string, toId: string): string[] {
    // BFS dla znalezienia ścieżki wstecz
  }

  findCommonAncestor(id1: string, id2: string): string | null {
    // Dla przyszłego merge conflict resolution
  }

  findOrphans(headId: string): string[] {
    // Dla Garbage Collection
  }
}
```

### Złożoność

| Operation | Time | Space |
|-----------|------|-------|
| addVersion | O(1) | O(1) |
| findPath | O(V) | O(V) |
| findCommonAncestor | O(V) | O(V) |
| findOrphans | O(V) | O(V) |

## LRU Cache - dla blobów

### Problem
Frequently accessed blobs powinny być w pamięci, ale pamięć jest ograniczona.

### Rozwiązanie

```typescript
class LRUCache<K, V> {
  private cache = new Map<K, V>();
  private maxSize: number;

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
    this.cache.delete(key);
    this.cache.set(key, value);

    if (this.cache.size > this.maxSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
  }
}
```

### Dlaczego LRU?

- Temporal locality: często edytowane pliki używane wielokrotnie w krótkim czasie
- Simple O(1) operations
- Optymalne dla przypadków użycia systemu wersjonowania

### Złożoność

| Operation | Time | Space |
|-----------|------|-------|
| get | O(1) | - |
| set | O(1) | O(n) |
| eviction | O(1) | - |

## Set - dla Garbage Collection

### Problem
Garbage Collection potrzebuje szybkiej operacji "czy ten hash jest używany?"

### Rozwiązanie

```typescript
class GarbageCollector {
  async runGC(): Promise<GCReport> {
    // MARK phase: Collect used hashes
    const usedBlobs = new Set<string>();

    for (const version of this.manifest.versionHistory) {
      for (const [path, fileState] of version.fileStates) {
        if (fileState.hash) {
          usedBlobs.add(fileState.hash);
        }
      }
    }

    // SWEEP phase: Find orphans
    const allBlobs = await this.listAllBlobs();
    const orphanedBlobs = allBlobs.filter(hash => !usedBlobs.has(hash));

    // DELETE phase
    for (const hash of orphanedBlobs) {
      await this.deleteBlob(hash);
    }

    return {
      blobsRemoved: orphanedBlobs.length,
      spaceFreed: calculateSize(orphanedBlobs)
    };
  }
}
```

### Dlaczego Set zamiast Array?

| Operation | Array | Set | Winner |
|-----------|-------|-----|--------|
| .has(value) | O(n) | O(1) | Set |
| .add(value) | O(1) | O(1) | Tie |
| Duplicates | Allowed | Prevented | Set |
| Memory | Lower | Slightly higher | Array |

Decision: Set dla O(1) lookup - krytyczne dla GC z tysiącami hashów.

## Path Trie - optymalizacja wyszukiwania

### Problem
Szybkie wyszukiwanie plików po prefiksie (autocomplete, wildcard search).

### Status
⚠️ Optional Optimization - Not in v1.0 MVP

V1.0 używa prostego Map lookup (O(1) dla exact match). Trie przydatne dla:
- Autocomplete
- Wildcard search
- Directory listing

Można dodać później bez breaking changes.

## Podsumowanie struktur

| Structure | Purpose | Complexity | Status |
|-----------|---------|------------|--------|
| Manifest | Central database | O(1) lookup | ✅ v1.0 |
| Version Graph | Version navigation | O(V) traversal | ✅ v1.0 |
| LRU Cache | Blob caching | O(1) get/set | ✅ v1.0 |
| Set | GC mark phase | O(1) lookup | ✅ v1.0 |
| Path Trie | Prefix search | O(L) search | ⚠️ Future |

## Zasady projektowe

### Wybieraj strukturę danych wg wzorca dostępu
- Frequent random access → Map/Set (O(1))
- Sequential iteration → Array
- Prefix search → Trie

### Optymalizuj dla common case
- 95% operacji to dodawanie/odczyt plików → Map (O(1))
- GC rzadko, ale potrzebuje szybkiego lookup → Set

### Mierz przed optymalizacją
PathTrie nie w v1.0 bo:
- Map wystarczy dla MVP
- Trie dodaje complexity
- Można dodać później bez breaking changes