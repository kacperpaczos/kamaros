#!/usr/bin/env python3
"""
Example 04: Save & Load Archive

Demonstrates archive operations:
- save() - Export to .jcf archive
- load() - Import from .jcf archive
- get_file_at_version() - Read historical file
"""

import os
import shutil
from kamaros import JCFManager, FileAdapter

PROJECT_STORE = "/tmp/kamaros-example-04"
PROJECT_STORE_2 = "/tmp/kamaros-example-04-loaded"


def cleanup():
    for path in [PROJECT_STORE, PROJECT_STORE_2]:
        if os.path.exists(path):
            shutil.rmtree(path)
        os.makedirs(path)


def main():
    cleanup()
    
    print("=" * 50)
    print("Example 04: Save & Load Archive")
    print("=" * 50)
    
    # Create project and versions
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    manager.create_project("ArchiveDemo", description="Archive test", author="Demo")
    
    manager.add_file("index.html", b"<html>v1</html>")
    v1_id = manager.save_checkpoint("Version 1")
    
    manager.add_file("index.html", b"<html>v2 - updated</html>")
    manager.add_file("style.css", b"body { color: blue; }")
    v2_id = manager.save_checkpoint("Version 2")
    
    print(f"\n[1] Created project with 2 versions")
    print(f"    v1: {v1_id[:12]}...")
    print(f"    v2: {v2_id[:12]}...")
    
    # --- save() ---
    print("\n[2] save('project.jcf')")
    manager.save("project.jcf")
    archive_path = os.path.join(PROJECT_STORE, "project.jcf")
    size = os.path.getsize(archive_path)
    print(f"    Saved: {archive_path}")
    print(f"    Size: {size} bytes")
    
    # --- load() in new manager ---
    print("\n[3] load() in new manager")
    
    # Copy archive and blobs to new location
    shutil.copy(archive_path, os.path.join(PROJECT_STORE_2, "project.jcf"))
    shutil.copytree(
        os.path.join(PROJECT_STORE, ".store"),
        os.path.join(PROJECT_STORE_2, ".store")
    )
    
    adapter2 = FileAdapter(PROJECT_STORE_2)
    manager2 = JCFManager(adapter2)
    manager2.load("project.jcf")
    
    info = manager2.get_project_info()
    print(f"    Loaded: {info['name']}")
    print(f"    Versions: {info['version_count']}")
    print(f"    Files: {info['file_count']}")
    
    # --- get_file_at_version() ---
    print("\n[4] get_file_at_version('index.html', v1)")
    old_content = manager2.get_file_at_version("index.html", v1_id)
    print(f"    Content at v1: {old_content.decode()}")
    assert b"v1" in old_content
    
    current_content = manager2.get_file("index.html")
    print(f"    Current content: {current_content.decode()}")
    assert b"v2" in current_content
    
    print("\n    SUCCESS!")


if __name__ == "__main__":
    main()
