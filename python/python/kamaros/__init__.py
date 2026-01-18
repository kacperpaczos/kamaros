"""
Kamaros Python Library

High-level API for managing JCF (JSON Content Format) files.

Example:
    >>> from kamaros import JCFManager, MemoryAdapter
    >>> manager = JCFManager(MemoryAdapter())
    >>> manager.create_project("MyProject")
    >>> manager.add_file("main.py", b"print('Hello')")
    >>> manager.save("project.jcf")
"""

from .manager import JCFManager
from .adapters import MemoryAdapter, FileAdapter

# Import native functions from Rust (when built)
try:
    from kamaros._native import version, greet, create_empty_manifest, get_manifest_info
except ImportError:
    # Fallback for development without native module
    def version() -> str:
        return "0.1.0 (pure Python)"
    
    def greet(name: str) -> str:
        return f"Hello from Kamaros Python, {name}!"
    
    def create_empty_manifest(project_name: str) -> dict:
        from datetime import datetime
        now = datetime.now().isoformat()
        return {
            "format_version": "1.0.0",
            "metadata": {
                "name": project_name,
                "created": now,
                "last_modified": now,
            },
            "file_map": {},
            "version_history": [],
            "refs": {"head": ""},
            "rename_log": [],
        }
    
    def get_manifest_info(manifest: dict) -> dict:
        return {
            "name": manifest.get("metadata", {}).get("name", ""),
            "version_count": len(manifest.get("version_history", [])),
            "file_count": len(manifest.get("file_map", {})),
        }

__all__ = [
    "JCFManager",
    "MemoryAdapter",
    "FileAdapter",
    "version",
    "greet",
    "create_empty_manifest",
    "get_manifest_info",
]

__version__ = "0.1.0"
