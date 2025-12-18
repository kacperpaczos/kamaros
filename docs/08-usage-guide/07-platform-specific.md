# Platform-Specific Features

[← Back: Streaming Large Files](06-streaming-large-files.md) | [Next: Advanced Patterns →](08-advanced-patterns.md)

## Browser

```typescript
import { BrowserAdapter } from 'kamaros';
const adapter = new BrowserAdapter();
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
