# File Renames

[← Back: Advanced Patterns](../08-usage-guide/08-advanced-patterns.md) | [Next: Type Changes →](02-type-changes.md)

## How Renames are Handled

Uses inode system - each file has unique UUID that persists across renames.

```typescript
await manager.moveFile('old.js', 'new.js');
// History preserved via inodeId
```

See algorithm docs for details.

[← Back: Advanced Patterns](../08-usage-guide/08-advanced-patterns.md) | [Next: Type Changes →](02-type-changes.md)
