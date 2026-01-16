# Opowieść o JCF: Jak działa system wersjonowania Time-Travel

> **Narracyjne wyjaśnienie wszystkich technikaliów, procesów i schematów systemu JCF**

---

## Prolog: Problem, który rozwiązujemy

Wyobraź sobie, że pracujesz nad projektem - może to być gra, aplikacja webowa, czy jakikolwiek inny projekt z wieloma plikami. Chcesz mieć możliwość cofnięcia się w czasie do dowolnej wersji, ale jednocześnie nie chcesz, żeby plik projektu był ogromny. To jest dokładnie to, co rozwiązuje JCF (JSON Content Format).

JCF to inteligentny format pliku, który wygląda jak zwykły ZIP, ale w środku kryje się zaawansowany system wersjonowania. Możesz go otworzyć zwykłym unzipperem i zobaczyć aktualną wersję projektu, ale jeśli używasz biblioteki JCF Manager, możesz podróżować w czasie przez całą historię zmian.

---

## Rozdział 1: Struktura - Jak wygląda plik JCF?

### Wizualizacja: Co kryje się w pliku `.jcf`?

Gdy otwierasz plik `project.jcf`, widzisz strukturę podobną do zwykłego ZIP, ale z dodatkowymi elementami:

```
project.jcf (ZIP Archive)
│
├── mimetype                          [Pierwszy plik, nieskompresowany]
│   └── "application/x-jcf"           [Identyfikacja typu pliku]
│
├── manifest.json                     [Mózg całego systemu]
│   └── Zawiera:
│       - Metadane projektu (autor, data utworzenia)
│       - Mapę wszystkich plików (fileMap)
│       - Całą historię wersji (versionHistory)
│       - Referencje (refs.head wskazuje na aktualną wersję)
│
├── content/                          [Aktualny stan - HEAD]
│   └── src/
│       ├── index.js                  [Twoje pliki - zawsze aktualne]
│       ├── styles.css
│       └── logo.png
│
└── .store/                           [Ukryta historia - magazyn czasu]
    ├── blobs/                        [Content Addressable Storage]
    │   ├── a3f5e8d9c1b2e4f6...       [Pliki binarne - nazwa = hash]
    │   └── 9d4c1e2b3a4f5e6d7...
    │
    └── deltas/                       [Reverse delta patches]
        ├── v5_src_index.js.patch     [Patch: v5 → v4]
        ├── v4_src_index.js.patch     [Patch: v4 → v3]
        └── v3_src_index.js.patch     [Patch: v3 → v2]
```

### Dlaczego tak?

**`/content/`** to twoja "working copy" - zawsze zawiera najnowszą wersję. To oznacza, że możesz po prostu rozpakować plik JCF i od razu mieć dostęp do aktualnego stanu projektu. Nie musisz aplikować żadnych patchy - to jest kluczowa zaleta Reverse Delta Strategy.

**`.store/`** to magazyn historii. Tutaj przechowujemy:
- **`blobs/`**: Pliki binarne (obrazy, wideo, etc.) - każdy plik ma nazwę równą jego hash SHA-256
- **`deltas/`**: Patche dla plików tekstowych - każdy patch pokazuje, jak cofnąć się o jedną wersję wstecz

**`manifest.json`** to mózg systemu. Zawiera wszystkie informacje potrzebne do rekonstrukcji dowolnej wersji. To jest jak mapa skarbu - mówi ci, gdzie znaleźć każdy element historii.

---

## Rozdział 2: Reverse Delta Strategy - Filozofia systemu

### Historia trzech strategii

Wyobraź sobie, że masz 100 wersji projektu. Jak je przechowywać?

**Strategia 1: Forward Delta (jak Git)**
```
v1 (FULL) → [patch v1→v2] → v2 → [patch v2→v3] → ... → v100 (HEAD)
```

Aby dostać się do HEAD (v100), musisz:
1. Załadować v1 (pełny)
2. Zastosować patch v1→v2
3. Zastosować patch v2→v3
4. ... (99 razy)
5. W końcu masz v100

