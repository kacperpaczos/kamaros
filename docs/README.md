# Kamaros Documentation

> **High-Performance File Management Library with Time-Travel Versioning**
>
> Complete technical documentation for building a production-ready, isomorphic library for managing project files with full version history, optimized through hybrid Rust/TypeScript architecture.

---

## 📖 Jak Czytać Tę Dokumentację

Ta dokumentacja jest zorganizowana jako **liniarna narracja** - każdy rozdział buduje na poprzednich konceptach, prowadząc Cię od zrozumienia problemu do pełnej implementacji.

**Zalecana kolejność czytania**: 01 → 02 → 03 → 04 → 05 → 06 → 07 → 08 → 09 → 10

Możesz również wybrać ścieżkę dostosowaną do Twojej roli:

### 🚀 Szybki Start (Użytkownicy)
Chcesz tylko nauczyć się jak używać biblioteki?
- [`01-introduction/01-overview.md`](01-introduction/01-overview.md) - Czym jest Kamaros
- [`08-usage-guide/01-installation.md`](08-usage-guide/01-installation.md) - Instalacja
- [`08-usage-guide/02-quick-start.md`](08-usage-guide/02-quick-start.md) - Pierwsze kroki
- [`08-usage-guide/03-basic-operations.md`](08-usage-guide/03-basic-operations.md) - Podstawowe operacje

### 🏗️ Architekt (System Design)
Chcesz zrozumieć architekturę i decyzje projektowe?
- [`01-introduction/`](01-introduction/) - Problem i motywacja
- [`02-core-concepts/`](02-core-concepts/) - Fundamentalne koncepty
- [`03-architecture/`](03-architecture/) - Architektura systemu
- [`04-technical-decisions/`](04-technical-decisions/) - Uzasadnienie wyborów

### 💻 Implementator (Developerzy)
Chcesz wnieść wkład lub zrozumieć implementację?
- [`03-architecture/`](03-architecture/) - Jak działa system
- [`05-algorithms/`](05-algorithms/) - Szczegółowe algorytmy
- [`06-implementation/`](06-implementation/) - Standardy kodowania
- [`07-api-reference/`](07-api-reference/) - Kompletne API
- [`10-development/`](10-development/) - Setup i contributing

### 📚 Pełne Zrozumienie (Wszystko)
Chcesz znać każdy szczegół? Czytaj od A do Z:
**01** → **02** → **03** → **04** → **05** → **06** → **07** → **08** → **09** → **10**

---

## 📑 Mapa Dokumentacji

### 01. Introduction
**Dlaczego Kamaros istnieje?**

Wprowadzenie do problemu i rozwiązania.

- [`01-overview.md`](01-introduction/01-overview.md) - Czym jest Kamaros/JCF Manager
- [`02-problem-statement.md`](01-introduction/02-problem-statement.md) - Problem: zarządzanie wersjami plików
- [`03-key-concepts.md`](01-introduction/03-key-concepts.md) - Podstawowe koncepty
- [`04-use-cases.md`](01-introduction/04-use-cases.md) - Przykłady zastosowań

**Główne pytanie:** *"Dlaczego potrzebujemy tej biblioteki?"*

---

### 02. Core Concepts
**Jakie są fundamenty?**

Kluczowe koncepcje techniczne, które musisz znać.

- [`01-jcf-format.md`](02-core-concepts/01-jcf-format.md) - Specyfikacja formatu JCF (ZIP-based)
- [`02-time-travel-versioning.md`](02-core-concepts/02-time-travel-versioning.md) - Koncepcja time-travel
- [`03-reverse-delta-strategy.md`](02-core-concepts/03-reverse-delta-strategy.md) - Strategia wersjonowania
- [`04-content-addressing.md`](02-core-concepts/04-content-addressing.md) - CAS i deduplikacja
- [`05-streaming-architecture.md`](02-core-concepts/05-streaming-architecture.md) - Obsługa dużych plików
- [`06-platform-abstraction.md`](02-core-concepts/06-platform-abstraction.md) - Multi-platform support

**Główne pytanie:** *"Jakie są kluczowe koncepcje, które napędzają system?"*

---

### 03. Architecture
**Jak system jest zbudowany?**

Szczegółowa architektura i interakcje komponentów.

