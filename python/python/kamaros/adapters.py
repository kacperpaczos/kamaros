"""
Storage adapters for Kamaros
"""

import os
from pathlib import Path
from typing import Dict

from .manager import StorageAdapter


class MemoryAdapter(StorageAdapter):
    """In-memory storage adapter for testing."""
    
    def __init__(self):
        self._storage: Dict[str, bytes] = {}
    
    def read(self, path: str) -> bytes:
        if path not in self._storage:
            raise FileNotFoundError(f"File not found: {path}")
        return self._storage[path]
    
    def write(self, path: str, data: bytes) -> None:
        self._storage[path] = data
    
    def delete(self, path: str) -> None:
        if path in self._storage:
            del self._storage[path]
    
    def exists(self, path: str) -> bool:
        return path in self._storage
    
    def list(self, dir: str, recursive: bool = True) -> list:
        prefix = dir.rstrip('/') + '/'
        if recursive:
            return [
                key[len(prefix):]
                for key in self._storage
                if key.startswith(prefix)
            ]
        else:
            return list(set(
                key[len(prefix):].split('/')[0]
                for key in self._storage
                if key.startswith(prefix)
            ))
    
    def size(self, path: str) -> int:
        if path not in self._storage:
            raise FileNotFoundError(f"File not found: {path}")
        return len(self._storage[path])

    def list_blobs(self) -> list:
        """List all blobs in .store/blobs/."""
        prefix = ".store/blobs/"
        return [key[len(prefix):] for key in self._storage if key.startswith(prefix)]

    def clear(self) -> None:
        """Clear all stored data."""
        self._storage.clear()


class FileAdapter(StorageAdapter):
    """File system storage adapter."""
    
    def __init__(self, base_path: str):
        self.base_path = Path(base_path)
        self.base_path.mkdir(parents=True, exist_ok=True)
    
    def read(self, path: str) -> bytes:
        full_path = self.base_path / path
        return full_path.read_bytes()
    
    def write(self, path: str, data: bytes) -> None:
        full_path = self.base_path / path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_bytes(data)
    
    def delete(self, path: str) -> None:
        full_path = self.base_path / path
        if full_path.exists():
            full_path.unlink()
    
    def exists(self, path: str) -> bool:
        return (self.base_path / path).exists()
    
    def list(self, dir: str) -> list:
        full_path = self.base_path / dir
        if not full_path.exists():
            return []
        # Recursive listing for JCF compatibility
        files = []
        for f in full_path.rglob('*'):
            if f.is_file():
                files.append(str(f.relative_to(full_path)))
        return files

    def size(self, path: str) -> int:
        full_path = self.base_path / path
        return full_path.stat().st_size

    def list_blobs(self) -> list:
        """List all blobs in .store/blobs/."""
        blob_path = self.base_path / ".store" / "blobs"
        if not blob_path.exists():
            return []
        return [f.name for f in blob_path.iterdir() if f.is_file()]
