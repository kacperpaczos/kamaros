# Kamaros Python Library

Python library for managing JCF (JSON Content Format) files - intelligent ZIP with Time-Travel versioning.

## Installation

```bash
pip install kamaros
```

## Quick Start

```python
from kamaros import JCFManager, MemoryAdapter

# Create manager with in-memory storage
manager = JCFManager(MemoryAdapter())

# Create new project
manager.create_project("MyProject", description="Demo project")

# Add files
manager.add_file("main.py", b"print('Hello')")
manager.add_file("README.md", b"# My Project")

# Save to JCF file
manager.save("project.jcf")

# Load existing project
manager2 = JCFManager(MemoryAdapter())
manager2.load("project.jcf")
print(manager2.get_project_info())
```

## Features

- **JCF Format**: Standard ZIP with versioning
- **Time-Travel**: Access any version of your files
- **Content Addressable Storage**: Automatic binary deduplication
- **Cross-Platform**: Works with Rust core via PyO3

## API

### JCFManager

- `create_project(name, description?, author?)` - Create new project
- `load(path)` - Load JCF file
- `save(path)` - Save JCF file
- `add_file(path, content)` - Add file
- `get_file(path)` - Get file content
- `delete_file(path)` - Delete file
- `list_files()` - List all files
- `get_manifest()` - Get manifest dict
- `get_project_info()` - Get project info

### Adapters

- `MemoryAdapter()` - In-memory storage (testing)
- `FileAdapter(base_path)` - File system storage

## License

MIT
