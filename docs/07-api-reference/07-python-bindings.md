# Python API Bindings (PyO3)

## Przegląd

Python bindings zapewniają pełny dostęp do funkcjonalności JCF Manager z poziomu Pythona. Używają PyO3 do wysokiej wydajności FFI między Python i Rust core.

## Architektura

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│     Python      │     │     PyO3         │     │      Rust       │
│   (asyncio)     │◄───►│   FFI Bridge     │◄───►│   Core Logic    │
│                 │     │   (pydantic)     │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
         └──── Python API ───────┴────── Native Calls ──┘
```

## Instalacja

### PyPI

```bash
pip install kamaros
```

### Z kodu źródłowego

```bash
git clone https://github.com/your-org/kamaros.git
cd kamaros
pip install -e .
```

### Wymagania systemowe

- Python 3.8+
- Rust 1.70+ (dla kompilacji)
- maturin (dla build)

## Podstawowe użycie

### Inicjalizacja

```python
import asyncio
from kamaros import JCFManager, FileAdapter

async def main():
    # Inicjalizacja managera
    manager = JCFManager()

    # Inicjalizacja adaptera systemu plików
    adapter = FileAdapter("./my-project")
    await manager.init(adapter)

    print("JCF Manager initialized")

asyncio.run(main())
```

### Operacje na plikach

```python
import asyncio
from kamaros import JCFManager, FileAdapter

async def file_operations():
    manager = JCFManager()
    await manager.init(FileAdapter("./project"))

    # Dodanie plików
    await manager.add_file("README.md", b"# My Project")
    await manager.add_file("src/main.py", b'print("Hello World")')

    # Dodanie pliku tekstowego z autodetekcją encoding
    await manager.add_file("config.json", '{"debug": true}')

    # Lista plików
    files = await manager.list_files()
    print("Project files:", files)

    # Odczyt pliku
    content = await manager.get_file_content("README.md")
    print("README content:", content.decode('utf-8'))

asyncio.run(file_operations())
```

## Klasy i API

### JCFManager

Główna klasa Python API.

```python
from kamaros import JCFManager
from typing import Optional, Dict, Any, List

class JCFManager:
    def __init__(self, config: Optional[JCFConfig] = None):
        """Inicjalizacja managera z opcjonalną konfiguracją"""

    async def init(self, adapter: FileSystemAdapter) -> None:
        """Inicjalizacja z adapterem systemu plików"""

    async def save_checkpoint(self, message: str, **options) -> str:
        """Tworzy checkpoint z opcjami"""

    async def restore_version(self, version_id: str, **options) -> None:
        """Przywraca wersję"""

    async def add_file(self, path: str, content: Union[bytes, str], **options) -> None:
        """Dodaje plik"""

    async def remove_file(self, path: str, **options) -> None:
        """Usuwa plik"""

    async def get_file_content(self, path: str, version_id: Optional[str] = None) -> bytes:
        """Pobiera zawartość pliku"""

    async def list_files(self, directory: str = "", **options) -> List[FileInfo]:
        """Lista plików w katalogu"""

    async def get_history(self, **options) -> List[Version]:
        """Pobiera historię wersji"""

    async def get_version(self, version_id: str) -> Optional[Version]:
        """Pobiera szczegóły wersji"""

    async def get_stats(self) -> ProjectStats:
        """Pobiera statystyki projektu"""

    async def run_gc(self, **options) -> GCReport:
        """Uruchamia garbage collection"""

    async def verify_integrity(self, **options) -> VerificationReport:
        """Sprawdza integralność"""

    async def export(self, **options) -> AsyncIterable[bytes]:
        """Eksportuje projekt jako stream"""

    async def import_data(self, stream: AsyncIterable[bytes], **options) -> None:
        """Importuje projekt ze streama"""

    # Event system
    def on(self, event: str, callback: Callable) -> None:
        """Subskrypcja zdarzeń"""

    def off(self, event: str, callback: Optional[Callable] = None) -> None:
        """Usunięcie subskrypcji"""