**Problem**: Im więcej wersji, tym wolniejszy dostęp do HEAD. Po 1000 commitach, każdy dostęp do najnowszej wersji wymaga aplikacji 999 patchy!

**Strategia 2: Full Snapshots**
```
v1 (FULL) | v2 (FULL) | v3 (FULL) | ... | v100 (FULL)
```

Każda wersja jest pełna. Dostęp do dowolnej wersji jest natychmiastowy.

**Problem**: Rozmiar pliku rośnie liniowo z liczbą wersji. 100 wersji = 100x rozmiar projektu. Dla projektu 50MB, to 5GB!

**Strategia 3: Reverse Delta (JCF) ⭐**
```
v1 ← [patch v2→v1] ← v2 ← [patch v3→v2] ← ... ← v100 (FULL, HEAD)
```

HEAD (v100) jest zawsze pełny - dostęp natychmiastowy. Aby dostać się do v1:
1. Zacznij od v100 (już masz w `/content/`)
2. Zastosuj patch v100→v99 (cofnij się o jedną wersję)
3. Zastosuj patch v99→v98
4. ... (99 razy)
5. Masz v1

**Zaleta**: 95% czasu pracujesz z HEAD - to jest natychmiastowe. Historia jest używana rzadko, więc nie szkodzi, że wymaga rekonstrukcji.

### Dlaczego "Reverse"?

Patche są "reverse" (odwrotne), bo pokazują, jak cofnąć się w czasie:
- **Forward patch** (v1→v2): "Dodaj linię X"
- **Reverse patch** (v2→v1): "Usuń linię X"

W JCF przechowujemy reverse patches, bo zaczynamy od HEAD i cofamy się wstecz.

---

## Rozdział 3: Save Checkpoint - Jak zapisujemy nową wersję?

### Podróż przez proces zapisu

Wyobraź sobie, że właśnie zmodyfikowałeś kilka plików i chcesz zapisać checkpoint. Co się dzieje?

#### Krok 1: Identyfikacja zmian

System skanuje wszystkie pliki i porównuje je z poprzednią wersją (HEAD). Dla każdego pliku:
- Oblicza hash SHA-256 aktualnej zawartości
- Porównuje z hashem z poprzedniej wersji
- Jeśli hash się różni → plik został zmieniony

**Przykład**:
```
HEAD (v5):
- src/index.js: hash = abc123
- assets/logo.png: hash = def456

Working copy:
- src/index.js: hash = xyz789  ← ZMIENIONY!
- assets/logo.png: hash = def456  ← BEZ ZMIAN
```

System identyfikuje: `src/index.js` został zmodyfikowany.

#### Krok 2: Przetwarzanie plików tekstowych

Dla każdego zmienionego pliku tekstowego:

1. **Odczytaj aktualną zawartość** (NEW) z working copy
2. **Odczytaj poprzednią zawartość** (OLD) z HEAD
3. **Wygeneruj reverse patch** (NEW → OLD)

**Normalizacja tekstu** - zanim wygenerujemy patch, normalizujemy tekst:
- Konwersja line endings: `\r\n` (Windows) → `\n` (Unix)
- Normalizacja Unicode: `café` (composed) i `cafe\u0301` (decomposed) → ten sam format
- Dodanie trailing newline (jeśli brakuje)

**Dlaczego normalizacja?** Bez niej, ten sam plik na Windows i Linux generowałby różne patche!

**Generowanie patchy** - używamy biblioteki `diff-match-patch` (Google), która implementuje algorytm Myers:
- Porównuje NEW i OLD tekst
- Znajduje różnice
- Tworzy patch w formacie tekstowym

**Przykład**:
```
OLD (v4): 
console.log('Hello');

NEW (v5):
console.log('Hello');
console.log('World');

Reverse Patch (v5→v4):
@@ -1,2 +1,1 @@
 console.log('Hello');
-console.log('World');
```

Ten patch mówi: "Aby cofnąć się z v5 do v4, usuń linię 'console.log('World');'"

4. **Zapisz patch** do `.store/deltas/v5_src_index.js.patch`

#### Krok 3: Przetwarzanie plików binarnych

