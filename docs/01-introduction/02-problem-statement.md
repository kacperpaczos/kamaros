# Problem zarządzania wersjami plików w aplikacjach web

## Scenariusz użycia

Tworzenie edytora kodu w przeglądarce (jak CodeSandbox/StackBlitz). Użytkownik pracuje nad projektem i potrzebuje:
- Undo/Redo - cofanie zmian
- History - przeglądanie historii edycji
- Restore - przywrócenie do dowolnego punktu
- Offline - działanie bez serwera
- Performance - szybki dostęp do aktualnego stanu

## Dostępne rozwiązania

### Opcja A: Git (isomorphic-git)

Zalety:
- Sprawdzony system wersjonowania
- Potężne narzędzia

Wady:
- Słaba obsługa plików binarnych (duże obrazy, wideo)
- Skomplikowany (staging, branches, merge conflicts)
- Pełna historia w pamięci (problem dla dużych projektów)
- Nie działa dobrze z projektami większymi niż 100MB w przeglądarce

### Opcja B: Format własny (custom binary)

Zalety:
- Pełna kontrola

Wady:
- Brak standardów
- Zero toolingu (debugging, recovery)
- Długi czas rozwoju
- Trudne utrzymanie

### Opcja C: Baza danych (IndexedDB/SQLite)

Zalety:
- Structured query

Wady:
- Brak kompresji
- Trudne backup/export
- Nie działa jako standalone file
- Specyficzne dla platformy

## Luka na rynku

Potrzebny jest:
- Prosty format (łatwiejszy niż Git)
- Dobrze radzi sobie z binariami
- Działa offline
- Fast HEAD access
- Standard recovery tools (ZIP)
- Izomorficzny (Browser/Node/Tauri)

To jest Kamaros!

## Problem rozmiaru pliku

### Scenariusz
Projekt zawiera:
- 1000 plików tekstowych (~10MB)
- 50 obrazów (~50MB)
- Historia 100 commitów

### Naiwne podejście (snapshot każdej wersji):
100 versions × (10MB text + 50MB images) = 6GB

### Podejście Git (forward deltas + pack):
~500MB (lepiej, ale nadal duże)

### Podejście Kamaros (reverse delta + CAS):
10MB (current text) + 50MB (current images) + 5MB (patches for 100 versions) + 10MB (deduplicated old images) = ~75MB

### Techniki kompresji

1. Reverse Delta: HEAD zawsze pełny (szybki), historia jako patche
2. Content Addressable Storage: Ten sam obraz użyty 10x → przechowany 1x
3. Smart Compression: Tylko pliki, które się kompresują

## Problem wydajności w przeglądarce

### Scenariusz
Użytkownik dodaje 500MB wideo do projektu w edytorze webowym.

### Naiwne podejście:
Ładowanie całego pliku do pamięci RAM, haszowanie na głównym wątku UI, zapis do IndexedDB.

### Podejście Kamaros:
Strumieniowe przetwarzanie, Web Workers dla haszowania, chunked write.

### Rezultat:
- Zużycie RAM: ~50MB (bufor chunków)
- UI responsywny (Web Workers)
- Przyrostowe aktualizacje postępu

## Rozwiązanie: Kamaros JCF

**JCF (JSON Content Format)** to inteligentny format pliku oparty na standardowym ZIP archive, który łączy prostotę ZIP z zaawansowanym systemem wersjonowania.

### Jak Kamaros rozwiązuje te problemy?

| Problem | Rozwiązanie Kamaros |
|---------|---------------------|
| Złożoność | Simple API (addFile, saveCheckpoint, restoreVersion) |
| Pliki binarne | Content Addressable Storage + deduplikacja |
| Rozmiar pliku | Reverse delta + smart compression |
| Wydajność | Streaming + Web Workers + LRU cache |
| Recovery | Standard ZIP tools |
| Platforma | Adapter pattern (Browser/Node/Tauri) |