```

### FileSystemAdapter

Abstrakcja systemu plików dla Pythona.

```python
from abc import ABC, abstractmethod
from typing import List, Optional, AsyncIterable

class FileSystemAdapter(ABC):
    @property
    def supports_streaming(self) -> bool:
        """Czy adapter wspiera streaming"""

    @property
    def max_file_size(self) -> int:
        """Maksymalny rozmiar pliku"""

    async def init(self) -> None:
        """Inicjalizacja adaptera"""

    async def dispose(self) -> None:
        """Zwolnienie zasobów"""

    @abstractmethod
    async def read_file(self, path: str) -> bytes:
        """Odczyt pliku"""

    @abstractmethod
    async def write_file(self, path: str, data: bytes) -> None:
        """Zapis pliku"""

    @abstractmethod
    async def file_exists(self, path: str) -> bool:
        """Sprawdzenie czy plik istnieje"""

    @abstractmethod
    async def delete_file(self, path: str) -> None:
        """Usunięcie pliku"""

    @abstractmethod
    async def list_files(self, directory: str) -> List[str]:
        """Lista plików w katalogu"""
```

### FileAdapter

Domyślny adapter dla systemu plików.

```python
from pathlib import Path
from kamaros import FileAdapter

class FileAdapter(FileSystemAdapter):
    def __init__(self, root_path: Union[str, Path], **options):
        """
        Args:
            root_path: Katalog root projektu
            create_if_missing: Czy utworzyć katalog jeśli nie istnieje
            enable_compression: Czy włączyć kompresję
        """

    @property
    def root_path(self) -> Path:
        """Ścieżka root projektu"""

    # Implementuje wszystkie metody FileSystemAdapter
```

### MemoryAdapter

Adapter in-memory dla testów.

```python
from kamaros import MemoryAdapter

class MemoryAdapter(FileSystemAdapter):
    def __init__(self):
        """Adapter przechowujący dane w pamięci"""

    def to_bytes(self) -> bytes:
        """Serializuje stan adaptera"""

    @classmethod
    def from_bytes(cls, data: bytes) -> 'MemoryAdapter':
        """Deserializuje stan adaptera"""
```

## Zaawansowane użycie

### Konfiguracja

```python
from kamaros import JCFManager, JCFConfig

config = JCFConfig(
    author="Jan Kowalski",
    email="jan@example.com",
    compression_level=9,
    auto_gc=True,
    max_file_size=500 * 1024 * 1024,  # 500MB
    snapshot_interval=50
)

manager = JCFManager(config)
```

### Operacje na wersjach

```python
async def version_management():
    manager = JCFManager()
    await manager.init(FileAdapter("./project"))

    # Tworzenie wersji
    v1 = await manager.save_checkpoint("Initial setup")
    print(f"Created version: {v1}")

    # Dodanie zmian
    await manager.add_file("feature.py", b"def new_feature(): pass")
    v2 = await manager.save_checkpoint("Add new feature")

    # Historia
    history = await manager.get_history()
    for version in history:
        print(f"{version.id}: {version.message} by {version.author}")

    # Przywracanie
    await manager.restore_version(v1)
    print("Restored to initial version")
```

### Streaming i duże pliki

```python
import aiofiles

async def handle_large_files():
    manager = JCFManager()
    await manager.init(FileAdapter("./project"))

    # Odczyt dużego pliku przez streaming
    async def read_large_file():
        async for chunk in manager.stream_file("large-video.mp4"):
            # Przetwarzaj chunk
            process_chunk(chunk)

    # Zapis dużego pliku
    async with aiofiles.open("large-file.dat", "rb") as f:
        async for chunk in read_file_in_chunks(f):
            await manager.append_to_file("large-file.dat", chunk)

    # Eksport projektu
    async with aiofiles.open("backup.jcf", "wb") as f:
        async for chunk in manager.export():
            await f.write(chunk)
