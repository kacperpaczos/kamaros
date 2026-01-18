"""
Tests for Storage Adapters

Each test focuses on a single aspect of adapter functionality.
"""

import pytest
from kamaros.adapters import MemoryAdapter, FileAdapter
import tempfile
import os


class TestMemoryAdapterHappyPath:
    """Happy path tests for MemoryAdapter"""

    def test_write_and_read(self):
        """Test: Can write and read data"""
        adapter = MemoryAdapter()
        
        adapter.write("test.bin", b"\x01\x02\x03\x04\x05")
        data = adapter.read("test.bin")
        
        assert data == b"\x01\x02\x03\x04\x05"

    def test_exists(self):
        """Test: Can check file existence"""
        adapter = MemoryAdapter()
        adapter.write("exists.txt", b"content")
        
        assert adapter.exists("exists.txt") is True
        assert adapter.exists("notexists.txt") is False

    def test_delete(self):
        """Test: Can delete file"""
        adapter = MemoryAdapter()
        adapter.write("test.txt", b"content")
        
        adapter.delete("test.txt")
        
        assert adapter.exists("test.txt") is False

    def test_list(self):
        """Test: Can list files in directory"""
        adapter = MemoryAdapter()
        adapter.write("dir/file1.txt", b"1")
        adapter.write("dir/file2.txt", b"2")
        adapter.write("other/file3.txt", b"3")
        
        files = adapter.list("dir")
        
        assert len(files) == 2
        assert "file1.txt" in files
        assert "file2.txt" in files

    def test_clear(self):
        """Test: Clear removes all data"""
        adapter = MemoryAdapter()
        adapter.write("file1.txt", b"1")
        adapter.write("file2.txt", b"2")
        
        adapter.clear()
        
        assert adapter.exists("file1.txt") is False
        assert adapter.exists("file2.txt") is False


class TestMemoryAdapterEdgeCases:
    """Edge case tests for MemoryAdapter"""

    def test_read_nonexistent_raises(self):
        """Edge case: Read non-existent file raises"""
        adapter = MemoryAdapter()
        
        with pytest.raises(FileNotFoundError):
            adapter.read("nonexistent.txt")

    def test_delete_nonexistent_no_error(self):
        """Edge case: Delete non-existent file doesn't raise"""
        adapter = MemoryAdapter()
        
        # Should not raise
        adapter.delete("nonexistent.txt")

    def test_empty_file(self):
        """Edge case: Handle empty file"""
        adapter = MemoryAdapter()
        
        adapter.write("empty.txt", b"")
        data = adapter.read("empty.txt")
        
        assert data == b""
        assert len(data) == 0

    def test_overwrite_file(self):
        """Edge case: Overwrite existing file"""
        adapter = MemoryAdapter()
        adapter.write("test.txt", b"Version 1")
        
        adapter.write("test.txt", b"Version 2")
        data = adapter.read("test.txt")
        
        assert data == b"Version 2"

    def test_nested_paths(self):
        """Edge case: Handle nested directory paths"""
        adapter = MemoryAdapter()
        
        adapter.write("a/b/c/deep.txt", b"content")
        
        assert adapter.exists("a/b/c/deep.txt") is True

    def test_list_empty_directory(self):
        """Edge case: List non-existent directory returns empty list"""
        adapter = MemoryAdapter()
        
        files = adapter.list("nonexistent")
        
        assert files == []


class TestFileAdapter:
    """Tests for FileAdapter (with real filesystem)"""

    def test_write_and_read(self):
        """Test: Can write and read file"""
        with tempfile.TemporaryDirectory() as tmpdir:
            adapter = FileAdapter(tmpdir)
            
            adapter.write("test.txt", b"Hello World")
            data = adapter.read("test.txt")
            
            assert data == b"Hello World"

    def test_exists(self):
        """Test: Can check file existence"""
        with tempfile.TemporaryDirectory() as tmpdir:
            adapter = FileAdapter(tmpdir)
            adapter.write("exists.txt", b"content")
            
            assert adapter.exists("exists.txt") is True
            assert adapter.exists("notexists.txt") is False

    def test_delete(self):
        """Test: Can delete file"""
        with tempfile.TemporaryDirectory() as tmpdir:
            adapter = FileAdapter(tmpdir)
            adapter.write("test.txt", b"content")
            
            adapter.delete("test.txt")
            
            assert adapter.exists("test.txt") is False

    def test_list(self):
        """Test: Can list files"""
        with tempfile.TemporaryDirectory() as tmpdir:
            adapter = FileAdapter(tmpdir)
            adapter.write("file1.txt", b"1")
            adapter.write("file2.txt", b"2")
            
            files = adapter.list("")
            
            assert len(files) >= 2

    def test_nested_directories(self):
        """Test: Creates nested directories automatically"""
        with tempfile.TemporaryDirectory() as tmpdir:
            adapter = FileAdapter(tmpdir)
            
            adapter.write("a/b/c/deep.txt", b"content")
            
            assert adapter.exists("a/b/c/deep.txt") is True
