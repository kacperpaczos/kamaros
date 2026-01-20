"""
JCFManager - High-level API for JCF file operations
"""

from typing import Optional, Dict, Any
import zipfile
import json
import io
from datetime import datetime


def _create_empty_manifest(project_name: str) -> dict:
    """Create an empty manifest (internal helper)."""
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
        self.manifest = _create_empty_manifest(name)
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
        self.manifest["metadata"]["lastModified"] = datetime.now().isoformat()
        
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
        if path not in self.manifest["fileMap"]:
            self.manifest["fileMap"][path] = {
                "inodeId": str(uuid.uuid4()),
                "type": "text" if self._is_text_file(path) else "binary",
                "created": now,
                "modified": now,
            }
        else:
            self.manifest["fileMap"][path]["modified"] = now
    
    def get_file(self, path: str) -> Optional[bytes]:
        """Get a file from working directory."""
        return self.working_dir.get(path)
    
    def delete_file(self, path: str) -> bool:
        """Delete a file from working directory."""
        if path in self.working_dir:
            del self.working_dir[path]
            # save_checkpoint will handle fileMap update
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
            "version_count": len(self.manifest["versionHistory"]),
            "file_count": len(self.manifest["fileMap"]),
        }
    
    def save_checkpoint(self, message: str, author: str = "unknown") -> str:
        """
        Create a new version (checkpoint) of the project.
        Uses native Rust implementation for performance.
        """
        if self.manifest is None:
            raise ValueError("No project loaded. Call create_project() or load() first.")
            
        import kamaros
        
        result = kamaros.save_checkpoint(
            self.manifest,
            self.working_dir,
            message,
            author
        )
        
        # Persist blobs (snapshot storage)
        if "blobs" in result:
            for path, content in result["blobs"]:
                # Content is [u8] from Rust, which is list of ints in Python if not PyBytes?
                # pythonize converts Vec<u8> to list of integers usually unless configured?
                # Actually, Vec<u8> via serde/pythonize -> list[int].
                # We need to convert to bytes.
                data = bytes(content)
                self.adapter.write(path, data)
        
        # Update local manifest
        self.manifest = result["manifest"]
        
        return result["version_id"]

    def restore_version(self, version_id: str) -> str:
        """
        Restore project to a specific version.
        """
        if self.manifest is None:
            raise ValueError("No project loaded.")
            
        import kamaros
        
        current_files = self.list_files()
        
        result = kamaros.restore_version(
            self.manifest,
            current_files,
            version_id
        )
        
        # Execute Restoration Plan
        
        # 1. Delete files
        for path in result["files_to_delete"]:
            self.delete_file(path)
            if path in self.working_dir:
                del self.working_dir[path]
        
        # 2. Restore files
        for path, blob_ref in result["files_to_restore"].items():
            # Read blob (handle .store prefix if needed)
            # Rust returns full path e.g. .store/blobs/hash
            if not self.adapter.exists(blob_ref):
                 # Fallback check? Maybe path is relative?
                 # My MemoryAdapter stores paths exactly as given.
                 pass
            
            content = self.adapter.read(blob_ref)
            # Convert [u8] to bytes if needed? 
            # Adapter.read() returns bytes usually.
            # But wait, did I implementation MemoryAdapter properly?
            # Yes, MemoryAdapter stores bytes.
            self.working_dir[path] = content
            
        # 3. Update Manifest
        self.manifest = result["manifest"]
        
        return result["restored_version_id"]

    def get_version_info(self, version_id: str) -> Optional[Dict[str, Any]]:
        """
        Get detailed information about a specific version.
        
        Returns:
            dict with: id, message, timestamp, author, parent_id, file_states
        """
        if self.manifest is None:
            return None
        
        for version in self.manifest.get("versionHistory", []):
            if version["id"] == version_id:
                return {
                    "id": version["id"],
                    "message": version.get("message", ""),
                    "timestamp": version.get("timestamp", ""),
                    "author": version.get("author", "unknown"),
                    "parent_id": version.get("parentId"),
                    "file_states": version.get("fileStates", {}),
                    "file_count": len(version.get("fileStates", {})),
                }
        return None

    def get_file_at_version(self, path: str, version_id: str) -> Optional[bytes]:
        """
        Get file content as it was in a specific version.
        
        Args:
            path: File path
            version_id: Version ID to read from
            
        Returns:
            File content as bytes, or None if not found
        """
        if self.manifest is None:
            return None
        
        # Find version
        version_info = self.get_version_info(version_id)
        if version_info is None:
            return None
        
        file_states = version_info.get("file_states", {})
        if path not in file_states:
            return None
        
        # Get blob reference (Rust uses contentRef, not blobRef)
        file_state = file_states[path]
        blob_ref = file_state.get("contentRef")
        if not blob_ref:
            # Fallback to blobRef for compatibility
            blob_ref = file_state.get("blobRef")
        if not blob_ref:
            return None
        
        # Add .store prefix if needed
        full_path = f".store/{blob_ref}" if not blob_ref.startswith(".store") else blob_ref
        
        # Read blob from storage
        try:
            return self.adapter.read(full_path)
        except Exception:
            return None

    def rename_file(self, old_path: str, new_path: str) -> bool:
        """
        Rename a file with history tracking.
        
        The rename is logged in renameLog for tracking file identity across versions.
        
        Returns:
            True if successful, False otherwise
        """
        if self.manifest is None:
            return False
        
        if old_path not in self.working_dir:
            return False
        
        if new_path in self.working_dir:
            return False  # Target exists
        
        # Move content
        content = self.working_dir[old_path]
        del self.working_dir[old_path]
        self.working_dir[new_path] = content
        
        # Update fileMap
        if old_path in self.manifest["fileMap"]:
            file_entry = self.manifest["fileMap"][old_path]
            del self.manifest["fileMap"][old_path]
            file_entry["modified"] = datetime.now().isoformat()
            self.manifest["fileMap"][new_path] = file_entry
        
        # Log rename (use field names matching Rust RenameEntry: from, to, timestamp, versionId)
        # Note: versionId will be set on next checkpoint, for now use empty string
        self.manifest["renameLog"].append({
            "from": old_path,
            "to": new_path,
            "timestamp": datetime.now().isoformat(),
            "versionId": "",  # Will be updated on next save_checkpoint
        })
        
        return True

    def get_file_history(self, path: str) -> list:
        """
        Get modification history of a specific file.
        
        Returns:
            List of versions where this file was modified, with details
        """
        if self.manifest is None:
            return []
        
        history = []
        previous_blob = None
        
        for version in self.manifest.get("versionHistory", []):
            file_states = version.get("fileStates", {})
            if path in file_states:
                current_blob = file_states[path].get("contentRef") or file_states[path].get("blobRef")
                
                # Check if file changed
                if current_blob != previous_blob:
                    history.append({
                        "version_id": version["id"],
                        "message": version.get("message", ""),
                        "timestamp": version.get("timestamp", ""),
                        "action": "created" if previous_blob is None else "modified",
                        "blob_ref": current_blob,
                    })
                    previous_blob = current_blob
            elif previous_blob is not None:
                # File was deleted in this version
                history.append({
                    "version_id": version["id"],
                    "message": version.get("message", ""),
                    "timestamp": version.get("timestamp", ""),
                    "action": "deleted",
                    "blob_ref": None,
                })
                previous_blob = None
        
        return history

    def compare_versions(self, v1_id: str, v2_id: str) -> Dict[str, Any]:
        """
        Compare two versions and return differences.
        
        Returns:
            dict with:
                - added: files added in v2
                - removed: files removed in v2
                - modified: files changed between v1 and v2
                - unchanged: files same in both versions
        """
        if self.manifest is None:
            return {"error": "No project loaded"}
        
        v1_info = self.get_version_info(v1_id)
        v2_info = self.get_version_info(v2_id)
        
        if v1_info is None or v2_info is None:
            return {"error": "Version not found"}
        
        v1_files = v1_info.get("file_states", {})
        v2_files = v2_info.get("file_states", {})
        
        v1_paths = set(v1_files.keys())
        v2_paths = set(v2_files.keys())
        
        added = list(v2_paths - v1_paths)
        removed = list(v1_paths - v2_paths)
        common = v1_paths & v2_paths
        
        modified = []
        unchanged = []
        
        for path in common:
            v1_blob = v1_files[path].get("contentRef") or v1_files[path].get("blobRef")
            v2_blob = v2_files[path].get("contentRef") or v2_files[path].get("blobRef")
            if v1_blob != v2_blob:
                modified.append(path)
            else:
                unchanged.append(path)
        
        return {
            "v1_id": v1_id,
            "v2_id": v2_id,
            "added": added,
            "removed": removed,
            "modified": modified,
            "unchanged": unchanged,
            "summary": f"+{len(added)} -{len(removed)} ~{len(modified)} ={len(unchanged)}"
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