```

### Event handling

```python
from kamaros import JCFEvent

async def event_handling():
    manager = JCFManager()

    # Subskrypcja zdarzeń
    @manager.on('checkpoint:progress')
    def on_progress(event: JCFEvent):
        print(f"Progress: {event.percent}%")

    @manager.on('file:change')
    def on_file_change(event: JCFEvent):
        print(f"File {event.change_type}: {event.path}")

    # Operacje z callbackami
    await manager.save_checkpoint("Big operation")
```

## Typy danych

### Pydantic Models

Wszystkie typy używają Pydantic dla walidacji i serializacji.

```python
from pydantic import BaseModel
from typing import Optional, List, Dict, Any
from datetime import datetime

class JCFConfig(BaseModel):
    author: Optional[str] = None
    email: Optional[str] = None
    compression_level: int = 6
    auto_gc: bool = False
    max_file_size: Optional[int] = None
    snapshot_interval: int = 50

class FileInfo(BaseModel):
    path: str
    size: int
    file_type: str  # 'text' | 'binary'
    hash: Optional[str] = None
    modified: Optional[datetime] = None
    encoding: Optional[str] = None
    mime: Optional[str] = None

class Version(BaseModel):
    id: str
    timestamp: datetime
    message: str
    author: str
    email: Optional[str] = None
    parent_id: Optional[str] = None
    file_states: Dict[str, FileState]
    tags: Optional[List[str]] = None

class ProjectStats(BaseModel):
    total_versions: int
    total_files: int
    total_size: int
    content_size: int
    blobs_size: int
    deltas_size: int
    unique_blobs: int
    blob_references: int
    deduplication_ratio: float
    oldest_version: Optional[Dict[str, Any]] = None
    newest_version: Optional[Dict[str, Any]] = None
```

## Obsługa błędów

### Hierarchia wyjątków

```python
from kamaros import JCFError, FileNotFoundError, ValidationError

class JCFError(Exception):
    """Bazowy wyjątek JCF"""
    def __init__(self, message: str, code: str, details: Optional[Dict] = None):
        self.code = code
        self.details = details or {}

class FileNotFoundError(JCFError):
    """Plik nie istnieje"""
    pass

class VersionNotFoundError(JCFError):
    """Wersja nie istnieje"""
    pass

class ValidationError(JCFError):
    """Błąd walidacji danych"""
    pass

class StorageError(JCFError):
    """Błąd systemu plików"""
    pass

class CorruptionError(JCFError):
    """Uszkodzone dane"""
    pass
```

### Obsługa wyjątków

```python
async def robust_operations():
    manager = JCFManager()
    await manager.init(FileAdapter("./project"))

    try:
        await manager.restore_version("nonexistent-version")
    except VersionNotFoundError as e:
        print(f"Version not found: {e.details.get('version_id')}")
    except StorageError as e:
        print(f"Storage error: {e}")
        # Spróbuj recovery
        await manager.verify_integrity(repair=True)
    except JCFError as e:
        print(f"JCF error [{e.code}]: {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")
```

## Wydajność

### Async/Await

Wszystkie operacje są asynchroniczne:

```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

async def parallel_operations():
    manager = JCFManager()
    await manager.init(FileAdapter("./project"))

    # Parallel file processing
    async def process_file(filename):
        content = await manager.get_file_content(filename)
        return process_content(content)

    files = await manager.list_files()
    tasks = [process_file(f) for f in files]
    results = await asyncio.gather(*tasks)
```

### Memory Management

```python
# Dla dużych plików użyj streaming
async def stream_processing():
    manager = JCFManager()

    # Nie ładuj całego pliku do pamięci
    async for chunk in manager.stream_file("large-file.dat"):
        await process_chunk(chunk)
        del chunk  # Explicit memory cleanup
```

### Connection Pooling

```python
from kamaros import ConnectionPool

# Dla wielu operacji użyj pool
async with ConnectionPool(FileAdapter("./project")) as pool:
    manager = JCFManager()
    await manager.init(pool.get_adapter())

    # Pool zarządza połączeniami automatycznie
