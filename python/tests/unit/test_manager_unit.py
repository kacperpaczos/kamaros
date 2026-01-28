import pytest
from kamaros import JCFManager, MemoryAdapter

@pytest.fixture
def manager():
    return JCFManager(MemoryAdapter())

def test_create_project(manager):
    """Unit test for JCFManager.create_project using MemoryAdapter."""
    manager.create_project("UnitTestProject", description="Unit test desc", author="Tester")
    
    assert manager.manifest["metadata"]["name"] == "UnitTestProject"
    assert manager.manifest["metadata"]["description"] == "Unit test desc"
    assert manager.manifest["metadata"]["author"] == "Tester"
    assert manager.manifest["formatVersion"] == "1.0.0"

def test_add_file_memory(manager):
    """Unit test for adding file to working directory (in memory)."""
    manager.create_project("MemoryProject")
    content = b"Unit Content"
    manager.add_file("unit.txt", content)
    
    # Verify content retrieved from working memory
    assert manager.get_file("unit.txt") == content

def test_add_file_validation(manager):
    """Unit test checking validation logic when adding files."""
    manager.create_project("ValidationProject")
    
    # Adding empty file should be allowed
    manager.add_file("empty.txt", b"")
    assert manager.get_file("empty.txt") == b""

    # Adding file with subdirectories
    manager.add_file("subdir/deep/test.txt", b"Deep")
    assert manager.get_file("subdir/deep/test.txt") == b"Deep"

def test_roadmap_tag_logic(manager):
    """Unit test for tag validation logic."""
    manager.create_project("TagLogic")
    manager.add_file("f.txt", b"v1")
    v1 = manager.save_checkpoint("v1")
    
    # Tagging missing version should fail (returns False)
    assert manager.tag_version("missing-id", "tag") == False
    
    # Tagging existing version
    assert manager.tag_version(v1, "release") == True
    
    # Duplicate tag should fail
    assert manager.tag_version(v1, "release") == False