Dla plików binarnych (obrazy, wideo, etc.) używamy Content Addressable Storage (CAS):

1. **Oblicz hash SHA-256** zawartości pliku
2. **Sprawdź, czy blob już istnieje** w `.store/blobs/[hash]`
3. **Jeśli nie istnieje**: zapisz nowy blob
4. **Jeśli istnieje**: nie zapisuj ponownie (deduplikacja!)

**Przykład deduplikacji**:
```
v1: Dodajesz logo.png → hash = abc123 → zapisano blob
v2: Zmieniasz logo.png → hash = def456 → zapisano nowy blob
v3: Cofasz się do starego logo.png → hash = abc123 → blob już istnieje!
    → Nie zapisujesz ponownie, tylko wskazujesz na istniejący blob
```

**Korzyść**: Jeśli ten sam plik binarny pojawia się w wielu wersjach, przechowujemy go tylko raz!

#### Krok 4: Tworzenie nowego Version object

System tworzy nowy obiekt Version, który reprezentuje snapshot stanu projektu:

```typescript
{
  id: "v6",                    // UUID nowej wersji
  timestamp: "2025-01-18T10:30:00Z",
  message: "Add login feature",
  author: "Jan Kowalski",
  parentId: "v5",              // Poprzednia wersja
  fileStates: {
    "src/index.js": {
      inodeId: "inode-123",
      contentRef: ".store/deltas/v5_src_index.js.patch",
      size: 150
    },
    "assets/logo.png": {
      inodeId: "inode-456",
      hash: "abc123",           // Wskazuje na blob
      size: 50000
    }
  }
}
```

#### Krok 5: Aktualizacja manifest.json

Manifest jest aktualizowany:
- Dodanie nowego Version do `versionHistory`
- Aktualizacja `refs.head` na nową wersję
- Aktualizacja `fileMap` z nowymi hashami i referencjami

#### Krok 6: Aktualizacja /content/

Working copy (`/content/`) już zawiera aktualny stan - nie trzeba go zmieniać! To jest kluczowe - HEAD zawsze reprezentuje aktualny stan.

#### Krok 7: Zapis do ZIP

Wszystkie zmiany są zapisywane do pliku ZIP:
- Nowe patche w `.store/deltas/`
- Nowe bloby w `.store/blobs/`
- Zaktualizowany `manifest.json`
- Zaktualizowany `/content/`

**Ważne**: Zapis jest atomowy - albo wszystko się zapisze, albo nic. Jeśli coś pójdzie nie tak, system wykonuje rollback.

---

## Rozdział 4: Restore Version - Podróż w czasie

### Jak cofamy się do starej wersji?

Wyobraź sobie, że chcesz wrócić do wersji v2, a obecnie jesteś na v10. Co się dzieje?

#### Krok 1: Budowanie ścieżki wersji

System buduje ścieżkę od HEAD do target version, przechodząc przez parent links:

```
HEAD (v10) → v9 → v8 → v7 → v6 → v5 → v4 → v3 → v2 (target)
```

Każda wersja ma `parentId`, który wskazuje na poprzednią wersję. System przechodzi wstecz przez ten łańcuch.

#### Krok 2: Rekonstrukcja plików tekstowych

Dla każdego pliku tekstowego w target version:

1. **Zacznij od aktualnej zawartości** (HEAD) z `/content/`
2. **Dla każdego kroku w ścieżce wstecz**:
   - Załaduj reverse patch (np. `v10→v9`, potem `v9→v8`, etc.)
   - Zastosuj patch do aktualnego tekstu
   - Otrzymujesz poprzednią wersję

**Przykład krok po kroku**:
```
Start: v10 content = "console.log('Hello');\nconsole.log('World');\n"

Apply patch v10→v9:
  Patch: "-console.log('World');\n"
  Result: "console.log('Hello');\n"

Apply patch v9→v8:
  Patch: (empty - no changes)
  Result: "console.log('Hello');\n"

... (kontynuuj dla v8→v7, v7→v6, etc.)

Final: v2 content = "console.log('Hello');\n"
```