- [`01-system-overview.md`](03-architecture/01-system-overview.md) - Diagram całego systemu
- [`02-layer-architecture.md`](03-architecture/02-layer-architecture.md) - Warstwy: API → Core → Storage
- [`03-data-flow.md`](03-architecture/03-data-flow.md) - Przepływ danych (save/restore)
- [`04-component-interaction.md`](03-architecture/04-component-interaction.md) - Jak komponenty współpracują
- [`05-design-patterns.md`](03-architecture/05-design-patterns.md) - Wzorce projektowe
- [`06-data-structures.md`](03-architecture/06-data-structures.md) - Struktury danych

**Główne pytanie:** *"Jak to wszystko ze sobą współpracuje?"*

---

### 04. Technical Decisions
**Dlaczego te technologie?**

Uzasadnienie kluczowych decyzji technicznych.

- [`01-rust-vs-typescript.md`](04-technical-decisions/01-rust-vs-typescript.md) - Matryca decyzyjna
- [`02-hybrid-architecture.md`](04-technical-decisions/02-hybrid-architecture.md) - Rust core + bindings
- [`03-compression-library.md`](04-technical-decisions/03-compression-library.md) - fflate vs JSZip
- [`04-diff-algorithm.md`](04-technical-decisions/04-diff-algorithm.md) - diff-match-patch
- [`05-concurrency-model.md`](04-technical-decisions/05-concurrency-model.md) - Web Workers
- [`06-performance-rationale.md`](04-technical-decisions/06-performance-rationale.md) - Uzasadnienia

**Główne pytanie:** *"Dlaczego wybraliśmy właśnie te technologie i podejścia?"*

---

### 05. Algorithms
**Jak dokładnie to działa?**

Szczegółowe algorytmy krok po kroku.

- [`01-save-checkpoint.md`](05-algorithms/01-save-checkpoint.md) - Algorytm zapisu wersji
- [`02-restore-version.md`](05-algorithms/02-restore-version.md) - Algorytm odtwarzania
- [`03-diff-generation.md`](05-algorithms/03-diff-generation.md) - Generowanie patchy (Myers)
- [`04-patch-application.md`](05-algorithms/04-patch-application.md) - Aplikowanie patchy
- [`05-garbage-collection.md`](05-algorithms/05-garbage-collection.md) - GC: Mark & Sweep
- [`06-file-rename-tracking.md`](05-algorithms/06-file-rename-tracking.md) - Śledzenie rename/move
- [`07-conflict-resolution.md`](05-algorithms/07-conflict-resolution.md) - Rozwiązywanie konfliktów

**Główne pytanie:** *"Jakie są dokładne kroki każdej operacji?"*

---

### 06. Implementation
**Jak zakodować rozwiązanie?**

Praktyczne wskazówki dla implementacji.

- [`01-project-structure.md`](06-implementation/01-project-structure.md) - Struktura projektu
- [`02-naming-conventions.md`](06-implementation/02-naming-conventions.md) - Konwencje nazewnictwa
- [`03-coding-standards.md`](06-implementation/03-coding-standards.md) - Standardy kodu
- [`04-module-organization.md`](06-implementation/04-module-organization.md) - Organizacja modułów
- [`05-error-handling.md`](06-implementation/05-error-handling.md) - Obsługa błędów
- [`06-testing-strategy.md`](06-implementation/06-testing-strategy.md) - Strategia testowania
- [`07-build-pipeline.md`](06-implementation/07-build-pipeline.md) - Build i packaging

**Główne pytanie:** *"Jak to praktycznie zakodować zgodnie z best practices?"*

---

### 07. API Reference
**Jak używać biblioteki?**

Kompletna dokumentacja API.

- [`01-jcf-manager-class.md`](07-api-reference/01-jcf-manager-class.md) - Główna klasa
- [`02-core-methods.md`](07-api-reference/02-core-methods.md) - init, save, restore, addFile
- [`03-query-methods.md`](07-api-reference/03-query-methods.md) - getHistory, getFile, listFiles
- [`04-utility-methods.md`](07-api-reference/04-utility-methods.md) - gc, validate, export
- [`05-typescript-types.md`](07-api-reference/05-typescript-types.md) - Definicje typów
- [`06-rust-bindings.md`](07-api-reference/06-rust-bindings.md) - WASM FFI
- [`07-python-bindings.md`](07-api-reference/07-python-bindings.md) - PyO3 interface

**Główne pytanie:** *"Jakie metody są dostępne i jak ich używać?"*

---

### 08. Usage Guide
**Konkretne przykłady użycia**

Praktyczne przewodniki i przykłady.

