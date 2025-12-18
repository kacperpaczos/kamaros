# Orphaned Blobs

[← Back: Type Changes](02-type-changes.md) | [Next: Corrupted Data →](04-corrupted-data.md)

## Garbage Collection

```typescript
const report = await manager.runGC({
  gracePeriodDays: 30
});

console.log(`Removed ${report.blobsRemoved} orphaned blobs`);
console.log(`Freed ${report.spaceFreed} bytes`);
```

[← Back: Type Changes](02-type-changes.md) | [Next: Corrupted Data →](04-corrupted-data.md)
