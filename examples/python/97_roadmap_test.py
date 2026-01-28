#!/usr/bin/env python3
"""
Example 97: Roadmap Features Test

Tests new roadmap features: tag_version, get_version_by_tag, verify_integrity.
"""

import os
import shutil
from kamaros import JCFManager, FileAdapter

STORE = "/tmp/kamaros-test-roadmap"


def cleanup():
    if os.path.exists(STORE):
        shutil.rmtree(STORE)
    os.makedirs(STORE)


def main():
    cleanup()
    
    print("=" * 60)
    print("Example 97: Roadmap Features Test")
    print("=" * 60)
    
    adapter = FileAdapter(STORE)
    manager = JCFManager(adapter)
    manager.create_project("RoadmapTest")
    
    # Create versions
    manager.add_file("file.txt", b"Version 1")
    v1 = manager.save_checkpoint("v1")
    print(f"[1] Created v1: {v1[:16]}...")
    
    manager.add_file("file.txt", b"Version 2")
    v2 = manager.save_checkpoint("v2")
    print(f"[2] Created v2: {v2[:16]}...")
    
    # Test tag_version
    print("\n[3] Testing tag_version()...")
    result = manager.tag_version(v1, "release-1.0")
    print(f"    Tagged v1 as 'release-1.0': {result}")
    assert result == True
    
    result = manager.tag_version(v2, "latest")
    print(f"    Tagged v2 as 'latest': {result}")
    assert result == True
    
    # Try duplicate tag
    result = manager.tag_version(v2, "release-1.0")
    print(f"    Duplicate tag 'release-1.0' on v2: {result} (expected False)")
    assert result == False
    
    # Test get_version_by_tag
    print("\n[4] Testing get_version_by_tag()...")
    found = manager.get_version_by_tag("release-1.0")
    print(f"    'release-1.0' -> {found[:16] if found else None}...")
    assert found == v1
    
    found = manager.get_version_by_tag("latest")
    print(f"    'latest' -> {found[:16] if found else None}...")
    assert found == v2
    
    found = manager.get_version_by_tag("nonexistent")
    print(f"    'nonexistent' -> {found}")
    assert found is None
    
    # Test verify_integrity
    print("\n[5] Testing verify_integrity()...")
    result = manager.verify_integrity()
    print(f"    Valid: {result['valid']}")
    print(f"    Checked: {result['checked']} blobs")
    print(f"    Errors: {len(result['errors'])}")
    if not result["valid"]:
        import json
        print(json.dumps(result["errors"], indent=2))
    assert result["valid"] == True
    assert result["checked"] >= 2  # At least 2 blob checks
    
    print("\n" + "=" * 60)
    print("SUCCESS: All roadmap features work correctly!")
    print("=" * 60)


if __name__ == "__main__":
    main()