**Aplikacja patchy** - proces składa się z:
1. **Parsowanie patchy** - konwersja tekstu patchy na obiekty Patch
2. **Normalizacja tekstu** - upewnienie się, że tekst używa tej samej normalizacji co przy generowaniu
3. **Aplikacja** - użycie `diff-match-patch.patch_apply()`
4. **Fuzzy matching fallback** - jeśli patch nie może się zastosować (np. plik został dodatkowo zmodyfikowany), system próbuje znaleźć podobne fragmenty w większym zakresie
5. **Snapshot fallback** - jeśli wszystko zawiedzie, system próbuje załadować pełny snapshot (jeśli dostępny)

#### Krok 3: Rekonstrukcja plików binarnych

Dla plików binarnych jest prościej:
1. Odczytaj hash z `fileState` w target version
2. Załaduj blob z `.store/blobs/[hash]`
3. Gotowe!

**Dlaczego to działa?** Pliki binarne są immutable w CAS - ten sam hash zawsze oznacza tę samą zawartość.

#### Krok 4: Aktualizacja working copy

Wszystkie zrekonstruowane pliki są zapisywane do `/content/`, zastępując aktualny stan.

#### Krok 5: Aktualizacja HEAD

`refs.head` w manifest.json jest aktualizowany na target version. Teraz HEAD wskazuje na starą wersję!

#### Krok 6: Usunięcie nieistniejących plików

Jeśli w target version nie ma pliku, który istnieje w aktualnej wersji, plik jest usuwany z `/content/`.

---

## Rozdział 5: Content Addressable Storage - Magia deduplikacji

### Jak działa CAS?

Content Addressable Storage to system, gdzie **nazwa pliku = hash jego zawartości**. To brzmi prosto, ale ma ogromne konsekwencje.

#### Dlaczego SHA-256?

SHA-256 to funkcja hashująca, która:
- Przyjmuje dowolne dane (1 bajt lub 1TB)
- Zwraca zawsze 256-bitowy hash (64 znaki hex)
- Jest deterministyczna - te same dane = ten sam hash
- Ma ekstremalnie niskie prawdopodobieństwo kolizji

**Prawdopodobieństwo kolizji**: Aby mieć 50% szansy na kolizję, potrzebujesz około 2^128 plików. To jest praktycznie niemożliwe.

#### Jak działa deduplikacja?

**Scenariusz 1: Ten sam plik w wielu wersjach**
```
v1: logo.png → hash = abc123 → zapisano blob
v2: logo.png (bez zmian) → hash = abc123 → blob już istnieje!
v3: logo.png (bez zmian) → hash = abc123 → blob już istnieje!
...
v10: logo.png (bez zmian) → hash = abc123 → blob już istnieje!

Wynik: 1 blob, 10 referencji
```

**Scenariusz 2: Revert pliku**
```
v1: logo.png → hash = abc123
v2: logo.png (zmieniony) → hash = def456
v3: logo.png (revert do v1) → hash = abc123 → blob już istnieje!

Wynik: 2 bloby (abc123 i def456), 3 referencje
```

**Scenariusz 3: Duplikaty plików**
```
assets/logo.png → hash = abc123 → zapisano blob
assets/logo-copy.png (ten sam plik) → hash = abc123 → blob już istnieje!

Wynik: 1 blob, 2 referencje
```

#### Struktura `.store/blobs/`

Bloby są przechowywane w płaskiej strukturze:
```
.store/blobs/
├── a3f5e8d9c1b2e4f67890abcdef1234567890abcdef1234567890abcdef123456
├── 9d4c1e2b3a4f5e6d7c8b9a0f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8f9e0
└── ...
```

Każdy plik ma nazwę równą jego hash SHA-256 (64 znaki hex).

**Dlaczego płaska struktura?**
- Prostsza implementacja
- Brak problemów z path traversal
- Łatwiejsze garbage collection

**Alternatywa (Git-style sharding)**:
```
.store/blobs/a3/f5e8d9c1b2e4f6...
```

To mogłoby być lepsze dla bardzo dużej liczby blobów, ale dla większości przypadków płaska struktura jest wystarczająca.

