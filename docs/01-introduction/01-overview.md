# Kamaros JCF Manager

## System zarządzania plikami projektowymi

JCF Manager to biblioteka izomorficzna JavaScript/TypeScript do zarządzania plikami projektowymi z wbudowanym systemem wersjonowania Time-Travel.

## Kluczowe cechy systemu

### Reverse Delta Versioning
Aktualna wersja przechowywana w pełnej postaci, historia jako patche. Zapewnia natychmiastowy dostęp do bieżącego stanu projektu.

### Content Addressable Storage
Automatyczna deduplikacja plików binarnych poprzez haszowanie SHA-256. Te same pliki przechowywane tylko raz.

### Izomorficzność
Działa w każdym środowisku JavaScript:
- Przeglądarka: IndexedDB + File API
- Node.js: fs/promises
- Tauri: @tauri-apps/api/fs

### Streaming Support
Obsługa plików większych niż 500MB bez przepełnienia pamięci RAM poprzez strumieniowe przetwarzanie.

### Wielowątkowość
Web Workers dla operacji wymagających dużych zasobów CPU - haszowanie, diffing, kompresja - interfejs użytkownika pozostaje responsywny.

### Production Ready
Kompletna obsługa błędów, walidacja danych, kompleksowy zestaw testów.

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