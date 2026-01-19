# Specyfikacja formatu JCF

## Wprowadzenie

Format JCF (JSON Container Format) to format pliku oparty na ZIP, zaprojektowany dla przechowywania projektów z pełną historią wersji. Łączy prostotę ZIP z zaawansowanym systemem wersjonowania.

**Biblioteka JCF Manager** działa według specyfikacji formatu:
- **Tworzy lub wczytuje specyfikację** (`manifest.json`) - definicję struktury projektu
- **Wczytuje pliki zgodnie ze specyfikacją** - odczytuje i rekonstruuje z formatu JCF
- **Zapisuje pliki zgodnie ze specyfikacją** - zapisuje do formatu JCF z wersjonowaniem
- **Waliduje zgodność ze specyfikacją** - sprawdza integralność i poprawność danych
- **Wersjonuje zgodnie ze specyfikacją** - zarządza historią zmian zgodnie z formatem

**Ważne**: Biblioteka nie interpretuje zawartości plików - przechowuje dowolne pliki (`.js`, `.glsl`, `.json`, `.png`, `.fbx`, etc.) zgodnie ze specyfikacją formatu JCF. To aplikacja decyduje, jakie pliki przechowuje i jak je interpretuje.

> 📖 **Przykłady**: Zobacz [examples/](./examples/) dla konkretnych przykładów plików specyfikacji:
> - [manifest-example.json](./examples/manifest-example.json) - Pełny przykład manifestu
> - [manifest-minimal.json](./examples/manifest-minimal.json) - Minimalny poprawny manifest
> - [format-specification.md](./examples/format-specification.md) - Szczegółowa specyfikacja z przykładami
> - [usage-examples.md](./examples/usage-examples.md) - Przykłady użycia w różnych scenariuszach

## Podstawowe założenia

### Dlaczego ZIP?

Zalety:
- Uniwersalny - każdy system operacyjny rozumie ZIP
- Streaming - można tworzyć i odczytywać bez ładowania całości do pamięci
- Kompresja - wbudowana deflate redukcja rozmiaru
- Recovery - standardowe narzędzia mogą odzyskać pliki nawet z uszkodzonego archiwum
- Ecosystem - tysiące bibliotek i narzędzi

Alternatywy odrzucone:
- TAR: Brak wbudowanej kompresji, trudniejszy streaming
- Custom binary: Zero tooling, trudne debugging
- SQLite: Overkill, wymaga parsera, nie human-readable

### Założenia projektowe

1. Human Recoverable: W najgorszym przypadku użytkownik może unzipnąć plik zwykłym archiverem
2. Self-Describing: manifest.json opisuje całą strukturę
3. Backward Compatible: Nowe wersje formatu mogą czytać stare pliki
4. Extensible: Pole extra w manifeście dla custom metadata

## Struktura ZIP

### Hierarchia plików

```
project.jcf (ZIP Archive)
│
├── mimetype                          [UNCOMPRESSED, FIRST]
│   └── "application/x-jcf"
│
├── manifest.json                     [COMPRESSED]
│   └── Metadata + wersje + fileMap
│
├── content/                          [WORKING COPY]
│   └── [dowolne pliki zgodnie ze specyfikacją]
│       └── (przykład: src/, assets/, config files, etc.)
│
└── .store/                           [HISTORY]
    ├── blobs/                        [Content Addressable Storage]
    │   ├── a3f5e8...2b1c (SHA-256)  [Old/alternate binary files]
    │   ├── 9d4c1e...7f3a
    │   └── ...
    │
    └── deltas/                       [Text Diffs]
        ├── v5_src_index.js.patch     [Reverse patch: v5 → v4]
        ├── v4_src_index.js.patch     [Reverse patch: v4 → v3]
        ├── v3_src_index.js.patch
        └── ...
```

### Szczegóły plików

#### mimetype

Wymagania:
- MUSI być pierwszym plikiem w ZIP
- MUSI być nieskompresowany (STORE method)
- MUSI zawierać dokładnie: application/x-jcf
- Brak newline na końcu

Cel: Umożliwia szybką identyfikację typu pliku bez parsowania całego ZIP.

Walidacja:
Sprawdzenie czy pierwszy wpis to "mimetype" uncompressed.

#### manifest.json

Struktura:

```typescript
interface Manifest {
  formatVersion: '1.0.0';

  metadata: {
    author: string;
    created_at: string;
    last_modified: string;
    application: string;
    description?: string;
    tags?: string[];
    extra?: Record<string, unknown>;
  };

  fileMap: Record<string, FileEntry>;

  versionHistory: Version[];

  refs: {
    head: string;
    [branchName: string]: string;
  };

  renameLog: RenameEntry[];

  config?: {
    autoGC?: boolean;
    compressionLevel?: number;
    maxHistorySize?: number;
  };
}
```

Typy pomocnicze:

```typescript
interface FileEntry {
  type: 'text' | 'binary';
  inodeId: string;
  currentHash?: string;
  encoding?: string;
  created_at?: string;
  modified_at?: string;
  size?: number;
  mime?: string;
  extra?: Record<string, unknown>;
}

interface Version {
  id: string;
  timestamp: string;
  message: string;
  author: string;
  email?: string;
  parentId: string | null;
  fileStates: Record<string, FileState>;
  tags?: string[];
  extra?: Record<string, unknown>;
}

interface FileState {
  inodeId: string;
  path: string;
  hash?: string;
  contentRef?: string;
  size: number;
  deleted?: boolean;
  changeType?: 'added' | 'modified' | 'deleted' | 'renamed';
}

interface RenameEntry {
  inodeId: string;
  fromPath: string;
  toPath: string;
  versionId: string;
  timestamp: string;
}
```

#### content/ - Working Copy

Przechowuje bieżący stan (HEAD) projektu w pełnej postaci.

Dlaczego ważne:
1. Recovery: Można unzipnąć standardowym narzędziem i odzyskać najnowszą wersję
2. Performance: Dostęp do HEAD nie wymaga rekonstrukcji z delta
3. Compatibility: Zewnętrzne narzędzia mogą czytać pliki bez znajomości JCF

#### .store/blobs/ - Content Addressable Storage

Deduklikacja plików binarnych poprzez content addressing.

Naming Convention: SHA-256(content)

#### .store/deltas/ - Text Diffs

Przechowywanie zmian tekstowych jako patche.

Format: Google diff-match-patch text format.

## Reverse Delta Strategy

### Koncepcja

Reverse Delta (JCF): Nowy→Stary
Zaleta: HEAD zawsze pełny, starsze wersje rekonstruowane wstecz

### Przykład

Stan początkowy (v1):
```javascript
console.log('Hello');
```

Zmiana (v2):
```javascript
console.log('Hello World');
```

Zapisane:
1. /content/src/index.js (v2, pełny)
2. /.store/deltas/v1_3a2f1b4e5c6d7e8f.patch

## Obsługa Warstw (Layers)

Format obsługuje koncepcję warstw, umożliwiając niezależne zarządzanie grupami treści.

### Niezależność warstw

- Do wybranej warstwy można wkładać treści (pliki) niezależnie od innych warstw.
- Warstwy pozwalają na logiczną separację danych (np. warstwa aplikacji, warstwa użytkownika, warstwa systemowa).
- Operacje na jednej warstwie nie muszą wpływać na strukturę lub zawartość innych warstw.

## Kompresja i optymalizacja

### Poziomy kompresji

```typescript
interface CompressionPolicy {
  manifest: 6;
  textFiles: 6;
  images: 0;
  videos: 0;
  archives: 0;
  deltas: 9;
}
```

### Strategia per typ pliku

Biblioteka rozpoznaje typy plików (tekst vs binary) **tylko dla celów wersjonowania**:
- **Pliki tekstowe** → reverse delta strategy (efektywne dla zmian)
- **Pliki binarne** → Content Addressable Storage (deduplikacja)

**To nie jest interpretacja zawartości** - biblioteka nie wie, co to jest `script.js` czy `shader.glsl`, tylko rozpoznaje, że to tekst i używa odpowiedniej strategii wersjonowania.

```typescript
function getCompressionLevel(filepath: string): number {
  const ext = getExtension(filepath);

  // Already compressed formats - nie kompresuj ponownie
  if (['.png', '.jpg', '.mp4', '.zip'].includes(ext)) {
    return 0; // STORE
  }

  // Text files - kompresuj dla efektywności
  if (['.js', '.ts', '.json', '.txt', '.md'].includes(ext)) {
    return 6; // DEFLATE
  }

  return 0;
}
```

## Walidacja i integrity

### Manifest Schema Validation

Użycie JSON Schema do walidacji struktury manifestu.

### Checksomy

- ZIP CRC (built-in)
- SHA-256 dla blobów
- Manifest checksum (opcjonalny)

## Migracje formatu

### Versioning Strategy

Semantic Versioning dla formatVersion:
- Major: Breaking changes
- Minor: Additive changes
- Patch: Bug fixes

### Migration System

```typescript
interface FormatMigration {
  from: string;
  to: string;
  migrate: (manifest: any) => Manifest;
}
```

## Best Practices

### Do's
- Zawsze waliduj manifest po odczycie
- Używaj streaming dla plików >10MB
- Uruchamiaj GC regularnie
- Twórz backup przed saveCheckpoint
- Atomic writes (temp file + rename)

### Don'ts
- Nie modyfikuj manifestu ręcznie
- Nie kompresuj już skompresowanych plików
- Nie ładuj całego ZIP do RAM
- Nie używaj długich commit messages
- Nie przechowuj secrets w manifeście