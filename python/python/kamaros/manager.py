"""
JCFManager - High-level API for JCF file operations
"""

from typing import Optional, Dict, Any
import zipfile
import json
import io

from . import create_empty_manifest


class JCFManager:
    """
    Main class for managing JCF files.
    
    Example:
        >>> manager = JCFManager(MemoryAdapter())
        >>> manager.create_project("MyProject")
        >>> manager.add_file("main.py", b"print('Hello')")
        >>> manager.save("project.jcf")
    """
    
    def __init__(self, adapter: "StorageAdapter"):
        self.adapter = adapter
        self.manifest: Optional[Dict[str, Any]] = None
        self.working_dir: Dict[str, bytes] = {}
    
    def create_project(self, name: str, description: Optional[str] = None, author: Optional[str] = None) -> None:
        """Create a new empty project."""
        self.manifest = create_empty_manifest(name)
        if description:
            self.manifest["metadata"]["description"] = description
        if author:
            self.manifest["metadata"]["author"] = author
        self.working_dir = {}
    
    def load(self, path: str) -> None:
        """Load a JCF file from storage."""
        data = self.adapter.read(path)
        
        with zipfile.ZipFile(io.BytesIO(data), 'r') as zf:
            # Read manifest
            manifest_data = zf.read("manifest.json")
            self.manifest = json.loads(manifest_data.decode('utf-8'))
            
            # Load working directory
            self.working_dir = {}
            for name in zf.namelist():
                if name.startswith("content/"):
                    relative_path = name[len("content/"):]
                    if relative_path:
                        self.working_dir[relative_path] = zf.read(name)
    
    def save(self, path: str) -> None:
        """Save JCF file to storage."""
        if self.manifest is None:
            raise ValueError("No project loaded. Call create_project() or load() first.")
        
        # Update timestamp
        from datetime import datetime
        self.manifest["metadata"]["last_modified"] = datetime.now().isoformat()
        
        # Create ZIP in memory
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, 'w', zipfile.ZIP_DEFLATED) as zf:
            # Write mimetype
            zf.writestr("mimetype", "application/x-jcf")
            
            # Write manifest
            zf.writestr("manifest.json", json.dumps(self.manifest, indent=2))
            
            # Write working directory
            for file_path, data in self.working_dir.items():
                zf.writestr(f"content/{file_path}", data)
        
        self.adapter.write(path, buffer.getvalue())
    
    def add_file(self, path: str, content: bytes) -> None:
        """Add or update a file in the working directory."""
        if self.manifest is None:
            raise ValueError("No project loaded.")
        
        self.working_dir[path] = content
        
        # Update file map
        from datetime import datetime
        import uuid
        
        now = datetime.now().isoformat()
        if path not in self.manifest["file_map"]:
            self.manifest["file_map"][path] = {
                "inode_id": str(uuid.uuid4()),
                "type": "text" if self._is_text_file(path) else "binary",
                "created": now,
                "modified": now,
            }
        else:
            self.manifest["file_map"][path]["modified"] = now
    
    def get_file(self, path: str) -> Optional[bytes]:
        """Get a file from working directory."""
        return self.working_dir.get(path)
    
    def delete_file(self, path: str) -> bool:
        """Delete a file from working directory."""
        if path in self.working_dir:
            del self.working_dir[path]
            if self.manifest and path in self.manifest["file_map"]:
                del self.manifest["file_map"][path]
            return True
        return False
    
    def list_files(self) -> list:
        """List all files in working directory."""
        return list(self.working_dir.keys())
    
    def get_manifest(self) -> Optional[Dict[str, Any]]:
        """Get current manifest."""
        return self.manifest
    
    def get_project_info(self) -> Optional[Dict[str, Any]]:
        """Get project info."""
        if self.manifest is None:
            return None
        return {
            "name": self.manifest["metadata"]["name"],
            "version_count": len(self.manifest["version_history"]),
            "file_count": len(self.manifest["file_map"]),
        }
    
    def _is_text_file(self, path: str) -> bool:
        """Check if file is text based on extension."""
        text_extensions = ['.txt', '.md', '.json', '.js', '.ts', '.css', '.html', '.xml', '.yaml', '.yml', '.py']
        return any(path.lower().endswith(ext) for ext in text_extensions)


class StorageAdapter:
    """Base class for storage adapters."""
    
    def read(self, path: str) -> bytes:
        raise NotImplementedError
    
    def write(self, path: str, data: bytes) -> None:
        raise NotImplementedError
    
    def delete(self, path: str) -> None:
        raise NotImplementedError
    
    def exists(self, path: str) -> bool:
        raise NotImplementedError
    
    def list(self, dir: str) -> list:
        raise NotImplementedError
