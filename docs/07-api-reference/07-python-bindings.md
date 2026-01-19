# Python API Bindings

## Przegląd

Python bindings zapewniają dostęp do funkcjonalności JCF Manager z poziomu Pythona. Kluczowe operacje obliczeniowe (różnice, hashowanie, aktualizacja manifestu) są delegowane do modułu natywnego `kamaros_rs` (napisanego w Rust i zbudowanego z PyO3) dla maksymalnej wydajności.

## Instalacja

Paczka wymaga zbudowania modułu natywnego.

### Z kodu źródłowego (Maturin)

```bash
# Wymaga instalacji maturin (pip install maturin)
maturin develop
# lub dla release
maturin develop --release
```

W przyszłości dostępne będzie `pip install kamaros`.

## API Reference

### JCFManager

Główna klasa do zarządzania projektem JCF. API jest w pełni **synchroniczne**.

```python
from kamaros import JCFManager, MemoryAdapter, FileAdapter

# Inicjalizacja z adapterem
manager = JCFManager(FileAdapter("./my-project"))

# 1. Utworzenie nowego projektu
manager.create_project("MyProject", author="Jan Kowalski")

# 2. Dodanie/Modyfikacja plików (w pamięci "Working Directory")
manager.add_file("README.md", b"# Hello World")
manager.add_file("src/main.py", b"print('Hi')")

# 3. Zapisanie wersji (Checkpoint)
# Oblicza hashe, deduplikuje dane i zapisuje nową wersję
version_id = manager.save_checkpoint("Initial commit")
print(f"Created version: {version_id}")

# 4. Sprawdzenie historii
manifest = manager.get_manifest()
for ver in manifest['versionHistory']:
    print(f"{ver['id']}: {ver['message']}")

# 5. Przywracanie wersji
# Przywraca stan plików w 'working directory' do wybranej wersji
manager.restore_version(old_version_id)
```

### Metody JCFManager

| Metoda | Opis |
|--------|------|
| `__init__(adapter)` | Tworzy instancję managera z danym adapterem pamięci. |
| `create_project(name, description?, author?)` | Inicjalizuje nowy pusty manifest projektu. |
| `load(path)` | Ładuje projekt z pliku `.jcf` (ZIP). |
| `save(path)` | Zapisuje cały projekt (manifest + content) do pliku `.jcf` (ZIP). |
| `add_file(path, content)` | Dodaje lub aktualizuje plik w wirtualnym katalogu roboczym. `content` to `bytes`. |
| `get_file(path)` | Zwraca zawartość pliku (`bytes`) lub `None`. |
| `delete_file(path)` | Usuwa plik z katalogu roboczego. |
| `save_checkpoint(message, author?)` | **Core**: Tworzy nowy commit. Używa Rust native module do obliczenia zmian i deduplikacji. Zwraca `version_id`. |
| `restore_version(version_id)` | **Core**: Przywraca stan projektu do podanej wersji (aktualizuje working dir i manifest). |
| `get_manifest()` | Zwraca słownik z pełnym manifestem JSON. |

### Adaptery (StorageAdapter)

Python API wykorzystuje wzorzec Adaptera do obsługi różnych źródeł danych.

#### MemoryAdapter
Przechowuje pliki w słowniku w pamięci RAM. Idealny do testów.

```python
from kamaros import MemoryAdapter
adapter = MemoryAdapter()
```

#### FileAdapter
Zapisuje dane fizycznie na dysku w podanym katalogu (np. `.store/blobs/...`).

```python
from kamaros import FileAdapter
adapter = FileAdapter("./fizyczny-folder-projektu")
```

Obsługuje metody: `read`, `write`, `delete`, `exists`, `list`.

## Moduł Natywny (`kamaros._native`)

Niskopoziomowe funkcje zaimplementowane w Rust (nie do bezpośredniego użycia, używaj `JCFManager`):

- `save_checkpoint(manifest, working_dir, message, author)` -> `SaveCheckpointResult`
- `restore_version(manifest, current_files, version_id)` -> `RestoreVersionResult`
- `version()` -> String (wersja lib)

Implementacja natywna zapewnia zgodność algorytmiczną z wersją TypeScript/WASM.