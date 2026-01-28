#!/usr/bin/env python3
"""
Example 98: Portable Archive Test

Verifies that archives created by save() are PORTABLE, 
meaning they include blobs and can be restored on a fresh system
without manually copying the .store directory.
"""

import os
import shutil
from kamaros import JCFManager, FileAdapter

STORE_ORIGIN = "/tmp/kamaros-test-portable-origin"
STORE_DEST = "/tmp/kamaros-test-portable-dest"
ARCHIVE_NAME = "portable.jcf"


def cleanup():
    for path in [STORE_ORIGIN, STORE_DEST]:
        if os.path.exists(path):
            shutil.rmtree(path)
    os.makedirs(STORE_ORIGIN)
    os.makedirs(STORE_DEST)


def main():
    cleanup()
    
    print("=" * 60)
    print("Example 98: Portable Archive Test")
    print("=" * 60)
    
    # 1. Create origin project
    print("[1] Creating origin project...")
    adapter1 = FileAdapter(STORE_ORIGIN)
    manager1 = JCFManager(adapter1)
    manager1.create_project("PortableProject")
    
    manager1.add_file("data.txt", b"Version 1 content")
    v1_id = manager1.save_checkpoint("v1")
    print(f" -> Saved v1: {v1_id}")
    
    manager1.add_file("data.txt", b"Version 2 content")
    v2_id = manager1.save_checkpoint("v2")
    print(f" -> Saved v2: {v2_id}")
    
    # 2. Export to archive
    print("\n[2] Exporting to archive (should include blobs)...")
    manager1.save(ARCHIVE_NAME)
    archive_path = os.path.join(STORE_ORIGIN, ARCHIVE_NAME)
    assert os.path.exists(archive_path)
    print(f" -> Archive created: {archive_path}")
    
    # 3. Simulate transfer to new machine
    print("\n[3] Transferring archive to clean location...")
    dest_archive_path = os.path.join(STORE_DEST, ARCHIVE_NAME)
    shutil.copy(archive_path, dest_archive_path)
    print(f" -> Copied to {dest_archive_path}")
    
    # Note: We do NOT copy .store folder manually!
    
    # 4. Load in new location
    print("\n[4] Loading in new location...")
    adapter2 = FileAdapter(STORE_DEST)
    manager2 = JCFManager(adapter2)
    
    manager2.load(ARCHIVE_NAME)
    info = manager2.get_project_info()
    print(f" -> Loaded project: {info['name']}")
    
    # 5. Verify blob extraction through list
    # FileAdapter writes to disk, so we can check if .store exists
    store_path = os.path.join(STORE_DEST, ".store")
    has_store = os.path.exists(store_path)
    print(f" -> .store directory present: {has_store}")
    assert has_store, "Architecture flaw: .store directory was not extracted from archive!"

    # 6. Attempt Restore (requires blobs)
    print(f"\n[5] Attempting restore of v1 ({v1_id})...")
    try:
        manager2.restore_version(v1_id)
        content = manager2.get_file("data.txt")
        print(f" -> Content: {content.decode()}")
        
        if content == b"Version 1 content":
            print("\nSUCCESS: Archive is fully portable!")
        else:
            print("\nFAILURE: Content mismatch!")
    except Exception as e:
        print(f"\nFAILURE: Restore failed: {e}")
        # Debug info
        if os.path.exists(store_path):
             print(f"DEBUG: .store content: {os.listdir(os.path.join(store_path, 'blobs'))}")


if __name__ == "__main__":
    main()
