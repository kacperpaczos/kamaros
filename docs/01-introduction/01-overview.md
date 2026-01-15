# Kamaros JCF Manager

## Inteligentny format pliku ZIP z wersjonowaniem

**JCF (JSON Content Format)** to inteligentny format pliku oparty na standardowym ZIP archive, zaprojektowany do przechowywania projektów z pełną historią wersji. Łączy prostotę ZIP z zaawansowanym systemem wersjonowania Time-Travel.

**Biblioteka JCF Manager** dostępna dla **TypeScript** i **Python** z wysokowydajnym **core w Rust** zapewnia efektywne zarządzanie plikami zgodnie ze specyfikacją formatu.

### Jak działa biblioteka:

1. **Tworzy lub wczytuje specyfikację JCF** (`manifest.json`) - definicję struktury projektu i historii wersji
2. **Wczytuje pliki zgodnie ze specyfikacją** - odczytuje i rekonstruuje pliki z formatu JCF
3. **Zapisuje pliki zgodnie ze specyfikacją** - zapisuje do formatu JCF z wersjonowaniem
4. **Waliduje zgodność ze specyfikacją** - sprawdza integralność i poprawność danych
5. **Wersjonuje zgodnie ze specyfikacją** - zarządza historią zmian zgodnie z formatem JCF

**Ważne**: Biblioteka nie interpretuje zawartości plików - przechowuje dowolne pliki (`.js`, `.glsl`, `.json`, `.png`, etc.) zgodnie ze specyfikacją formatu JCF. To aplikacja decyduje, jakie pliki przechowuje i jak je interpretuje.

## Kluczowe cechy systemu

