#!/usr/bin/env python3
"""
Example 01: Basic Workflow

Demonstrates the core lifecycle:
- create_project()
- add_file()
- save_checkpoint()
- restore_version()
"""

import os
import shutil
from kamaros import JCFManager, FileAdapter

PROJECT_STORE = "/tmp/kamaros-example-01"


def cleanup():
    if os.path.exists(PROJECT_STORE):
        shutil.rmtree(PROJECT_STORE)
    os.makedirs(PROJECT_STORE)


def main():
    cleanup()
    
    print("=" * 50)
    print("Example 01: Basic Workflow")
    print("=" * 50)

    print(f"Storage path: {PROJECT_STORE}")

    # 1. Initialize Manager
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    
    # 2. Create Project
    print("\n[1] Creating project 'DemoApp'...")
    manager.create_project("DemoApp", author="Python Example")
    
    # 3. Add initial file
    print("[2] Adding 'README.md'...")
    manager.add_file("README.md", b"# Demo Project\nInitial content.")
    
    # 4. Save Checkpoint v1
    v1_id = manager.save_checkpoint("Initial commit")
    print(f" -> Checkpoint saved: {v1_id[:12]}...")

    # 5. Modify file
    print("\n[3] Modifying 'README.md'...")
    manager.add_file("README.md", b"# Demo Project\nUpdated content with new features.")
    
    # 6. Save Checkpoint v2
    v2_id = manager.save_checkpoint("Update README")
    print(f" -> Checkpoint saved: {v2_id[:12]}...")

    # Verify current state
    current_content = manager.get_file("README.md")
    print(f"    Current content: {current_content.decode('utf-8').strip()}")
    assert b"Updated content" in current_content

    # 7. Restore v1
    print(f"\n[4] Restoring version {v1_id[:12]}...")
    restored_id = manager.restore_version(v1_id)
    print(f" -> Restored to: {restored_id[:12]}...")

    # 8. Verify restoration
    restored_content = manager.get_file("README.md")
    print(f"    Restored content: {restored_content.decode('utf-8').strip()}")
    
    if b"Initial content" in restored_content:
        print("\n    SUCCESS: Content restored correctly!")
    else:
        print("\n    FAILURE: Content mismatch!")


if __name__ == "__main__":
    main()
