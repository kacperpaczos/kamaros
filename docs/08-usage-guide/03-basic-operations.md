# Basic Operations

[← Back: Quick Start](02-quick-start.md) | [Next: Versioning Workflow →](04-versioning-workflow.md)

## CRUD Operations

```typescript
// Add file
await manager.addFile('index.js', content);

// Read file
const content = await manager.getFile('index.js');

// Delete file
await manager.deleteFile('index.js');

// List files
const files = await manager.listFiles();
```

[← Back: Quick Start](02-quick-start.md) | [Next: Versioning Workflow →](04-versioning-workflow.md)
