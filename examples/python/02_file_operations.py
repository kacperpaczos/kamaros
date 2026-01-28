#!/usr/bin/env python3
"""
Example 02: File Operations

Demonstrates file management functions:
- add_file()
- get_file()
- delete_file()
- list_files()
- rename_file()
"""

import os
import shutil
from kamaros import JCFManager, FileAdapter

PROJECT_STORE = "/tmp/kamaros-example-02"


def cleanup():
    if os.path.exists(PROJECT_STORE):
        shutil.rmtree(PROJECT_STORE)
    os.makedirs(PROJECT_STORE)


def main():
    cleanup()
    
    print("=" * 50)
    print("Example 02: File Operations")
    print("=" * 50)
    
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    manager.create_project("FileOpsDemo")
    
    # --- add_file ---
    print("\n[1] add_file()")
    manager.add_file("README.md", b"# Project")
    manager.add_file("src/main.py", b"print('hello')")
    manager.add_file("src/utils.py", b"def helper(): pass")
    print(f"    Added 3 files")
    
    # --- list_files ---
    print("\n[2] list_files()")
    files = manager.list_files()
    print(f"    Files: {files}")
    assert len(files) == 3
    
    # --- get_file ---
    print("\n[3] get_file()")
    content = manager.get_file("src/main.py")
    print(f"    src/main.py: {content.decode()}")
    assert b"print" in content
    
    # --- rename_file ---
    print("\n[4] rename_file()")
    success = manager.rename_file("src/utils.py", "src/helpers.py")
    print(f"    src/utils.py -> src/helpers.py: {success}")
    assert success
    assert "src/helpers.py" in manager.list_files()
    assert "src/utils.py" not in manager.list_files()
    
    # --- delete_file ---
    print("\n[5] delete_file()")
    deleted = manager.delete_file("src/helpers.py")
    print(f"    Deleted src/helpers.py: {deleted}")
    assert deleted
    assert "src/helpers.py" not in manager.list_files()
    
    # Final state
    print("\n[Result]")
    print(f"    Final files: {manager.list_files()}")
    print("\n    SUCCESS!")


if __name__ == "__main__":
    main()
