
import os
import shutil
from kamaros import JCFManager, FileAdapter

def main():
    # Setup paths
    project_path = "./demo-project-store"
    if os.path.exists(project_path):
        shutil.rmtree(project_path)
    os.makedirs(project_path)

    print(f"--- Kamaros Python Demo ---")
    print(f"Storage path: {project_path}")

    # 1. Initialize Manager
    adapter = FileAdapter(project_path)
    manager = JCFManager(adapter)
    
    # 2. Create Project
    print("\n[1] Creating project 'DemoApp'...")
    manager.create_project("DemoApp", author="Python Example")
    
    # 3. Add initial file
    print("[2] Adding 'README.md'...")
    manager.add_file("README.md", b"# Demo Project\nInitial content.")
    
    # 4. Save Checkpoint v1
    v1_id = manager.save_checkpoint("Initial commit")
    print(f" -> Checkpoint saved: {v1_id}")

    # 5. Modify file
    print("\n[3] Modifying 'README.md'...")
    manager.add_file("README.md", b"# Demo Project\nUpdated content with new features.")
    
    # 6. Save Checkpoint v2
    v2_id = manager.save_checkpoint("Update README")
    print(f" -> Checkpoint saved: {v2_id}")

    # Verify current state
    current_content = manager.get_file("README.md")
    print(f"Current content: {current_content.decode('utf-8').strip()}")
    assert b"Updated content" in current_content

    # 7. Restore v1
    print(f"\n[4] Restoring version {v1_id}...")
    restored_id = manager.restore_version(v1_id)
    print(f" -> Restored to: {restored_id}")

    # 8. Verify restoration
    restored_content = manager.get_file("README.md")
    print(f"Restored content: {restored_content.decode('utf-8').strip()}")
    
    if b"Initial content" in restored_content:
        print("\nSUCCESS: Content restored correctly!")
    else:
        print("\nFAILURE: Content mismatch!")

if __name__ == "__main__":
    main()
