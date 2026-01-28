# Kamaros Python Demos

## Overview

This directory contains demonstration scripts for the Kamaros JCF library.
Each demo tests specific functionality of the API.

## Available Demos

All examples are located in `examples/python/`.

### `01_basic_workflow.py`
**Basic Lifecycle**: Creates project, adds file, saves checkpoint, restores version.
Run: `python examples/python/01_basic_workflow.py`

### `02_file_operations.py`
**File Management**: Demonstrates add, get, list, rename, and delete file operations.
Run: `python examples/python/02_file_operations.py`

### `03_version_history.py`
**History Browsing**: Inspects version details, file history, and diffs between versions.
Run: `python examples/python/03_version_history.py`

### `04_save_load_archive.py`
**Import/Export**: Saves project to `.jcf` archive and loads it into a new manager.
Run: `python examples/python/04_save_load_archive.py`

### `05_comprehensive_demo.py`
**Full Workflow**: A complete walkthrough with images, metadata, and rollback.
Run: `python examples/python/05_comprehensive_demo.py`

### `99_api_test.py`
**API Reference**: Systematically tests all 16 public API functions.
Run: `python examples/python/99_api_test.py`

---

## API Function Reference

| Function | Status | Description |
|----------|--------|-------------|
| `create_project(name, desc?, author?)` | ✅ | Create new project |
| `load(path)` | ✅ | Load from .jcf archive |
| `save(path)` | ✅ | Save to .jcf archive |
| `add_file(path, content)` | ✅ | Add/update file |
| `get_file(path)` | ✅ | Get file content |
| `delete_file(path)` | ✅ | Delete file |
| `list_files()` | ✅ | List all files |
| `save_checkpoint(message, author?)` | ✅ | Create version |
| `restore_version(version_id)` | ✅ | Restore to version |
| `get_manifest()` | ✅ | Get full manifest |
| `get_project_info()` | ✅ | Get project summary |
| `get_version_info(version_id)` | ✅ | Get version details |
| `get_file_at_version(path, version_id)` | ✅ | Read historical file |
| `rename_file(old, new)` | ✅ | Rename with tracking |
| `get_file_history(path)` | ✅ | File change history |
| `compare_versions(v1, v2)` | ✅ | Diff between versions |

All 16 API functions are implemented and tested.

---

## Roadmap - Planned Functions

### Version Management
| Function | Priority | Description |
|----------|----------|-------------|
| `tag_version(version_id, tag)` | HIGH | Tag versions (e.g. "v1.0", "release") |
| `get_versions_by_tag(tag)` | HIGH | Find versions by tag |
| `get_head()` | MEDIUM | Get current HEAD version |
| `squash_versions(v1, v2)` | LOW | Merge multiple versions into one |

### File Operations
| Function | Priority | Description |
|----------|----------|-------------|
| `copy_file(src, dst)` | MEDIUM | Copy file with metadata |
| `search_files(pattern)` | MEDIUM | Search files by glob pattern |
| `get_file_size(path)` | LOW | Get file size |

### Export/Import
| Function | Priority | Description |
|----------|----------|-------------|
| `export_portable(path)` | HIGH | Export with blobs (fully portable archive) |
| `export_version(version_id, dir)` | MEDIUM | Export specific version to directory |
| `import_directory(dir)` | MEDIUM | Import directory as new project |

### Maintenance
| Function | Priority | Description |
|----------|----------|-------------|
| `gc()` | MEDIUM | Remove unreferenced blobs |
| `verify_integrity()` | HIGH | Check blob hash integrity |
| `get_storage_stats()` | LOW | Storage statistics |