- [`01-installation.md`](08-usage-guide/01-installation.md) - npm/pip install
- [`02-quick-start.md`](08-usage-guide/02-quick-start.md) - Hello World
- [`03-basic-operations.md`](08-usage-guide/03-basic-operations.md) - CRUD operations
- [`04-versioning-workflow.md`](08-usage-guide/04-versioning-workflow.md) - Workflow wersjonowania
- [`05-working-with-binaries.md`](08-usage-guide/05-working-with-binaries.md) - Obrazy/wideo
- [`06-streaming-large-files.md`](08-usage-guide/06-streaming-large-files.md) - Pliki >500MB
- [`07-platform-specific.md`](08-usage-guide/07-platform-specific.md) - Browser/Node/Tauri
- [`08-advanced-patterns.md`](08-usage-guide/08-advanced-patterns.md) - Zaawansowane wzorce

**Główne pytanie:** *"Jak rozwiązać konkretne problemy?"*

---

### 09. Edge Cases
**Co może pójść nie tak?**

Nietypowe scenariusze i ich rozwiązania.

- [`01-file-renames.md`](09-edge-cases/01-file-renames.md) - Obsługa rename
- [`02-type-changes.md`](09-edge-cases/02-type-changes.md) - Text ↔ Binary
- [`03-orphaned-blobs.md`](09-edge-cases/03-orphaned-blobs.md) - GC scenariusze
- [`04-corrupted-data.md`](09-edge-cases/04-corrupted-data.md) - Recovery
- [`05-memory-limits.md`](09-edge-cases/05-memory-limits.md) - RAM constraints
- [`06-concurrent-access.md`](09-edge-cases/06-concurrent-access.md) - Multi-process

**Główne pytanie:** *"Co się stanie w nietypowych sytuacjach?"*

---

### 10. Development
**Jak rozwijać projekt?**

Przewodnik dla kontrybutorów.

- [`01-setup-environment.md`](10-development/01-setup-environment.md) - Setup środowiska
- [`02-building-from-source.md`](10-development/02-building-from-source.md) - Kompilacja Rust → WASM
- [`03-running-tests.md`](10-development/03-running-tests.md) - Test suite
- [`04-debugging-guide.md`](10-development/04-debugging-guide.md) - Debugging
- [`05-contributing.md`](10-development/05-contributing.md) - Contributing guide
- [`06-implementation-roadmap.md`](10-development/06-implementation-roadmap.md) - Roadmap 6-tygodniowy

**Główne pytanie:** *"Jak mogę pomóc w rozwoju projektu?"*

---

## 🗺️ Kluczowe Diagramy

Szybki dostęp do najważniejszych wizualizacji:

- **System Architecture**: [`03-architecture/01-system-overview.md`](03-architecture/01-system-overview.md)
- **Data Flow (Save/Restore)**: [`03-architecture/03-data-flow.md`](03-architecture/03-data-flow.md)
- **Rust vs TypeScript Matrix**: [`04-technical-decisions/01-rust-vs-typescript.md`](04-technical-decisions/01-rust-vs-typescript.md)
- **Save Algorithm**: [`05-algorithms/01-save-checkpoint.md`](05-algorithms/01-save-checkpoint.md)
- **Restore Algorithm**: [`05-algorithms/02-restore-version.md`](05-algorithms/02-restore-version.md)

---

## 🔗 Quick Links

- **API Reference**: [`07-api-reference/01-jcf-manager-class.md`](07-api-reference/01-jcf-manager-class.md)
- **Quick Start**: [`08-usage-guide/02-quick-start.md`](08-usage-guide/02-quick-start.md)
- **Installation**: [`08-usage-guide/01-installation.md`](08-usage-guide/01-installation.md)
- **Troubleshooting**: [`09-edge-cases/04-corrupted-data.md`](09-edge-cases/04-corrupted-data.md)
- **Contributing**: [`10-development/05-contributing.md`](10-development/05-contributing.md)

---

## 📊 Statystyki Dokumentacji

- **Rozdziały**: 10
- **Pliki**: ~65
- **Średni czas czytania (pełna dokumentacja)**: ~6-8 godzin
- **Szybki start**: ~15 minut

---

## 🎯 Struktura Narracji

```
Problem Definition (01)
         ↓
Core Concepts (02)
         ↓
Architecture Design (03)
         ↓
Technical Justifications (04)
         ↓
Detailed Algorithms (05)
         ↓
Implementation Guide (06)
         ↓
API Documentation (07)
         ↓
Practical Usage (08)
         ↓
Edge Cases (09)
         ↓
Development (10)
```

---

**Zaczynamy?** → [`01-introduction/01-overview.md`](01-introduction/01-overview.md)
