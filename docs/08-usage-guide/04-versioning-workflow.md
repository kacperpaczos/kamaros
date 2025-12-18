# Versioning Workflow

[← Back: Basic Operations](03-basic-operations.md) | [Next: Working with Binaries →](05-working-with-binaries.md)

## Create & Restore Versions

```typescript
// Create checkpoint
const v1 = await manager.saveCheckpoint('Initial commit');

// Make changes...
await manager.addFile('new.js', code);

// Create another checkpoint
const v2 = await manager.saveCheckpoint('Add new.js');

// Restore to v1
await manager.restoreVersion(v1);
```

[← Back: Basic Operations](03-basic-operations.md) | [Next: Working with Binaries →](05-working-with-binaries.md)
