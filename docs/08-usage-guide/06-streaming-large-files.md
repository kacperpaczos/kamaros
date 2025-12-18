# Streaming Large Files

[← Back: Working with Binaries](05-working-with-binaries.md) | [Next: Platform Specific →](07-platform-specific.md)

## >500MB Files

```typescript
// Use stream
const stream = file.stream();
await manager.addFile('video.mp4', stream);

// With progress
manager.on('file:progress', (e) => {
  console.log(`${e.percent}% uploaded`);
});
```

[← Back: Working with Binaries](05-working-with-binaries.md) | [Next: Platform Specific →](07-platform-specific.md)
