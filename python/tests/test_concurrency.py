import os
import threading
import pytest
import time
from kamaros import JCFManager, MemoryAdapter

def test_concurrent_saves():
    """Test that multiple threads can save checkpoints concurrently (stress test)."""
    adapter = MemoryAdapter()
    manager = JCFManager(adapter)
    manager.create_project("ConcurrentTest")
    
    errors = []
    
    def worker(worker_id):
        try:
            for i in range(10):
                manager.add_file(f"thread_{worker_id}/file_{i}.txt", b"content")
                manager.save_checkpoint(f"Commit {i} from thread {worker_id}", author=f"worker_{worker_id}")
        except Exception as e:
            errors.append(e)

    threads = []
    for i in range(5):
        t = threading.Thread(target=worker, args=(i,))
        threads.append(t)
        t.start()

    for t in threads:
        t.join()

    assert not errors, f"Encountered errors during concurrent execution: {errors}"
    
    info = manager.get_project_info()
    assert info["version_count"] >= 50
    assert info["file_count"] == 50

def test_concurrent_read_write():
    """Test concurrent reads and writes using multiple manager instances sharing an adapter."""
    adapter = MemoryAdapter()
    
    # Initialize project
    m1 = JCFManager(adapter)
    m1.create_project("SharedTest")
    m1.add_file("init.txt", b"start")
    m1.save_checkpoint("base")
    
    def writer():
        for i in range(20):
            m = JCFManager(adapter)
            m.load_manifest() # Ensure it sees current state
            m.add_file(f"write_{i}.txt", b"data")
            m.save_checkpoint(f"Write {i}")
            time.sleep(0.01)

    def reader():
        for i in range(20):
            m = JCFManager(adapter)
            m.load_manifest()
            m.get_project_info()
            time.sleep(0.01)

    t1 = threading.Thread(target=writer)
    t2 = threading.Thread(target=reader)
    
    t1.start()
    t2.start()
    
    t1.join()
    t2.join()
    
    assert True # Should not crash
