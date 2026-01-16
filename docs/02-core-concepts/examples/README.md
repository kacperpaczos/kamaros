# JCF Format Examples

This directory contains example files demonstrating the JCF (JSON Content Format) specification and structure.

## Files

### Specification Files

- **[format-specification.md](./format-specification.md)** - Complete format specification with examples
  - File structure explanation
  - Manifest.json structure
  - Delta patch format
  - Blob storage (CAS)
  - Validation rules
  - Usage examples

- **[usage-examples.md](./usage-examples.md)** - Practical usage scenarios
  - Simple text file project
  - Binary assets project
  - Mixed content project
  - File rename tracking
  - Large file streaming
  - Deduplication examples
  - Deleted files
  - Tagged versions

### Example Data Files

- **[manifest-example.json](./manifest-example.json)** - Complete manifest.json example
  - Full project with 3 versions
  - Text and binary files
  - Version history chain
  - File metadata
  - Configuration

- **[manifest-minimal.json](./manifest-minimal.json)** - Minimal valid manifest.json
  - Simplest possible valid manifest
  - Single file, single version
  - Useful for understanding required fields

- **[delta-example.patch](./delta-example.patch)** - Example reverse delta patch
  - Unified diff format
  - Shows how text changes are stored
  - Reverse delta strategy example

## Quick Start

1. **Read the specification**: Start with [format-specification.md](./format-specification.md)
2. **Check examples**: Look at [manifest-example.json](./manifest-example.json) for structure
3. **See usage**: Review [usage-examples.md](./usage-examples.md) for practical scenarios

## Understanding the Format

### Key Concepts

1. **JCF is a ZIP-based format** - Can be opened with standard ZIP tools
2. **Manifest.json defines everything** - Structure, history, metadata
3. **Content/ is current state** - Full files, no reconstruction needed
4. **.store/ is versioning** - Blobs (CAS) and deltas (reverse patches)
5. **Library works by specification** - Reads/writes/validates according to manifest

### File Types

- **Text files** → Reverse delta patches in `.store/deltas/`
- **Binary files** → Content Addressable Storage in `.store/blobs/`
- **Detection is for versioning only** - Library doesn't interpret content

## Validation

All example files are valid according to JCF format specification v1.0.0.

To validate your own JCF files:

```typescript
const manager = new JCFManager();
await manager.init(adapter);
await manager.import(jcfFile);
const report = await manager.verifyIntegrity();
```

## Related Documentation

- [JCF Format Specification](../01-jcf-format.md) - Full specification
- [Time-Travel Versioning](../02-time-travel-versioning.md) - Versioning concepts
- [Reverse Delta Strategy](../03-reverse-delta-strategy.md) - Delta strategy
- [Content Addressing](../04-content-addressing.md) - CAS implementation
