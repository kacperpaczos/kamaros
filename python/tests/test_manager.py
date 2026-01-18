"""
Tests for JCFManager

Each test focuses on a single aspect of JCFManager functionality.
"""

import pytest
from kamaros import JCFManager
from kamaros.adapters import MemoryAdapter


class TestJCFManagerHappyPath:
    """Happy path tests for JCFManager"""

    def test_create_project(self):
        """Test: Can create a new project"""
        manager = JCFManager(MemoryAdapter())
        
        manager.create_project("TestProject")
        
        info = manager.get_project_info()
        assert info is not None
        assert info["name"] == "TestProject"
        assert info["file_count"] == 0
        assert info["version_count"] == 0

    def test_create_project_with_options(self):
        """Test: Can create project with description and author"""
        manager = JCFManager(MemoryAdapter())
        
        manager.create_project("TestProject", description="Test description", author="Tester")
        
        manifest = manager.get_manifest()
        assert manifest is not None
        assert manifest["metadata"]["description"] == "Test description"
        assert manifest["metadata"]["author"] == "Tester"

    def test_add_file(self):
        """Test: Can add file to project"""
        manager = JCFManager(MemoryAdapter())
        manager.create_project("TestProject")
        
        manager.add_file("test.txt", b"Hello World")
        
        content = manager.get_file("test.txt")
        assert content == b"Hello World"

    def test_list_files(self):
        """Test: Can list files in project"""
        manager = JCFManager(MemoryAdapter())
        manager.create_project("TestProject")
        manager.add_file("file1.txt", b"content1")
        manager.add_file("file2.txt", b"content2")
        
        files = manager.list_files()
        
        assert len(files) == 2
        assert "file1.txt" in files
        assert "file2.txt" in files

    def test_delete_file(self):
        """Test: Can delete file from project"""
        manager = JCFManager(MemoryAdapter())
        manager.create_project("TestProject")
        manager.add_file("test.txt", b"content")
        
        result = manager.delete_file("test.txt")
        
        assert result is True
        assert manager.get_file("test.txt") is None

    def test_save_and_load_project(self):
        """Test: Can save and load project"""
        adapter = MemoryAdapter()
        manager = JCFManager(adapter)
        manager.create_project("TestProject")
        manager.add_file("main.py", b"print('Hello')")
        
        manager.save("project.jcf")
        
        # Load in new manager
        new_manager = JCFManager(adapter)
        new_manager.load("project.jcf")
        
        assert new_manager.get_project_info()["name"] == "TestProject"
        assert new_manager.get_file("main.py") == b"print('Hello')"


class TestJCFManagerEdgeCases:
    """Edge case tests for JCFManager"""

    def test_create_project_empty_name(self):
        """Edge case: Create project with empty name"""
        manager = JCFManager(MemoryAdapter())
        
        manager.create_project("")
        
        info = manager.get_project_info()
        assert info["name"] == ""

    def test_create_project_unicode_name(self):
        """Edge case: Create project with Unicode name"""
        manager = JCFManager(MemoryAdapter())
        
        manager.create_project("Проект 日本語 🚀")
        
        info = manager.get_project_info()
        assert info["name"] == "Проект 日本語 🚀"

    def test_get_nonexistent_file(self):
        """Edge case: Get file that doesn't exist returns None"""
        manager = JCFManager(MemoryAdapter())
        manager.create_project("TestProject")
        
        content = manager.get_file("nonexistent.txt")
        
        assert content is None

    def test_delete_nonexistent_file(self):
        """Edge case: Delete file that doesn't exist returns False"""
        manager = JCFManager(MemoryAdapter())
        manager.create_project("TestProject")
        
        result = manager.delete_file("nonexistent.txt")
        
        assert result is False

    def test_update_existing_file(self):
        """Edge case: Update existing file content"""
        manager = JCFManager(MemoryAdapter())
        manager.create_project("TestProject")
        manager.add_file("test.txt", b"Version 1")
        
        manager.add_file("test.txt", b"Version 2")
        
        content = manager.get_file("test.txt")
        assert content == b"Version 2"

    def test_add_file_without_project_raises(self):
        """Edge case: Add file before creating project raises"""
        manager = JCFManager(MemoryAdapter())
        
        with pytest.raises(ValueError):
            manager.add_file("test.txt", b"content")

    def test_save_without_project_raises(self):
        """Edge case: Save without project raises"""
        manager = JCFManager(MemoryAdapter())
        
        with pytest.raises(ValueError):
            manager.save("project.jcf")
