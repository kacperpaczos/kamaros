#!/usr/bin/env python3
"""
API Reference Demo - Tests ALL supported Kamaros functions

This demo exercises every public API function in JCFManager.
"""

import os
import shutil
import urllib.request
from kamaros import JCFManager, FileAdapter

PROJECT_STORE = "./demo-api-reference-store"


def download_image(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=10) as response:
        return response.read()


def cleanup():
    for path in [PROJECT_STORE, PROJECT_STORE + "-loaded"]:
        if os.path.exists(path):
            shutil.rmtree(path)
    os.makedirs(PROJECT_STORE)


def section(title: str):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")


def test_result(name: str, passed: bool):
    status = "PASS" if passed else "FAIL"
    print(f"  [{status}] {name}")


def main():
    cleanup()
    results = []
    
    section("1. create_project(name, description, author)")
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    manager.create_project(
        name="APITestProject",
        description="Testing all API functions",
        author="Test Script"
    )
    info = manager.get_project_info()
    passed = info["name"] == "APITestProject"
    test_result("create_project", passed)
    results.append(("create_project", passed))
    
    # ---
    section("2. add_file(path, content)")
    manager.add_file("README.md", b"# Test Project\nVersion 1")
    manager.add_file("src/main.py", b"print('Hello')")
    
    photo = download_image("https://picsum.photos/seed/apitest/200/200.jpg")
    manager.add_file("images/photo.jpg", photo)
    
    passed = len(manager.list_files()) == 3
    test_result("add_file", passed)
    results.append(("add_file", passed))
    
    # ---
    section("3. get_file(path)")
    content = manager.get_file("README.md")
    passed = content is not None and b"Test Project" in content
    test_result("get_file", passed)
    results.append(("get_file", passed))
    
    # ---
    section("4. list_files()")
    files = manager.list_files()
    passed = "README.md" in files and "src/main.py" in files
    test_result("list_files", passed)
    results.append(("list_files", passed))
    print(f"  Files: {files}")
    
    # ---
    section("5. save_checkpoint(message, author)")
    v1_id = manager.save_checkpoint("Initial commit", "Tester")
    passed = v1_id is not None and len(v1_id) > 10
    test_result("save_checkpoint", passed)
    results.append(("save_checkpoint", passed))
    print(f"  Version ID: {v1_id[:16]}...")
    
    # ---
    section("6. get_manifest()")
    manifest = manager.get_manifest()
    passed = manifest is not None and "versionHistory" in manifest
    test_result("get_manifest", passed)
    results.append(("get_manifest", passed))
    print(f"  Format version: {manifest.get('formatVersion')}")
    
    # ---
    section("7. get_project_info()")
    info = manager.get_project_info()
    passed = info["version_count"] == 1 and info["file_count"] == 3
    test_result("get_project_info", passed)
    results.append(("get_project_info", passed))
    print(f"  Info: {info}")
    
    # Make more changes for testing
    manager.add_file("README.md", b"# Test Project\nVersion 2 - updated")
    manager.add_file("docs/guide.md", b"# User Guide")
    v2_id = manager.save_checkpoint("Updated README, added docs")
    
    manager.delete_file("src/main.py")
    v3_id = manager.save_checkpoint("Removed main.py")
    
    # ---
    section("8. get_version_info(version_id)")
    v1_info = manager.get_version_info(v1_id)
    passed = v1_info is not None and v1_info["message"] == "Initial commit"
    test_result("get_version_info", passed)
    results.append(("get_version_info", passed))
    print(f"  V1 info: {v1_info['message']}, files: {v1_info['file_count']}")
    
    # ---
    section("9. get_file_at_version(path, version_id)")
    old_readme = manager.get_file_at_version("README.md", v1_id)
    passed = old_readme is not None and b"Version 1" in old_readme
    test_result("get_file_at_version", passed)
    results.append(("get_file_at_version", passed))
    if old_readme:
        print(f"  README at v1: {old_readme.decode()[:50]}...")
    else:
        # Debug: print file_states to understand structure
        v1_info = manager.get_version_info(v1_id)
        print(f"  DEBUG: file_states keys: {list(v1_info.get('file_states', {}).keys())[:3]}")
        if "README.md" in v1_info.get("file_states", {}):
            fs = v1_info["file_states"]["README.md"]
            print(f"  DEBUG: README.md state: {fs}")
    
    # ---
    section("10. rename_file(old_path, new_path)")
    manager.add_file("old_name.txt", b"Rename test")
    success = manager.rename_file("old_name.txt", "new_name.txt")
    files_after = manager.list_files()
    passed = success and "new_name.txt" in files_after and "old_name.txt" not in files_after
    test_result("rename_file", passed)
    results.append(("rename_file", passed))
    print(f"  Renamed: old_name.txt -> new_name.txt")
    
    # ---
    section("11. get_file_history(path)")
    history = manager.get_file_history("README.md")
    passed = len(history) >= 2  # created + modified
    test_result("get_file_history", passed)
    results.append(("get_file_history", passed))
    for h in history:
        print(f"  - {h['action']}: {h['message'][:30]}...")
    
    # ---
    section("12. compare_versions(v1_id, v2_id)")
    diff = manager.compare_versions(v1_id, v2_id)
    passed = "added" in diff and "docs/guide.md" in diff["added"]
    test_result("compare_versions", passed)
    results.append(("compare_versions", passed))
    print(f"  Diff v1->v2: {diff['summary']}")
    
    # ---
    section("13. delete_file(path)")
    manager.add_file("to_delete.txt", b"Delete me")
    deleted = manager.delete_file("to_delete.txt")
    passed = deleted and manager.get_file("to_delete.txt") is None
    test_result("delete_file", passed)
    results.append(("delete_file", passed))
    
    # ---
    section("14. save(path) - export to .jcf")
    manager.save("project.jcf")
    archive_path = os.path.join(PROJECT_STORE, "project.jcf")
    passed = os.path.exists(archive_path)
    test_result("save", passed)
    results.append(("save", passed))
    print(f"  Archive size: {os.path.getsize(archive_path):,} bytes")
    
    # ---
    section("15. load(path) - import from .jcf")
    loaded_store = PROJECT_STORE + "-loaded"
    os.makedirs(loaded_store, exist_ok=True)
    shutil.copy(archive_path, os.path.join(loaded_store, "project.jcf"))
    shutil.copytree(os.path.join(PROJECT_STORE, ".store"), os.path.join(loaded_store, ".store"))
    
    adapter2 = FileAdapter(loaded_store)
    manager2 = JCFManager(adapter2)
    manager2.load("project.jcf")
    info2 = manager2.get_project_info()
    passed = info2["name"] == "APITestProject"
    test_result("load", passed)
    results.append(("load", passed))
    print(f"  Loaded project: {info2['name']}")
    
    # ---
    section("16. restore_version(version_id)")
    manager2.restore_version(v1_id)
    files_restored = manager2.list_files()
    passed = "src/main.py" in files_restored  # Should be back
    test_result("restore_version", passed)
    results.append(("restore_version", passed))
    print(f"  Files after restore to v1: {files_restored}")
    
    # =========================================================================
    section("SUMMARY")
    
    passed_count = sum(1 for _, p in results if p)
    total = len(results)
    
    print(f"\n  Total: {passed_count}/{total} tests passed")
    print()
    
    for name, passed in results:
        status = "OK" if passed else "FAILED"
        print(f"    {status:6} | {name}")
    
    if passed_count == total:
        print("\n  ALL TESTS PASSED!")
    else:
        print(f"\n  {total - passed_count} test(s) failed.")


if __name__ == "__main__":
    main()
