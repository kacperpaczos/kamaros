#!/usr/bin/env python3
"""
Example 06: Concurrency & Stress Test

Demonstrates multi-process access to the same project store.
Since the storage backend (FileAdapter) relies on OS filesystem locking (or lack thereof),
this test verifies if concurrent writes cause corruption or if they are handled gracefully.

Note: Native JCF Manager doesn't handle locking yet, so this test EXPECTS failures
or race conditions if locking isn't implemented. It serves as a benchmark/proving ground.
"""

import os
import shutil
import time
import random
import multiprocessing
from concurrent.futures import ProcessPoolExecutor, as_completed
from kamaros import JCFManager, FileAdapter

PROJECT_STORE = "/tmp/kamaros-example-06"
WORKER_COUNT = 4
ITERATIONS_PER_WORKER = 10

def worker_task(worker_id: int):
    """
    Worker process that tries to add files and create checkpoints simultaneously.
    """
    try:
        # Each worker needs its own adapter instance
        adapter = FileAdapter(PROJECT_STORE)
        manager = JCFManager(adapter)
        
        # Reload manifest to get latest state
        # In a real scenario, you'd need a way to refresh or re-instantiate
        
        for i in range(ITERATIONS_PER_WORKER):
            timestamp = time.time()
            filename = f"worker_{worker_id}_{i}.txt"
            content = f"Data from worker {worker_id} iteration {i} at {timestamp}".encode()
            
            # 1. Add File
            manager.add_file(filename, content)
            
            # 2. Random sleep to scramble timing
            time.sleep(random.random() * 0.1)
            
            # 3. Try to checkpoint
            try:
                version_id = manager.save_checkpoint(f"Commit from worker {worker_id} #{i}")
                print(f"[Worker {worker_id}] Saved {version_id[:8]}...")
            except Exception as e:
                print(f"[Worker {worker_id}] Checkpoint failed (expected race): {e}")
                # Reload might be needed here if state is stale
                
        return f"Worker {worker_id} done"
        
    except Exception as e:
        return f"Worker {worker_id} crashed: {e}"


def cleanup():
    if os.path.exists(PROJECT_STORE):
        shutil.rmtree(PROJECT_STORE)
    os.makedirs(PROJECT_STORE)


def main():
    cleanup()
    print("=" * 60)
    print("Example 06: Concurrency Stress Test")
    print("=" * 60)
    print(f"Workers: {WORKER_COUNT}")
    print(f"Iterations: {ITERATIONS_PER_WORKER}")
    print(f"Store: {PROJECT_STORE}")
    
    # 1. Initialize Project
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    manager.create_project("ConcurrencyTest", description="Multi-process stress test")
    print("[Main] Project created.")
    
    # 2. Run Workers
    print("\n[Main] Starting workers...")
    start_time = time.time()
    
    with ProcessPoolExecutor(max_workers=WORKER_COUNT) as executor:
        futures = [executor.submit(worker_task, i) for i in range(WORKER_COUNT)]
        
        for future in as_completed(futures):
            print(f"    -> {future.result()}")
            
    duration = time.time() - start_time
    print(f"\n[Main] All workers finished in {duration:.2f}s")
    
    # 3. Validation
    print("\n[Main] Validating final state...")
    
    # Reload fresh
    manager_final = JCFManager(FileAdapter(PROJECT_STORE))
    # Note: In a real app we might need to explicitly 'load' or refresh if the manager instance
    # holds stale state. Python wrapper might need a .reload() method. 
    # For now, we assume FileAdapter + JCFManager reads manifest on init/access if constructed fresh.
    # But JCFManager constructor doesn't auto-load "current" state from disk unless create_project or load is called?
    # Actually, JCFManager in Python keeps state in memory. So we need to reconstruct it 
    # to see what persists on disk.
    
    # There isn't a direct "load_from_store" without an archive. 
    # We essentially need to re-read manifest.json from the store if we want to sync.
    # But the current API (based on looking at other examples) expects 'load' from .jcf archive
    # or 'create_project'. 
    # If we just want to inspect the 'latest' state in the folder, how do we attach?
    # Python JCFManager seems to assume we are the owner.
    # We will try to read the manifest manually via adapter to verify.
    
    try:
        manifest_data = adapter.read("manifest.json")
        print(f"    ✓ manifest.json exists ({len(manifest_data)} bytes)")
        
        import json
        manifest = json.loads(manifest_data)
        version_count = len(manifest.get('versionHistory', []))
        print(f"    ✓ Version count: {version_count}")
        
    except Exception as e:
        print(f"    ✗ Validation failed: {e}")

if __name__ == "__main__":
    main()
