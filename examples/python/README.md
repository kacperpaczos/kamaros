# Python Examples

Complete API demonstration for the Kamaros Python library.

## Quick Start

```bash
# From project root
source .venv/bin/activate
cd python && maturin develop && cd ..
python examples/python/01_basic_workflow.py
```

## Examples

| # | File | Functions Tested | Description |
|---|------|------------------|-------------|
| 01 | `01_basic_workflow.py` | create_project, add_file, save_checkpoint, restore_version | Basic versioning workflow |
| 02 | `02_file_operations.py` | add_file, get_file, delete_file, list_files, rename_file | File management |
| 03 | `03_version_history.py` | get_version_info, get_file_history, compare_versions | History browsing |
| 04 | `04_save_load_archive.py` | save, load, get_file_at_version | Archive import/export |
| 05 | `05_comprehensive_demo.py` | ALL 16 functions | Full integration test |

## API Reference

All 16 implemented functions:

### Project Management
- `create_project(name, description?, author?)` - Create new project
- `load(path)` - Load from .jcf archive
- `save(path)` - Save to .jcf archive
- `get_manifest()` - Get raw manifest
- `get_project_info()` - Get project summary

### File Operations
- `add_file(path, content)` - Add/update file
- `get_file(path)` - Read file content
- `delete_file(path)` - Delete file
- `list_files()` - List all files
- `rename_file(old, new)` - Rename with history tracking

### Version Control
- `save_checkpoint(message, author?)` - Create new version
- `restore_version(version_id)` - Restore to version
- `get_version_info(version_id)` - Get version details
- `get_file_at_version(path, version_id)` - Read historical file
- `get_file_history(path)` - File modification history
- `compare_versions(v1, v2)` - Diff between versions

## Roadmap

See [demos.md](demos.md) for planned functions.