#### Kompresja blobów

Większość plików binarnych jest już skompresowana:
- PNG, JPG - kompresja obrazów
- MP4, AVI - kompresja wideo
- ZIP, TAR - archiwa

Recompression tych plików byłby stratą czasu i dawałby marginalne korzyści. Dlatego bloby są przechowywane bez dodatkowej kompresji (STORE method w ZIP).

---

## Rozdział 6: Diff Generation - Jak tworzymy patche?

### Algorytm Myers w akcji

Gdy system generuje patch dla pliku tekstowego, używa algorytmu Myers (zaimplementowanego w `diff-match-patch`).

#### Krok 1: Normalizacja

Zanim zaczniemy porównywać teksty, normalizujemy je:
- **Line endings**: `\r\n` → `\n`, `\r` → `\n`
- **Unicode**: NFC normalization (`café` i `cafe\u0301` → ten sam format)
- **Trailing newline**: Dodaj, jeśli brakuje

**Dlaczego?** Bez normalizacji, ten sam plik na różnych systemach generowałby różne patche!

#### Krok 2: Porównanie

Algorytm Myers porównuje dwa teksty i znajduje optymalną sekwencję zmian (dodania, usunięcia, modyfikacje).

**Złożoność**:
- Best case: O(N) - identyczne pliki
- Average case: O(ND) - N = długość, D = liczba różnic
- Worst case: O(N²) - całkowicie różne pliki

#### Krok 3: Tworzenie patchy

Algorytm tworzy obiekty Patch, które zawierają:
- **diffs**: Tablica zmian ([-1, "usuń"], [0, "bez zmian"], [1, "dodaj"])
- **start1, start2**: Pozycje w starym i nowym tekście
- **length1, length2**: Długości fragmentów

#### Krok 4: Serializacja

Patche są serializowane do formatu tekstowego:
```
@@ -1,2 +1,1 @@
 console.log('Hello');
-console.log('World');
```

**Format**:
- `@@ -start,count +start,count @@`: Header (pozycja i długość)
- `-`: Linia do usunięcia
- `+`: Linia do dodania
- ` `: Linia bez zmian (kontekst)

#### Optymalizacje

**Hash-based skip**: Jeśli hashe są identyczne, skip diffing (10-100x szybsze).

**Worker pool**: Dla plików >100KB, diffing jest wykonywany w Web Worker (nie blokuje UI).

**Threshold check**: Jeśli patch jest większy niż 50% rozmiaru pliku, rozważ snapshot zamiast patch.

---

## Rozdział 7: Patch Application - Jak aplikujemy patche?

### Proces aplikacji patchy

Gdy system chce zrekonstruować starą wersję, musi zastosować reverse patches. Jak to działa?

#### Krok 1: Parsowanie

Patch tekstowy jest parsowany do obiektów Patch:
```typescript
const patchText = '@@ -1,2 +1,1 @@\n console.log("Hello");\n-console.log("World");\n';
const patches = dmp.patch_fromText(patchText);
// patches = [Patch object with diffs array]
```

#### Krok 2: Normalizacja tekstu

Aktualny tekst jest normalizowany (ta sama normalizacja co przy generowaniu):
- Line endings → LF
- Unicode → NFC
- Trailing newline

**Ważne**: Normalizacja musi być identyczna, inaczej patch się nie zastosuje!

#### Krok 3: Aplikacja

Używamy `diff-match-patch.patch_apply()`:
```typescript
const [resultText, successArray] = dmp.patch_apply(patches, normalizedText);
```

**Return values**:
- `resultText`: Zrekonstruowany tekst
- `successArray`: Tablica boolean - czy każdy patch się powiódł

#### Krok 4: Fuzzy Matching (fallback)

Jeśli patch nie może się zastosować (np. plik został dodatkowo zmodyfikowany), system próbuje fuzzy matching:
- Zwiększa `Match_Distance` (szuka w większym zakresie)
- Obniża `Match_Threshold` (bardziej tolerancyjny)
- Próbuje znaleźć podobne fragmenty

