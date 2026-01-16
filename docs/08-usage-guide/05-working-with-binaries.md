# Praca z plikami binarnymi

## Deduplikacja automatyczna

```typescript
// Pierwszy obraz
await manager.addFile('assets/logo.png', logoData);

// Ten sam obraz w innym miejscu - automatycznie deduplikowany
await manager.addFile('public/logo.png', logoData);

// Zmodyfikowany obraz - nowa kopia
await manager.addFile('assets/logo-dark.png', modifiedLogoData);
```

## Duże pliki binarne

### Streaming dla plików >50MB
```typescript
const file = largeFileInput.files[0];
const stream = file.stream();

await manager.addFile('video.mp4', stream);
```

### Progress tracking
```typescript
manager.on('checkpoint:progress', (data) => {
  if (data.phase === 'processing' && data.fileSize > 10 * 1024 * 1024) {
    console.log(`Processing ${data.fileName}: ${data.percent}%`);
  }
});
```

## Obsługa różnych typów plików

### Obrazy
```typescript
// PNG, JPG, WebP - automatycznie bez kompresji
await manager.addFile('photo.jpg', imageBuffer);
```

### Wideo/Audio
```typescript
// MP4, WebM, MP3 - streaming zalecany dla dużych plików
const videoFile = await fetch('/tutorial.mp4');
await manager.addFile('tutorial.mp4', videoFile.body);
```

### Dokumenty binarne
```typescript
// PDF, DOCX, XLSX
await manager.addFile('document.pdf', pdfBuffer);
```

## Wydajność

### Hash computation
- SHA-256 obliczony w Web Worker
- Nie blokuje UI
- Incremental dla dużych plików

### Storage efficiency
- CAS eliminuje duplikaty
- Smart compression tylko dla kompresowalnych plików
- Metadata tracking bez duplikacji danych

## Ograniczenia

### Browser storage limits
- IndexedDB: ~50MB - 1GB depending on browser
- Automatic cleanup of orphaned blobs
- Compression reduces storage needs

### Memory usage
- Large files processed in chunks
- Streaming prevents memory spikes
- LRU cache for frequently accessed binaries