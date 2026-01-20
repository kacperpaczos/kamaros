# Kamaros Python Demos

## Overview

This directory contains demonstration scripts for the Kamaros JCF library.
Each demo tests specific functionality of the API.

## Available Demos

### 1. simple_workflow.py

**Purpose**: Basic workflow test

**Tests**:
- `create_project()` - Creating new project
- `add_file()` - Adding files
- `save_checkpoint()` - Creating version checkpoint
- `restore_version()` - Restoring to previous version
- `get_file()` - Reading file content

**Run**:
```bash
python examples/python/simple_workflow.py
```

---

### 2. comprehensive_demo.py

**Purpose**: Full workflow with binary files, modifications, and history

**Tests**:
- `create_project(name, description, author)` - Project with metadata
- `add_file()` - Text and binary files (images from internet)
- `get_file()` - Reading files
- `delete_file()` - Removing files
- `list_files()` - Listing working directory
- `save_checkpoint()` - Multiple checkpoints
- `save()` - Exporting to .jcf archive
- `load()` - Loading from .jcf archive
- `get_manifest()` - Accessing version history
- `get_project_info()` - Project statistics
- `restore_version()` - Rollback to any version

**Run**:
```bash
python examples/python/comprehensive_demo.py
```

---

### 3. api_reference_demo.py

**Purpose**: Complete API reference with all supported functions

**Tests**:
- All functions from comprehensive_demo
- `get_version_info(version_id)` - Get details of specific version
- `get_file_at_version(path, version_id)` - Read file from past version
- `rename_file(old_path, new_path)` - Rename with history tracking
- `get_file_history(path)` - Get modification history of file
- `compare_versions(v1_id, v2_id)` - Diff between versions

**Run**:
```bash
python examples/python/api_reference_demo.py
```

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
