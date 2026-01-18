import pytest
import os
import sys
from kamaros.manager import JCFManager
from kamaros.adapters import MemoryAdapter

# Ensure we can import the new Rust module
# This assumes the .so file is available in the python path or installed
# For local dev without install, we might need to point to target/debug

def test_save_checkpoint_flow():
    adapter = MemoryAdapter()
    manager = JCFManager(adapter)
    
    # 1. Create Project
    manager.create_project("PyCheckpointTest")
    
    # 2. Add file
    manager.add_file("main.py", b"print('v1')")
    
    # 3. Save Checkpoint
    try:
        version_id = manager.save_checkpoint("Initial commit", author="PyTester")
    except ImportError:
        pytest.skip("kamaros Rust module not available")
        return
        
    assert version_id is not None
    assert len(version_id) > 0
    
    # 4. Verify Manifest
    manifest = manager.get_manifest()
    assert len(manifest["versionHistory"]) == 1
    assert manifest["versionHistory"][0]["id"] == version_id
    assert manifest["versionHistory"][0]["message"] == "Initial commit"
    assert "main.py" in manifest["versionHistory"][0]["fileStates"]
    assert manifest["refs"]["head"] == version_id
    
    # Verify Blob Persistence
    file_state = manifest["versionHistory"][0]["fileStates"]["main.py"]
    assert file_state["contentRef"] is not None
    assert file_state["contentRef"].startswith("blobs/")
    
    blob_path = f".store/{file_state['contentRef']}"
    assert adapter.exists(blob_path)
    # Verify content match
    stored_content = adapter.read(blob_path)
    assert stored_content == b"print('v1')"

def test_multiple_versions():
    adapter = MemoryAdapter()
    manager = JCFManager(adapter)
    manager.create_project("MultiVer")
    
    try:
        # V1
        manager.add_file("readme.md", b"# V1")
        v1 = manager.save_checkpoint("Commit 1")
        
        # V2
        manager.add_file("readme.md", b"# V2")
        manager.add_file("new.txt", b"New file")
        v2 = manager.save_checkpoint("Commit 2")
    except ImportError:
        pytest.skip("kamaros Rust module not available")
    
    assert v1 != v2
    
    manifest = manager.get_manifest()
    assert len(manifest["versionHistory"]) == 2
    
    # Verify file states
    v2_state = manifest["versionHistory"][1]["fileStates"]
    assert "readme.md" in v2_state
    assert "new.txt" in v2_state

def test_restore_version():
    adapter = MemoryAdapter()
    manager = JCFManager(adapter)
    manager.create_project("RestoreTest")
    
    try:
        # V1: Only file1
        manager.add_file("file1.txt", b"Content 1")
        v1_id = manager.save_checkpoint("V1")
        
        # V2: file1 modified, file2 added
        manager.add_file("file1.txt", b"Content 1 Modified")
        manager.add_file("file2.txt", b"Content 2")
        v2_id = manager.save_checkpoint("V2")
        
        # V3: file1 deleted
        manager.delete_file("file1.txt")
        v3_id = manager.save_checkpoint("V3")
        
    except ImportError:
        pytest.skip("kamaros Rust module not available")

    # Current state: V3 (file2 only)
    assert manager.get_file("file1.txt") is None
    assert manager.get_file("file2.txt") == b"Content 2"
    
    # Restore V1 (file1 original, no file2)
    restored_id = manager.restore_version(v1_id)
    assert restored_id == v1_id
    assert manager.get_file("file1.txt") == b"Content 1"
    assert manager.get_file("file2.txt") is None
    
    # Restore V2 (file1 mod, file2)
    manager.restore_version(v2_id)
    assert manager.get_file("file1.txt") == b"Content 1 Modified"
    assert manager.get_file("file2.txt") == b"Content 2"
    
    # Restore V3 (file2 only)
    manager.restore_version(v3_id)
    assert manager.get_file("file1.txt") is None
    assert manager.get_file("file2.txt") == b"Content 2"