### Format pliku: Inteligentny ZIP
Format JCF to standardowy ZIP archive z inteligentną strukturą wewnętrzną:
- **Manifest.json** - specyfikacja projektu i historii wersji
- **content/** - aktualny stan plików (working directory)
- **.store/blobs/** - deduplikowane pliki binarne (CAS)
- **.store/deltas/** - reverse delta patches dla plików tekstowych
- **Rozszerzenie**: `.jcf`, **MIME type**: `application/x-jcf`

### Reverse Delta Versioning
Aktualna wersja przechowywana w pełnej postaci, historia jako patche. Zapewnia natychmiastowy dostęp do bieżącego stanu projektu.

### Content Addressable Storage
Automatyczna deduplikacja plików binarnych poprzez haszowanie SHA-256. Te same pliki przechowywane tylko raz.

### Wielojęzyczna implementacja
Biblioteka dostępna dla **TypeScript** i **Python** z **core w Rust**:

**TypeScript/JavaScript:**
- Przeglądarka: IndexedDB + File API + Web Streams
- Node.js: fs/promises + native streams
- Tauri: @tauri-apps/api/fs
- Deno/Bun: pełne wsparcie

**Python:**
- asyncio-based API
- Pydantic models dla typów
- PyO3 FFI do Rust core

**Rust Core (WASM):**
- Wysokowydajne operacje: kompresja, haszowanie, diffing
- Memory-safe implementation
- WebAssembly dla przeglądarek
- Native performance dla Node.js/Python

### Streaming Support
Obsługa plików większych niż 500MB bez przepełnienia pamięci RAM poprzez strumieniowe przetwarzanie.

### Wielowątkowość
- **TypeScript/JavaScript**: Wykorzystanie Web Workers (przeglądarka) oraz Worker Threads (Node.js) do przeniesienia operacji wymagających dużych zasobów CPU (haszowanie, diffing, kompresja) do osobnych wątków, dzięki czemu interfejs użytkownika pozostaje w pełni responsywny.
- **Python**: Wykorzystanie modułu `multiprocessing` oraz natywnej wielowątkowości Rust Core. Dzięki zastosowaniu PyO3, krytyczne wydajnościowo operacje zwalniają blokadę GIL (Global Interpreter Lock), umożliwiając rzeczywistą równoległość obliczeń na wielu rdzeniach.
- **Rust Core**: Równoległe przetwarzanie danych przy użyciu biblioteki Rayon oraz asynchroniczne operacje wejścia/wyjścia, co pozwala na optymalne wykorzystanie architektury wielordzeniowej przy minimalnym narzucie pamięciowym.

### Production Ready
- **Kompleksowa obsługa błędów**: Precyzyjna diagnostyka i obsługa wyjątków na wszystkich warstwach (Rust, TS, Python), zapewniająca stabilność nawet w sytuacjach krytycznych.
- **Rygorystyczna walidacja**: Automatyczna weryfikacja schematu `manifest.json` oraz walidacja sum kontrolnych SHA-256 dla każdego blobu, gwarantująca 100% integralności danych.
- **Zaawansowane testy**: Rozbudowany zestaw testów jednostkowych i integracyjnych (CI/CD) obejmujący różne systemy operacyjne oraz środowiska uruchomieniowe (Node.js, Browser, Python).
- **Bezpieczeństwo zapisu**: Mechanizmy atomowych operacji na plikach zapobiegające uszkodzeniu archiwum w przypadku nagłego przerwania procesu.

## Architektura wysokiego poziomu

```
┌─────────────────────────────────┐
│       Aplikacja użytkownika      │
│  (Browser/Node/Tauri/Deno)      │
└─────────────────┬───────────────┘
                  │
                  │ Public API
                  ↓
┌─────────────────────────────────┐
│       Klasa JCFManager          │
│ (Facade dla wszystkich operacji)│
└─────────────────┬───────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
        ↓                   ↓
┌─────────────┐  ┌─────────────┐
│ Warstwa     │  │ Warstwa     │
│ rdzeniowa   │  │ przechowy-  │
│ - Wersjonowanie││ - ZIP       │
│ - Diffing    │  │ - CAS       │
│ - CAS        │  │ - Deltas    │
└──────┬──────┘  └─────┬─────┘
       │                │
       │    ┌───────────┘
       ↓    ↓
┌──────────────────────┐
│  Warstwa adapterów   │
│  (specyficzna dla    │
│   platformy)         │
│  - Browser           │
│  - Node.js           │
│  - Tauri             │
└──────────────────────┘
```

## Zastosowania

### Edytory kodu
Edytory kodu w stylu VSCode z pełną historią zmian, praca offline.

### Narzędzia graficzne
Aplikacje graficzne w stylu Figma z wersjonowaniem elementów projektu.

### Edytory gier
Edytory gier jak Unity z wersjonowaniem assetów.

### Edytory dokumentów
Edytory dokumentów jak Google Docs z podejściem offline-first.

### Narzędzia CAD i modelowania 3D
Narzędzia CAD i modelowania 3D z kompleksowym version control.

## Technologie kluczowe

| Komponent | Technologia | Uzasadnienie |
|-----------|-------------|--------------|
| Kompresja | fflate | Dwudziestokrotnie szybsza od JSZip, natywny streaming |
| Diffing | diff-match-patch | Sprawdzona technologia (Google Docs), kompletne API |
| Hashing | WebCrypto/hash-wasm | Natywna, bezpieczna implementacja SHA-256 |
| Generowanie ID | uuid v4 | Odporne na kolizje, standard |
| Bezpieczeństwo typów | TypeScript 5.3+ | Błędy kompilacji, lepsza DX |

## Korzyści dla deweloperów

Dla deweloperów:
- Zero configuration - działa out-of-the-box
- Type-safe API - interfejsy TypeScript
- Platform agnostic - jeden kod, wszystkie platformy
- Extensible - wzorzec adapter, system plugin (przyszłość)

Dla użytkowników końcowych:
- Offline-first - nie wymaga serwera
- Szybki - reverse delta = natychmiastowy dostęp do HEAD
- Niezawodny - format ZIP = standardowe narzędzia recovery
- Efektywny - deduplikacja = mniejszy rozmiar

## Porównanie z alternatywami

| Cecha | Kamaros JCF | Git (isomorphic-git) | Format własny |
|-------|-------------|----------------------|----------------|
| Wsparcie przeglądarki | Tak | Tak | Nie |
| Rozmiar plików dużych | Tak | Nie | Tak |
| Deduplikacja binariów | Tak | Nie | Ręcznie |
| Wydajność HEAD | Tak | Tak | Zależy |
| Narzędzia recovery | Tak | Tak | Brak |
| Krzywa uczenia | Prosta | Strom | Nieznana |

Kamaros wypełnia niszę między Git (zbyt złożony, słaby z binariami) a rozwiązaniami własnymi (brak standardów).

## Techniczne założenia

Format JCF to standardowy ZIP archive zaprojektowany specjalnie dla przechowywania projektów z pełną historią wersji. Łączy prostotę ZIP z zaawansowanym systemem wersjonowania.