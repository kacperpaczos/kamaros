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
    from kamaros._native import version, greet, create_empty_manifest, get_manifest_info, save_checkpoint, restore_version
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
            "formatVersion": "1.0.0",
            "metadata": {
                "name": project_name,
                "created": now,
                "lastModified": now,
            },
            "fileMap": {},
            "versionHistory": [],
            "refs": {"head": ""},
            "renameLog": [],
        }
    
    def get_manifest_info(manifest: dict) -> dict:
        return {
            "name": manifest.get("metadata", {}).get("name", ""),
            "version_count": len(manifest.get("versionHistory", [])),
            "file_count": len(manifest.get("fileMap", {})),
        }

    def save_checkpoint(manifest: dict, working_dir: dict, message: str, author: str) -> dict:
        raise NotImplementedError("Native module not found. Checkpoint saving requires compiled Rust extension.")

    def restore_version(manifest: dict, current_files: list, version_id: str) -> dict:
        raise NotImplementedError("Native module not found.")

__all__ = [
    "JCFManager",
    "MemoryAdapter",
    "FileAdapter",
    "version",
    "greet",
    "create_empty_manifest",
    "get_manifest_info",
    "save_checkpoint",
    "restore_version",
]

__version__ = "0.1.0"
