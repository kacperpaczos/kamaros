# Memory Limits

[← Back: Corrupted Data](04-corrupted-data.md) | [Next: Concurrent Access →](06-concurrent-access.md)

## RAM Constraints

Use streaming for files >50MB:

```typescript
const stream = file.stream();
await manager.addFile('large.mp4', stream);
```

[← Back: Corrupted Data](04-corrupted-data.md) | [Next: Concurrent Access →](06-concurrent-access.md)
