# JCF Format Usage Examples

## Example 1: Simple Text File Project

### Scenario
A project with only text files (e.g., documentation, scripts).

### Structure
```
project.jcf
├── mimetype
├── manifest.json
├── content/
│   ├── README.md
│   ├── script.js
│   └── config.json
└── .store/
    └── deltas/
        ├── v1_README.md.patch
        ├── v1_script.js.patch
        └── v1_config.json.patch
```

### Manifest Excerpt
```json
{
  "fileMap": {
    "README.md": {
      "type": "text",
      "inodeId": "...",
      "currentHash": "...",
      "encoding": "utf-8"
    },
    "script.js": {
      "type": "text",
      "inodeId": "...",
      "currentHash": "...",
      "encoding": "utf-8"
    }
  }
}
```

### Notes
- All files use reverse delta strategy
- No `.store/blobs/` directory needed
- Fast versioning for text changes

---

## Example 2: Binary Assets Project

### Scenario
A project with images, videos, and other binary files.

### Structure
```
project.jcf
├── mimetype
├── manifest.json
├── content/
│   ├── assets/
│   │   ├── hero.png
│   │   ├── background.jpg
│   │   └── video.mp4
│   └── metadata.json
└── .store/
    ├── blobs/
    │   ├── a3f5e8b2c1d4e6f7... (hero.png)
    │   ├── 9d4c1e7f3a8b2c5d6... (background.jpg)
    │   └── 2b7c9e1f4a6d8b3c5... (video.mp4)
    └── deltas/
        └── v1_metadata.json.patch
```

### Manifest Excerpt
```json
{
  "fileMap": {
    "assets/hero.png": {
      "type": "binary",
      "inodeId": "...",
      "currentHash": "a3f5e8b2c1d4e6f7...",
      "size": 245760,
      "mime": "image/png"
    }
  }
}
```

### Notes
- Binary files stored in CAS (Content Addressable Storage)
- Same file in multiple versions → same blob reused
- Efficient deduplication

---

## Example 3: Mixed Content Project

### Scenario
A typical project with both text and binary files.

### Structure
```
project.jcf
├── mimetype
├── manifest.json
├── content/
│   ├── src/
│   │   ├── main.js
│   │   └── utils.js
│   ├── assets/
│   │   ├── logo.png
│   │   └── icon.svg
│   └── package.json
└── .store/
    ├── blobs/
    │   └── a3f5e8b2c1d4e6f7... (logo.png)
    └── deltas/
        ├── v3_src_main.js.patch
        ├── v2_src_utils.js.patch
        └── v1_package.json.patch
```

### Version History Example
```json
{
  "versionHistory": [
    {
      "id": "v1",
      "message": "Initial commit",
      "fileStates": {
        "src/main.js": { "changeType": "added" },
        "assets/logo.png": { "changeType": "added" }
      }
    },
    {
      "id": "v2",
      "message": "Add utils",
      "parentId": "v1",
      "fileStates": {
        "src/main.js": {},
        "src/utils.js": { "changeType": "added" },
        "assets/logo.png": {}
      }
    },
    {
      "id": "v3",
      "message": "Update main",
      "parentId": "v2",
      "fileStates": {
        "src/main.js": { "changeType": "modified" },
        "src/utils.js": {},
        "assets/logo.png": {}
      }
    }
  ]
}
```

---

## Example 4: File Rename Tracking

### Scenario
A file is renamed across versions.

### Manifest Excerpt
```json
{
  "renameLog": [
    {
      "inodeId": "550e8400-e29b-41d4-a716-446655440000",
      "fromPath": "old-name.js",
      "toPath": "new-name.js",
      "versionId": "v2-rename",
      "timestamp": "2025-01-18T12:00:00Z"
    }
  ],
  "versionHistory": [
    {
      "id": "v1",
      "fileStates": {
        "old-name.js": {
          "inodeId": "550e8400-e29b-41d4-a716-446655440000",
          "path": "old-name.js"
        }
      }
    },
    {
      "id": "v2-rename",
      "fileStates": {
        "new-name.js": {
          "inodeId": "550e8400-e29b-41d4-a716-446655440000",
          "path": "new-name.js",
          "changeType": "renamed"
        }
      }
    }
  ]
}
```

### Notes
- Same `inodeId` tracks file across rename
- `renameLog` records the rename operation
- History queries can follow file across renames

---

## Example 5: Large File with Streaming

### Scenario
A project with very large files (>500MB).

