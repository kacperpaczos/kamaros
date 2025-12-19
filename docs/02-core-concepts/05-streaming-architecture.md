# Streaming Architecture

> **Obsługa dużych plików bez przepełnienia pamięci**

[← Back: Content Addressing](04-content-addressing.md) | [Next: Platform Abstraction →](06-platform-abstraction.md)

---

## Problem

Plik 500MB w przeglądarce → RAM overflow → crash.

## Solution: Streaming

```typescript
// BAD: Load entire file
const buffer = await file.arrayBuffer(); // 500MB w RAM!

// GOOD: Stream
const stream = file.stream();
await manager.addFile('video.mp4', stream); // Chunked processing
```

## Benefits

- ✅ RAM usage: ~50MB (chunks)
- ✅ No browser crash
- ✅ Progress tracking
- ✅ Cancellable

## Implementation

- ReadableStream API
- Chunked hashing (incremental SHA-256)
- ZIP streaming (fflate AsyncZipDeflate)

---

[← Back: Content Addressing](04-content-addressing.md) | [Next: Platform Abstraction →](06-platform-abstraction.md)