```

## Testowanie

### Unit Tests

```python
import pytest
from kamaros import MemoryAdapter, JCFManager

@pytest.fixture
async def manager():
    manager = JCFManager()
    await manager.init(MemoryAdapter())
    return manager

@pytest.mark.asyncio
async def test_file_operations(manager):
    await manager.add_file("test.txt", b"Hello World")
    content = await manager.get_file_content("test.txt")
    assert content == b"Hello World"

@pytest.mark.asyncio
async def test_versioning(manager):
    v1 = await manager.save_checkpoint("Initial")
    await manager.add_file("file.txt", b"content")
    v2 = await manager.save_checkpoint("Added file")

    history = await manager.get_history()
    assert len(history) == 2
    assert history[0].id == v2
    assert history[1].id == v1
```

### Integration Tests

```python
import tempfile
from pathlib import Path

@pytest.fixture
async def temp_project():
    with tempfile.TemporaryDirectory() as tmpdir:
        manager = JCFManager()
        await manager.init(FileAdapter(tmpdir))
        yield manager

@pytest.mark.asyncio
async def test_persistence(temp_project):
    await temp_project.add_file("data.txt", b"persistent data")
    await temp_project.save_checkpoint("Save data")

    # Symuluj restart aplikacji
    new_manager = JCFManager()
    await new_manager.init(FileAdapter(temp_project.adapter.root_path))

    content = await new_manager.get_file_content("data.txt")
    assert content == b"persistent data"
```

## Rozszerzenia

### Custom Adapters

```python
from kamaros import FileSystemAdapter
from typing import List

class S3Adapter(FileSystemAdapter):
    def __init__(self, bucket_name: str, prefix: str = ""):
        self.bucket = bucket_name
        self.prefix = prefix
        self.client = None

    async def init(self):
        # Initialize S3 client
        import aioboto3
        self.client = aioboto3.Session().client('s3')

    async def read_file(self, path: str) -> bytes:
        key = f"{self.prefix}/{path}"
        response = await self.client.get_object(Bucket=self.bucket, Key=key)
        return await response['Body'].read()

    async def write_file(self, path: str, data: bytes) -> None:
        key = f"{self.prefix}/{path}"
        await self.client.put_object(Bucket=self.bucket, Key=key, Body=data)

    # Implement other methods...
```

### Middleware

```python
from kamaros import JCFManager

class LoggingMiddleware:
    def __init__(self, manager: JCFManager):
        self.manager = manager

    async def add_file(self, path: str, content, **options):
        print(f"Adding file: {path}")
        result = await self.manager.add_file(path, content, **options)
        print(f"File added successfully")
        return result

# Użycie
manager = LoggingMiddleware(JCFManager())
```

## Troubleshooting

### Common Issues

1. **MemoryError**: Użyj streaming dla dużych plików
2. **TimeoutError**: Zwiększ `operation_timeout` w konfiguracji
3. **FileNotFoundError**: Sprawdź ścieżki i uprawnienia
4. **CorruptionError**: Uruchom `verify_integrity(repair=True)`

### Debugowanie

```python
import logging

# Włącz debug logging
logging.basicConfig(level=logging.DEBUG)

# Lub w konfiguracji
config = JCFConfig(debug=True)
manager = JCFManager(config)
```

### Performance Monitoring

```python
from kamaros import PerformanceMonitor

monitor = PerformanceMonitor()

@monitor.timed
async def slow_operation():
    # Operacja do zmierzenia
    pass

# Pobierz metryki
stats = monitor.get_stats()
print(f"Average operation time: {stats.average_time}ms")
```

---

**Zobacz również:**
- [JCFManager Class](01-jcf-manager-class.md) - Główny interfejs API
- [TypeScript Types](05-typescript-types.md) - Typy używane przez Python bindings
- [Rust Bindings](06-rust-bindings.md) - Core implementacja w Rust