# Wizualizacja Architektury Kamaros

> **Diagramy Mermaid przedstawiające strukturę projektu, przepływy danych i zależności modułów**

---

## 1. Struktura Projektu (Drzewo plików)

```mermaid
graph TD
    subgraph "kamaros/"
        ROOT[kamaros/]
        
        subgraph CORE["core/ (Rust)"]
            CORE_SRC[src/]
            LIB_RS[lib.rs]
            JCF_RS[jcf.rs]
            VERSION_RS[versioning.rs]
            DIFF_RS[diff.rs]
            HASH_RS[hash.rs]
            ZIP_RS[zip.rs]
            MANIFEST_RS[manifest.rs]
            CAS_RS[cas.rs]
            CARGO[Cargo.toml]
        end
        
        subgraph JS["js/ (TypeScript)"]
            JS_SRC[src/]
            INDEX_TS[index.ts]
            JCFM_TS[JCFManager.ts]
            ADAPTERS[adapters/]
            BROWSER_AD[BrowserAdapter.ts]
            NODE_AD[NodeAdapter.ts]
            TAURI_AD[TauriAdapter.ts]
            CORE_WRAP[core/]
            VER_MGR[VersionManager.ts]
            FILE_MGR[FileManager.ts]
            DELTA_MGR[DeltaManager.ts]
            TYPES_TS[types.ts]
        end
        
        subgraph PY["python/ (PyO3)"]
            PY_SRC[src/]
            PY_INIT[__init__.py]
            PY_KAMAROS[kamaros/]
        end
    end
    
    ROOT --> CORE
    ROOT --> JS
    ROOT --> PY
    CORE_SRC --> LIB_RS
    CORE_SRC --> JCF_RS
    CORE_SRC --> VERSION_RS
    CORE_SRC --> DIFF_RS
    CORE_SRC --> HASH_RS
    CORE_SRC --> ZIP_RS
    CORE_SRC --> MANIFEST_RS
    CORE_SRC --> CAS_RS
```

---

## 2. Graf Zależności Modułów (Rust Core)

```mermaid
graph TB
    subgraph "Public API"
        LIB[lib.rs<br/>Main exports]
    end
    
    subgraph "Core Modules"
        JCF[jcf.rs<br/>JCF Format]
        VER[versioning.rs<br/>Time-Travel]
        MAN[manifest.rs<br/>JSON Manifest]
    end
    
    subgraph "Storage"
        CAS[cas.rs<br/>Content Addressing]
        ZIP[zip.rs<br/>Compression]
    end
    
    subgraph "Algorithms"
        DIFF[diff.rs<br/>Myers Diff]
        HASH[hash.rs<br/>SHA-256]
    end
    
    LIB --> JCF
    LIB --> VER
    LIB --> MAN
    
    JCF --> VER
    JCF --> MAN
    JCF --> CAS
    JCF --> ZIP
    
    VER --> DIFF
    VER --> HASH
    VER --> MAN
    
    CAS --> HASH
    CAS --> ZIP
    
    MAN --> HASH
    
    style LIB fill:#4CAF50,color:#fff
    style JCF fill:#2196F3,color:#fff
    style VER fill:#2196F3,color:#fff
    style MAN fill:#2196F3,color:#fff
    style CAS fill:#FF9800,color:#fff
    style ZIP fill:#FF9800,color:#fff
    style DIFF fill:#9C27B0,color:#fff
    style HASH fill:#9C27B0,color:#fff
```

---

## 3. Architektura Warstwowa

```mermaid
graph TB
    subgraph "User Application"
        APP[Browser / Node.js / Tauri / Python]
    end
    
    subgraph "API Layer"
        JCFM[JCFManager<br/>Public Interface]
        CONFIG[JCFConfig]
    end
    
    subgraph "Core Layer"
        VM[VersionManager<br/>Historia commitów]
        FM[FileManager<br/>CRUD operacje]
        DM[DeltaManager<br/>Diff/Patch]
        BM[BlobManager<br/>CAS binaries]
    end
    
    subgraph "Storage Layer"
        FSA[FileSystemAdapter<br/>Interface]
        BA[BrowserAdapter<br/>IndexedDB]
        NA[NodeAdapter<br/>fs/promises]
        TA[TauriAdapter<br/>tauri.fs]
    end
    
    subgraph "Worker Layer"
        HW[HashWorker<br/>SHA-256]
        DW[DiffWorker<br/>Text diff]
        CW[CompressWorker<br/>ZIP]
    end
    
    APP --> JCFM
    JCFM --> VM
    JCFM --> FM
    JCFM --> DM
    JCFM --> BM
    
    VM --> FSA
    FM --> FSA
    DM --> FSA
    BM --> FSA
    
    FSA --> BA
    FSA --> NA
    FSA --> TA
    
    DM --> DW
    BM --> HW
    FM --> CW
    
    style APP fill:#E91E63,color:#fff
    style JCFM fill:#4CAF50,color:#fff
    style VM fill:#2196F3,color:#fff
    style FM fill:#2196F3,color:#fff
    style DM fill:#2196F3,color:#fff
    style BM fill:#2196F3,color:#fff
    style FSA fill:#FF9800,color:#fff
    style HW fill:#9C27B0,color:#fff
    style DW fill:#9C27B0,color:#fff
    style CW fill:#9C27B0,color:#fff
```

