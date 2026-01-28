#!/usr/bin/env python3
"""
Example 03: Version History

Demonstrates version browsing functions:
- save_checkpoint()
- get_version_info()
- get_file_history()
- compare_versions()
"""

import os
import shutil
from kamaros import JCFManager, FileAdapter

PROJECT_STORE = "/tmp/kamaros-example-03"


def cleanup():
    if os.path.exists(PROJECT_STORE):
        shutil.rmtree(PROJECT_STORE)
    os.makedirs(PROJECT_STORE)


def main():
    cleanup()
    
    print("=" * 50)
    print("Example 03: Version History")
    print("=" * 50)
    
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    manager.create_project("HistoryDemo")
    
    # Create version 1
    manager.add_file("config.json", b'{"version": 1}')
    manager.add_file("data.txt", b"Initial data")
    v1_id = manager.save_checkpoint("Initial setup", "Author1")
    print(f"\n[1] Created v1: {v1_id[:12]}...")
    
    # Create version 2
    manager.add_file("config.json", b'{"version": 2, "debug": true}')
    manager.add_file("new_file.txt", b"New content")
    v2_id = manager.save_checkpoint("Added debug mode", "Author2")
    print(f"[2] Created v2: {v2_id[:12]}...")
    
    # Create version 3
    manager.add_file("data.txt", b"Updated data v3")
    manager.delete_file("new_file.txt")
    v3_id = manager.save_checkpoint("Updated data, removed new_file", "Author1")
    print(f"[3] Created v3: {v3_id[:12]}...")
    
    # --- get_version_info ---
    print("\n[4] get_version_info(v2)")
    v2_info = manager.get_version_info(v2_id)
    print(f"    Message: {v2_info['message']}")
    print(f"    Author: {v2_info['author']}")
    print(f"    File count: {v2_info['file_count']}")
    
    # --- get_file_history ---
    print("\n[5] get_file_history('config.json')")
    history = manager.get_file_history("config.json")
    for h in history:
        print(f"    - {h['action']}: {h['message'][:30]}")
    
    # --- compare_versions ---
    print("\n[6] compare_versions(v1, v2)")
    diff = manager.compare_versions(v1_id, v2_id)
    print(f"    Added: {diff['added']}")
    print(f"    Modified: {diff['modified']}")
    print(f"    Summary: {diff['summary']}")
    
    print("\n[7] compare_versions(v2, v3)")
    diff2 = manager.compare_versions(v2_id, v3_id)
    print(f"    Removed: {diff2['removed']}")
    print(f"    Modified: {diff2['modified']}")
    print(f"    Summary: {diff2['summary']}")
    
    print("\n    SUCCESS!")


if __name__ == "__main__":
    main()
