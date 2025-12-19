# Streaming Architecture

Obsługa dużych plików bez przepełnienia pamięci.

## Problem

Plik 500MB w przeglądarce → RAM overflow → crash.

## Rozwiązanie: Streaming

```typescript
// BAD: Load entire file
const buffer = await file.arrayBuffer(); // 500MB w RAM!

// GOOD: Stream
const stream = file.stream();
await manager.addFile('video.mp4', stream); // Chunked processing
```

## Korzyści

- ✅ RAM usage: ~50MB (chunks)
- ✅ No browser crash
- ✅ Progress tracking
- ✅ Cancellable