---

## 4. Przepływ Danych: Save Checkpoint

```mermaid
sequenceDiagram
    participant User
    participant JCFManager
    participant VersionManager
    participant DeltaManager
    participant BlobManager
    participant FileSystem
    
    User->>JCFManager: saveCheckpoint("message")
    JCFManager->>VersionManager: identifyChangedFiles()
    VersionManager-->>JCFManager: changedFiles[]
    
    loop For each text file
        JCFManager->>DeltaManager: generateReversePatch(new, old)
        DeltaManager-->>JCFManager: patch
        JCFManager->>FileSystem: write(.store/deltas/...)
    end
    
    loop For each binary file
        JCFManager->>BlobManager: hashAndStore(content)
        BlobManager->>BlobManager: SHA-256(content)
        BlobManager->>FileSystem: write(.store/blobs/hash)
        BlobManager-->>JCFManager: hash
    end
    
    JCFManager->>VersionManager: createVersion(fileStates)
    JCFManager->>FileSystem: updateManifest()
    JCFManager->>FileSystem: writeZIP()
    JCFManager-->>User: versionId
```

---

## 5. Przepływ Danych: Restore Version

```mermaid
sequenceDiagram
    participant User
    participant JCFManager
    participant VersionManager
    participant DeltaManager
    participant BlobManager
    participant FileSystem
    
    User->>JCFManager: restoreVersion("v5")
    JCFManager->>VersionManager: buildVersionPath(HEAD, v5)
    VersionManager-->>JCFManager: [v10, v9, v8, v7, v6, v5]
    
    loop For each text file
        JCFManager->>FileSystem: read(/content/file.txt)
        loop For each version step
            JCFManager->>FileSystem: read(.store/deltas/patch)
            JCFManager->>DeltaManager: applyPatch(content, patch)
            DeltaManager-->>JCFManager: previousContent
        end
        JCFManager->>FileSystem: write(/content/file.txt)
    end
    
    loop For each binary file
        JCFManager->>VersionManager: getFileHash(v5, file)
        VersionManager-->>JCFManager: hash
        JCFManager->>BlobManager: loadBlob(hash)
        BlobManager->>FileSystem: read(.store/blobs/hash)
        BlobManager-->>JCFManager: content
        JCFManager->>FileSystem: write(/content/file)
    end
    
    JCFManager->>FileSystem: updateHEADref(v5)
    JCFManager-->>User: success
```

---

## 6. Struktura Pliku JCF (ZIP)

```mermaid
graph TD
    subgraph "project.jcf (ZIP Archive)"
        MT[mimetype<br/>"application/x-jcf"]
        MF[manifest.json<br/>Metadane + Historia]
        
        subgraph CONTENT["/content/ (HEAD)"]
            C1[src/index.js]
            C2[src/styles.css]
            C3[assets/logo.png]
        end
        
        subgraph STORE["/.store/ (Historia)"]
            subgraph BLOBS["/blobs/ (CAS)"]
                B1[a3f5e8d9...]
                B2[9d4c1e2b...]
            end
            
            subgraph DELTAS["/deltas/"]
                D1[v5_src_index.js.patch]
                D2[v4_src_index.js.patch]
            end
        end
    end
    
    style MT fill:#4CAF50,color:#fff
    style MF fill:#2196F3,color:#fff
    style CONTENT fill:#E8F5E9
    style STORE fill:#FFF3E0
    style BLOBS fill:#FFECB3
    style DELTAS fill:#FFECB3
```

---

## 7. Główne Klasy i Interfejsy