**Przykład**:
```
Oryginalny patch: usuń "CHANGE_ME" z linii 4
Ale plik został zmodyfikowany: dodano linię przed linią 4
→ Pozycja się zmieniła, ale fuzzy matching znajdzie "CHANGE_ME" w nowej pozycji
```

#### Krok 5: Snapshot Fallback

Jeśli fuzzy matching zawiedzie, system próbuje załadować pełny snapshot (jeśli dostępny):
- Snapshots są tworzone co N wersji (np. co 50)
- Zawierają pełną zawartość wszystkich plików
- Są większe, ale gwarantują sukces

#### Krok 6: Walidacja

Po aplikacji, system weryfikuje wynik:
- Sprawdza hash (jeśli dostępny w fileState)
- Porównuje z oczekiwanym wynikiem
- Jeśli hash się nie zgadza → snapshot fallback

---

## Rozdział 8: Streaming Architecture - Obsługa dużych plików

### Problem z dużymi plikami

Wyobraź sobie, że chcesz dodać plik wideo 500MB do projektu. W przeglądarce, załadowanie całego pliku do RAM spowodowałoby crash.

### Rozwiązanie: Streaming

Zamiast ładować cały plik, przetwarzamy go w chunkach:

```typescript
// BAD: Load entire file
const buffer = await file.arrayBuffer(); // 500MB w RAM!

// GOOD: Stream
const stream = file.stream();
await manager.addFile('video.mp4', stream); // Chunked processing
```

#### Jak to działa?

1. **Stream hashing**: Hash jest obliczany przyrostowo, chunk po chunk
2. **Stream writing**: Plik jest zapisywany do ZIP w chunkach
3. **Progress tracking**: Możemy śledzić postęp (np. "45% uploaded")

**Korzyści**:
- RAM usage: ~50MB (chunks) zamiast 500MB
- Brak crashy przeglądarki
- Progress tracking
- Możliwość anulowania

#### Implementacja

```typescript
async function saveBlobStreaming(stream: ReadableStream): Promise<string> {
  // Tee stream: jeden do hashowania, jeden do zapisu
  const [hashStream, writeStream] = stream.tee();
  
  // Hash w tle
  const hashPromise = sha256Stream(hashStream);
  
  // Zapisz do tymczasowej lokalizacji
  const tempPath = `.store/temp/${uuidv4()}`;
  await this.writeToZipStream(tempPath, writeStream);
  
  // Pobierz hash
  const hash = await hashPromise;
  const finalPath = `.store/blobs/${hash}`;
  
  // Sprawdź deduplikację
  if (await this.blobExists(finalPath)) {
    await this.deleteFromZip(tempPath); // Deduplikowany!
  } else {
    await this.renameInZip(tempPath, finalPath);
  }
  
  return hash;
}
```

---

## Rozdział 9: Platform Abstraction - Działanie wszędzie

### Problem z różnymi platformami

Różne środowiska mają różne API:
- **Browser**: IndexedDB, File API
- **Node.js**: `fs/promises`
- **Tauri**: `tauri.fs`
- **Deno**: `Deno.readFile`

### Rozwiązanie: Adapter Pattern

Zamiast pisać kod specyficzny dla platformy, używamy adapterów:

```typescript
interface FileSystemAdapter {
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  fileExists(path: string): Promise<boolean>;
  // ...
}
```

**Implementacje**:
- `BrowserAdapter` - używa IndexedDB + File API
- `NodeAdapter` - używa `fs/promises`
- `TauriAdapter` - używa `tauri.fs`
- `MemoryAdapter` - dla testów

**Korzyści**:
- Ten sam kod działa wszędzie
- Łatwe testowanie (MemoryAdapter)
- Łatwe dodawanie nowych platform

---

## Rozdział 10: Garbage Collection - Sprzątanie historii

### Problem z orphaned blobs

Gdy usuwasz wersję lub plik, bloby mogą zostać "osierocone" - nie są używane przez żadną wersję, ale nadal zajmują miejsce.

### Rozwiązanie: Garbage Collection

GC identyfikuje i usuwa orphaned blobs:

1. **Kolekcja referencji**: Przejrzyj wszystkie wersje i zbierz wszystkie używane hashe
2. **Lista wszystkich blobów**: Sprawdź, które bloby istnieją w `.store/blobs/`
3. **Znajdź osierocone**: Bloby, które nie są w referencjach
4. **Usuń**: Usuń orphaned bloby i repack ZIP

**Grace period**: GC może mieć "grace period" - nie usuwa blobów od razu, tylko po N dniach (np. 7 dni). To daje czas na recovery, jeśli coś poszło nie tak.

**Przykład**:
```
Wersje: v1, v2, v3, v4, v5
Bloby: abc123 (używany przez v1, v2), def456 (używany przez v3), xyz789 (nie używany)

GC:
- Referencje: {abc123, def456}
- Wszystkie bloby: {abc123, def456, xyz789}
- Orphaned: {xyz789}
- Usuń: xyz789
```

---

## Rozdział 11: Snapshots - Optymalizacja dla głębokiej historii

### Problem z długimi ścieżkami

Jeśli masz 1000 wersji i chcesz wrócić do v1, musisz zastosować 999 patchy. To może być wolne.

### Rozwiązanie: Periodic Snapshots

System tworzy pełne snapshots co N wersji (np. co 50):

```
v1 (snapshot) → v2 → ... → v50 (snapshot) → v51 → ... → v100 (snapshot)
```

Gdy chcesz wrócić do v1:
1. Znajdź najbliższy snapshot (v1)
2. Załaduj snapshot (pełna zawartość)
3. Zastosuj patche od snapshot do target (jeśli potrzebne)

**Trade-off**:
- Pro: Szybsze restore dla starych wersji
- Con: Większy rozmiar pliku (snapshots są pełne)

**Przykład**:
```
Bez snapshot: restore v1 z v1000 = 999 patches
Z snapshot co 50: restore v1 z v1000 = 19 patches (od v1000 do v950) + snapshot v950 + 49 patches (od v950 do v1)
```

---

## Rozdział 12: Error Handling - Co gdy coś pójdzie nie tak?

### Rodzaje błędów

#### 1. Patch Application Failed

**Przyczyna**: Patch nie może się zastosować (np. plik został dodatkowo zmodyfikowany)

**Rozwiązanie**:
1. Próbuj fuzzy matching
2. Jeśli zawiedzie → snapshot fallback
3. Jeśli snapshot niedostępny → zwróć częściowo zaaplikowany wynik z ostrzeżeniem

#### 2. Blob Not Found

**Przyczyna**: Blob został usunięty przez GC lub korupcja ZIP

**Rozwiązanie**:
1. Sprawdź, czy blob istnieje w innych lokalizacjach
2. Spróbuj znaleźć alternatywną wersję z tym samym hashem
3. Jeśli wszystko zawiedzie → błąd

#### 3. Manifest Corruption

**Przyczyna**: manifest.json jest uszkodzony

**Rozwiązanie**:
1. Próbuj naprawić (walidacja JSON Schema)
2. Jeśli zawiedzie → próbuj odzyskać z backup
3. Ostatnia deska ratunku → ręczna naprawa przez użytkownika

#### 4. Rollback

Gdy krytyczny błąd występuje podczas saveCheckpoint, system wykonuje rollback:
- Przywraca poprzedni manifest
- Usuwa tymczasowe pliki
- Emituje event `checkpoint:rollback`

---

## Rozdział 13: Performance Optimizations - Szybkość działania

### Optymalizacje dla Save Checkpoint

1. **Parallel hashing**: Hashowanie wielu plików równolegle (4-8x szybsze)
2. **Worker pool**: Diffing w Web Workers (nie blokuje UI)
3. **Hash-based skip**: Skip diffing, jeśli hashe są identyczne
4. **Incremental processing**: Przetwarzaj pliki, gdy są gotowe (lepsze perceived performance)

### Optymalizacje dla Restore Version

1. **Snapshot usage**: Używaj snapshotów, gdy dostępne
2. **Batch patch loading**: Załaduj wszystkie patche upfront (dla małych plików)
3. **Streaming patch application**: Stream patche jeden po drugim (dla dużych plików)
4. **Patch caching**: Cache często używanych patchy (LRU cache)