### Structure
```
project.jcf
├── mimetype
├── manifest.json
├── content/
│   └── large-video.mp4 (500MB)
└── .store/
    └── blobs/
        └── 7f9a2c4e6b8d1f3a5... (large-video.mp4)
```

### Manifest Excerpt
```json
{
  "fileMap": {
    "large-video.mp4": {
      "type": "binary",
      "inodeId": "...",
      "currentHash": "7f9a2c4e6b8d1f3a5...",
      "size": 524288000,
      "mime": "video/mp4"
    }
  }
}
```

### Notes
- Large files stored as-is (no compression for already compressed formats)
- Streaming API used for read/write operations
- No memory issues - processed in chunks

---

## Example 6: Multiple Versions with Deduplication

### Scenario
Same binary file appears in multiple versions.

### Structure
```
project.jcf
├── content/
│   └── assets/logo.png (current)
└── .store/
    └── blobs/
        └── a3f5e8b2c1d4e6f7... (stored once, referenced multiple times)
```

### Version History
```json
{
  "versionHistory": [
    {
      "id": "v1",
      "fileStates": {
        "assets/logo.png": {
          "hash": "a3f5e8b2c1d4e6f7...",
          "contentRef": ".store/blobs/a3f5e8b2c1d4e6f7..."
        }
      }
    },
    {
      "id": "v2",
      "fileStates": {
        "assets/logo.png": {
          "hash": "a3f5e8b2c1d4e6f7...",
          "contentRef": ".store/blobs/a3f5e8b2c1d4e6f7..."
        }
      }
    },
    {
      "id": "v3",
      "fileStates": {
        "assets/logo.png": {
          "hash": "a3f5e8b2c1d4e6f7...",
          "contentRef": ".store/blobs/a3f5e8b2c1d4e6f7..."
        }
      }
    }
  ]
}
```

### Notes
- Same hash = same blob file
- Only one copy stored in `.store/blobs/`
- All versions reference the same blob
- Efficient storage for unchanged files

---

## Example 7: Deleted File

### Scenario
A file is deleted in a later version.

### Version History
```json
{
  "versionHistory": [
    {
      "id": "v1",
      "fileStates": {
        "temp-file.txt": {
          "inodeId": "...",
          "path": "temp-file.txt",
          "changeType": "added"
        }
      }
    },
    {
      "id": "v2",
      "fileStates": {
        "temp-file.txt": {
          "inodeId": "...",
          "path": "temp-file.txt",
          "deleted": true,
          "changeType": "deleted"
        }
      }
    }
  ]
}
```

### Notes
- File marked as `deleted: true` in version
- File removed from `content/` directory
- History still tracks the file (can be restored)
- Blob/delta still exists until GC runs

---

## Example 8: Tagged Versions

### Scenario
Important versions are tagged (e.g., releases).

### Manifest Excerpt
```json
{
  "versionHistory": [
    {
      "id": "v1.0.0-release",
      "message": "Release version 1.0.0",
      "tags": ["release", "v1.0.0", "stable"]
    },
    {
      "id": "v1.1.0-release",
      "message": "Release version 1.1.0",
      "tags": ["release", "v1.1.0"]
    }
  ],
  "refs": {
    "head": "v1.1.0-release",
    "stable": "v1.0.0-release"
  }
}
```

### Notes
- Tags stored in version metadata
- Refs can point to tagged versions
- Easy to checkout specific releases

---

## Validation Examples

### Valid Manifest
```json
{
  "formatVersion": "1.0.0",
  "metadata": {
    "author": "Developer",
    "created_at": "2025-01-15T10:00:00Z",
    "last_modified": "2025-01-15T10:00:00Z",
    "application": "JCF Manager"
  },
  "fileMap": {},
  "versionHistory": [],
  "refs": { "head": null }
}
```

### Invalid Manifest (Missing Required Fields)
```json
{
  "formatVersion": "1.0.0"
  // Missing: metadata, fileMap, versionHistory, refs
}
```

### Invalid Manifest (Broken Version Chain)
```json
{
  "versionHistory": [
    {
      "id": "v2",
      "parentId": "v1"  // v1 doesn't exist!
    }
  ]
}
```

---

## See Also

- [manifest-example.json](./manifest-example.json) - Complete manifest example
- [manifest-minimal.json](./manifest-minimal.json) - Minimal valid manifest
- [format-specification.md](./format-specification.md) - Format specification
- [delta-example.patch](./delta-example.patch) - Delta patch example