```mermaid
classDiagram
    class JCFManager {
        -adapter: FileSystemAdapter
        -versionManager: VersionManager
        -fileManager: FileManager
        +init() Promise~void~
        +saveCheckpoint(msg) Promise~string~
        +restoreVersion(id) Promise~void~
        +addFile(path, content) Promise~void~
        +getFile(path) Promise~Uint8Array~
        +runGC() Promise~GCReport~
    }
    
    class VersionManager {
        -manifest: Manifest
        -graph: VersionGraph
        +createVersion(states) Version
        +getVersion(id) Version
        +buildPath(from, to) string[]
        +findOrphans() string[]
    }
    
    class FileManager {
        -adapter: FileSystemAdapter
        +readFile(path) Promise~Uint8Array~
        +writeFile(path, data) Promise~void~
        +deleteFile(path) Promise~void~
        +listFiles() Promise~string[]~
    }
    
    class DeltaManager {
        +generatePatch(new, old) string
        +applyPatch(content, patch) string
        +normalizeText(text) string
    }
    
    class BlobManager {
        -cache: LRUCache
        +hashContent(data) string
        +storeBlob(data) Promise~string~
        +loadBlob(hash) Promise~Uint8Array~
        +blobExists(hash) Promise~boolean~
    }
    
    class FileSystemAdapter {
        <<interface>>
        +readFile(path) Promise~Uint8Array~
        +writeFile(path, data) Promise~void~
        +fileExists(path) Promise~boolean~
        +deleteFile(path) Promise~void~
    }
    
    JCFManager --> VersionManager
    JCFManager --> FileManager
    JCFManager --> DeltaManager
    JCFManager --> BlobManager
    JCFManager --> FileSystemAdapter
    FileManager --> FileSystemAdapter
    BlobManager --> FileSystemAdapter
```

---

## 8. Strategia Reverse Delta

```mermaid
graph LR
    subgraph "Forward Delta (Git)"
        FV1[v1 FULL] --> FP1[patch] --> FV2[v2] --> FP2[patch] --> FV3[v3 HEAD]
    end
    
    subgraph "Reverse Delta (JCF)"
        RV1[v1] --> RP1[patch] --> RV2[v2] --> RP2[patch] --> RV3[v3 HEAD FULL]
    end
    
    style FV1 fill:#4CAF50,color:#fff
    style FV3 fill:#F44336,color:#fff
    style RV1 fill:#F44336,color:#fff
    style RV3 fill:#4CAF50,color:#fff
```

**Legenda:**
- 🟢 Szybki dostęp (pełna zawartość)
- 🔴 Wolny dostęp (wymaga rekonstrukcji)

---

## 9. Content Addressable Storage (CAS)

```mermaid
graph TD
    subgraph "Pliki użytkownika"
        F1[logo.png v1]
        F2[logo.png v2]
        F3[logo.png v3<br/>= v1]
    end
    
    subgraph ".store/blobs/"
        B1[abc123<br/>50KB]
        B2[def456<br/>60KB]
    end
    
    F1 -->|hash=abc123| B1
    F2 -->|hash=def456| B2
    F3 -->|hash=abc123| B1
    
    style B1 fill:#4CAF50,color:#fff
    style B2 fill:#2196F3,color:#fff
```

**Deduplikacja:** v1 i v3 wskazują na ten sam blob!

---

## 10. FFI Bindings (Rust → Languages)

```mermaid
graph TB
    subgraph "Rust Core"
        RC[kamaros-core<br/>lib.rs]
    end
    
    subgraph "WASM (Browser/Node)"
        WB[wasm-bindgen]
        WP[wasm-pack]
        WASM[kamaros.wasm<br/>~600KB]
    end
    
    subgraph "PyO3 (Python)"
        PYO[PyO3]
        WHEEL[kamaros.so<br/>~800KB]
    end
    
    subgraph "TypeScript Wrapper"
        TSW[JCFManager.ts]
    end
    
    subgraph "Python Wrapper"
        PYW[kamaros/__init__.py]
    end
    
    RC --> WB --> WP --> WASM
    RC --> PYO --> WHEEL
    
    WASM --> TSW
    WHEEL --> PYW
    
    style RC fill:#FF5722,color:#fff
    style WASM fill:#2196F3,color:#fff
    style WHEEL fill:#4CAF50,color:#fff
```

---

## Powiązane dokumenty

- [System Overview](01-system-overview.md) - Opis architektury
- [Design Patterns](05-design-patterns.md) - Wzorce projektowe
- [Data Structures](06-data-structures.md) - Struktury danych
