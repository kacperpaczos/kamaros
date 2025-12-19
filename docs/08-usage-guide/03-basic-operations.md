# Podstawowe operacje

## Zarządzanie plikami

### Dodawanie plików

```typescript
// Plik tekstowy
await manager.addFile('src/index.js', 'console.log("Hello");');

// Plik binarny
const imageData = await fetch('/logo.png').then(r => r.arrayBuffer());
await manager.addFile('assets/logo.png', new Uint8Array(imageData));

// Zastąpienie pliku
await manager.addFile('src/index.js', 'console.log("Updated");');
```

### Odczyt plików

```typescript
// Zawartość jako Uint8Array
const content = await manager.getFileContent('src/index.js');
console.log(new TextDecoder().decode(content));

// Lista plików
const allFiles = await manager.listFiles();
const srcFiles = await manager.listFiles('src/');
```

### Usuwanie plików

```typescript
await manager.deleteFile('old-file.js');
```

## Zarządzanie wersjami

### Tworzenie checkpoint

```typescript
// Prosty commit
const versionId = await manager.saveCheckpoint('Add login feature');

// Commit z metadanymi
const versionId2 = await manager.saveCheckpoint('Fix bug', {
  author: 'Jan Kowalski',
  email: 'jan@example.com',
  tags: ['bugfix', 'urgent']
});
```

### Przywracanie wersji

```typescript
// Przywracanie do konkretnej wersji
await manager.restoreVersion('v1');

// Przywracanie o N commitów wstecz
const history = await manager.getHistory();
const targetVersion = history[history.length - 3]; // 3 commity wstecz
await manager.restoreVersion(targetVersion.id);
```

### Historia wersji

```typescript
// Wszystkie wersje
const history = await manager.getHistory();

// Szczegóły wersji
const version = await manager.getVersion('v1');
console.log(version.message, version.timestamp);

// Bieżąca wersja
const currentId = await manager.getCurrentVersion();
```

## Import/Export

### Eksport projektu

```typescript
// Eksport jako stream (dla przeglądarki)
const stream = await manager.export();
const blob = new Blob([await new Response(stream).arrayBuffer()]);
const url = URL.createObjectURL(blob);

// Eksport do pliku (Node.js)
const { writeFile } = require('fs/promises');
const stream = await manager.export();
const buffer = await new Response(stream).arrayBuffer();
await writeFile('project.jcf', new Uint8Array(buffer));
```

### Import projektu

```typescript
// Z pliku (przeglądarka)
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const file = fileInput.files[0];
const buffer = await file.arrayBuffer();
await manager.import(new Uint8Array(buffer));

// Z pliku (Node.js)
const { readFile } = require('fs/promises');
const buffer = await readFile('project.jcf');
await manager.import(new Uint8Array(buffer));
```

## Garbage Collection

### Ręczne uruchomienie

```typescript
const report = await manager.runGC();
console.log(`Removed ${report.blobsRemoved} blobs, freed ${report.spaceFreed} bytes`);
```

### Automatyczne GC

```typescript
// Włącz automatyczne GC
const manager = new JCFManager({
  autoGC: true,
  maxHistorySize: 100 // MB
});
```

## Praca z dużymi plikami

### Streaming upload

```typescript
// Dla plików > 50MB
const file = fileInput.files[0];
const stream = file.stream();
await manager.addFile('large-video.mp4', stream);
```

### Chunked processing

```typescript
// System automatycznie dzieli duże pliki na chunk'i
// Progress reporting dla dużych operacji
manager.on('checkpoint:progress', (data) => {
  if (data.fileSize > 10 * 1024 * 1024) { // >10MB
    console.log(`Processing large file: ${data.percent}%`);
  }
});
```

## Obsługa błędów

### Typowe błędy

```typescript
try {
  await manager.addFile('', 'content'); // Invalid path
} catch (error) {
  if (error.code === 'VALIDATION_ERROR') {
    console.log('Invalid file path');
  }
}

try {
  await manager.restoreVersion('nonexistent');
} catch (error) {
  if (error.code === 'VERSION_NOT_FOUND') {
    console.log('Version does not exist');
  }
}
```

### Retry logic

```typescript
async function saveWithRetry(message: string, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await manager.saveCheckpoint(message);
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      console.log(`Attempt ${i + 1} failed, retrying...`);
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
}
```

## Wydajność

### Optymalizacje

```typescript
// Cache manifest w pamięci
const manifest = await manager.getManifest();

// Batch operations
await Promise.all([
  manager.addFile('file1.js', content1),
  manager.addFile('file2.js', content2),
  manager.addFile('file3.js', content3),
]);

const versionId = await manager.saveCheckpoint('Batch update');
```

### Monitoring wydajności

```typescript
manager.on('checkpoint:complete', (data) => {
  console.log(`Checkpoint took ${data.duration}ms`);
  console.log(`Created version: ${data.versionId}`);
});
```