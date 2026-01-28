import pytest
import os
from kamaros import JCFManager

# Try to import FileAdapter; if not available, some tests might need skipping or fallback,
# but integration tests usually assume real environment availability.
try:
    from kamaros import FileAdapter
except ImportError:
    FileAdapter = None

@pytest.mark.skipif(FileAdapter is None, reason="FileAdapter required for integration tests")
def test_full_workflow(tmp_path):
    """Integration test: Create, Add, Save Checkpoint, Restore."""
    project_dir = tmp_path / "project"
    project_dir.mkdir()
    
    adapter = FileAdapter(str(project_dir))
    manager = JCFManager(adapter)
    
    manager.create_project("IntegrationProj")
    
    # Version 1
    manager.add_file("data.txt", b"v1")
    v1_id = manager.save_checkpoint("Commit 1")
    
    # Version 2
    manager.add_file("data.txt", b"v2")
    v2_id = manager.save_checkpoint("Commit 2")
    
    assert v1_id != v2_id
    
    # Verify history is persisted and retrieval works
    history = manager.get_file_history("data.txt")
    assert len(history) == 2
    # Verify ordering (newest to oldest or vice-versa depending on impl)
    # Our implementation returns oldest first based on previous debugging.
    assert history[0]["version_id"] == v1_id
    assert history[1]["version_id"] == v2_id

    # Restore v1
    manager.restore_version(v1_id)
    assert manager.get_file("data.txt") == b"v1"

@pytest.mark.skipif(FileAdapter is None, reason="FileAdapter required for integration tests")
def test_portable_export_import(tmp_path):
    """Integration test: Save portable archive (.jcf) and load in different location."""
    origin_dir = tmp_path / "origin"
    origin_dir.mkdir()
    dest_dir = tmp_path / "dest"
    dest_dir.mkdir()
    
    # 1. Create project with blobs
    mgr1 = JCFManager(FileAdapter(str(origin_dir)))
    mgr1.create_project("PortableProj")
    mgr1.add_file("image.bin", b"\x00\xFF\x00") # Binary content
    mgr1.save_checkpoint("Init")
    
    # 2. Export to JCF
    jcf_path = origin_dir / "export.jcf"
    mgr1.save(str(jcf_path))
    
    # 3. Load in fresh directory (simulation of another machine)
    mgr2 = JCFManager(FileAdapter(str(dest_dir)))
    mgr2.load(str(jcf_path))
    
    # 4. Verify content matches
    assert mgr2.get_file("image.bin") == b"\x00\xFF\x00"
    
    # 5. Verify integrity check works on imported project
    integrity = mgr2.verify_integrity()
    assert integrity["valid"] == True
    assert integrity["checked"] >= 1
