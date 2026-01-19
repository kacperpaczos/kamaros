# Platform-Specific Features

[← Back: Streaming Large Files](06-streaming-large-files.md) | [Next: Advanced Patterns →](08-advanced-patterns.md)

## Browser

### IndexedDB (Standard)
Best for general compatibility.

```typescript
import { IndexedDBAdapter } from 'kamaros-ts';
const adapter = new IndexedDBAdapter('my-project-scope');
```

### OPFS (High Permance)
Best for large files, requires Secure Context (HTTPS/localhost).

```typescript
import { OPFSAdapter } from 'kamaros-ts';

if (OPFSAdapter.isAvailable()) {
    const adapter = new OPFSAdapter('my-project-scope');
}
```

## Node.js

```typescript
import { NodeAdapter } from 'kamaros';
const adapter = new NodeAdapter('/path/to/project.jcf');
```

## Tauri

```typescript
import { TauriAdapter } from 'kamaros';
const adapter = new TauriAdapter();
```

[← Back: Streaming Large Files](06-streaming-large-files.md) | [Next: Advanced Patterns →](08-advanced-patterns.md)