### Optymalizacje dla CAS

1. **Blob caching**: LRU cache dla często używanych blobów
2. **Parallel blob operations**: Hashowanie i zapis wielu blobów równolegle
3. **Blob prefetching**: Prefetch blobów dla wersji w tle

---

## Rozdział 14: Security Considerations - Bezpieczeństwo

### Zagrożenia i mitagacje

#### 1. ZIP Bombs

**Zagrożenie**: Złośliwy plik z ekstremalną kompresją (1KB → 1GB po rozpakowaniu)

**Mitagacja**: Limity rozmiaru dekompresji

#### 2. Path Traversal

**Zagrożenie**: `../../etc/passwd` w nazwach plików

**Mitagacja**: Sanityzacja ścieżek, walidacja przed zapisem

#### 3. Manifest Tampering

**Zagrożenie**: Ręczna edycja manifest.json

**Mitagacja**: Checksums, walidacja JSON Schema

#### 4. Hash Collision

**Zagrożenie**: Dwa różne pliki z tym samym hashem (praktycznie niemożliwe dla SHA-256)

**Mitagacja**: SHA-256 jest collision-resistant

---

## Epilog: Całość w działaniu

### Przykładowy scenariusz

Wyobraź sobie, że pracujesz nad projektem gier:

1. **Dzień 1**: Tworzysz projekt, dodajesz pliki (`index.js`, `styles.css`, `logo.png`)
   - `saveCheckpoint("Initial commit")` → v1
   - System zapisuje wszystko do `/content/` i tworzy manifest

2. **Dzień 2**: Dodajesz funkcję logowania
   - Modyfikujesz `index.js`
   - `saveCheckpoint("Add login")` → v2
   - System generuje reverse patch (v2→v1) dla `index.js`

3. **Dzień 3**: Zmieniasz logo
   - Modyfikujesz `logo.png`
   - `saveCheckpoint("Update logo")` → v3
   - System zapisuje nowy blob (hash różny), stary blob pozostaje (może być używany przez v1, v2)

4. **Dzień 10**: Chcesz wrócić do starego logo
   - `restoreVersion("v2")` → system aplikuje reverse patches i przywraca starą wersję
   - `logo.png` jest ładowany z CAS (hash z v2)

5. **Tydzień później**: Uruchamiasz GC
   - System usuwa orphaned bloby (np. nowe logo z v3, jeśli nie jest używane)
   - Oszczędzasz miejsce

### Kluczowe wnioski

1. **HEAD zawsze szybki** - Reverse Delta Strategy optymalizuje dla common case
2. **Deduplikacja automatyczna** - CAS eliminuje duplikaty
3. **Historia kompletna** - Każda wersja może być zrekonstruowana
4. **Kompatybilność** - Standardowy ZIP = można otworzyć wszędzie
5. **Skalowalność** - Streaming i snapshots dla dużych projektów

---

## Podsumowanie: Filozofia JCF

JCF to nie tylko format pliku - to filozofia projektowania systemu wersjonowania:

- **Optymalizuj dla common case**: 95% czasu = HEAD, więc HEAD jest zawsze pełny
- **Deduplikacja automatyczna**: CAS eliminuje duplikaty bez wysiłku
- **Kompatybilność**: Standardowy ZIP = recovery możliwy zawsze
- **Skalowalność**: Streaming i snapshots dla dużych projektów
- **Bezpieczeństwo**: Walidacja, checksums, atomic writes

System jest zaprojektowany tak, aby był:
- **Szybki** dla typowych operacji (HEAD access)
- **Efektywny** w wykorzystaniu przestrzeni (deltas + deduplikacja)
- **Niezawodny** (error handling, rollback, recovery)
- **Uniwersalny** (działa wszędzie dzięki adapterom)

To jest opowieść o JCF - systemie, który pozwala podróżować w czasie przez historię projektu, zachowując przy tym efektywność i kompatybilność.

---

**Ostatnia aktualizacja**: 2025-01-18  
**Wersja dokumentu**: 1.0.0


