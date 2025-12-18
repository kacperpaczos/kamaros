# Working with Binaries

[← Back: Versioning Workflow](04-versioning-workflow.md) | [Next: Streaming Large Files →](06-streaming-large-files.md)

## Adding Images/Videos

```typescript
// From file input
const file = fileInput.files[0];
await manager.addFile('logo.png', file);

// From URL
const response = await fetch(url);
const blob = await response.blob();
await manager.addFile('image.jpg', blob);
```

[← Back: Versioning Workflow](04-versioning-workflow.md) | [Next: Streaming Large Files →](06-streaming-large-files.md)
