# 🚀 Quick Start - Pierwsze Kroki z JCF Manager

## 1. Instalacja

```bash
npm install jcf-manager
# lub
yarn add jcf-manager
# lub
pnpm add jcf-manager
```

## 2. Podstawowe Użycie

### 2.1 Tworzenie Nowego Projektu

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

// 1. Stwórz manager
const manager = new JCFManager({
  author: 'Twoje Imię',
  compressionLevel: 6
});

// 2. Inicjalizuj z adapterem
await manager.init(new BrowserAdapter());

// 3. Dodaj pliki
await manager.addFile('README.md', '# Mój Projekt\n\nOpis projektu...');
await manager.addFile('src/index.js', `
console.log('Hello World!');

function main() {
  // Your code here
}

main();
`);

// 4. Zapisz pierwszy checkpoint
const versionId = await manager.saveCheckpoint('Initial commit');
console.log(`Utworzono wersję: ${versionId}`);
```

### 2.2 Edycja i Kolejny Commit

```typescript
// 5. Edytuj pliki
await manager.addFile('src/index.js', `
console.log('Hello from v2!');

function main() {
  console.log('Updated version');
}

main();
`);

// Dodaj nowy plik
await manager.addFile('src/utils.js', `
export function add(a, b) {
  return a + b;
}

export function multiply(a, b) {
  return a * b;
}
`);

// 6. Zapisz zmiany
await manager.saveCheckpoint('Add utils module');
```

### 2.3 Przeglądanie Historii

```typescript
// Pobierz wszystkie wersje
const history = manager.getVersionHistory();

console.log('📜 Historia projektu:');
for (const version of history) {
  console.log(`  ${version.id}`);
  console.log(`    Autor: ${version.author}`);
  console.log(`    Data: ${new Date(version.timestamp).toLocaleString()}`);
  console.log(`    Wiadomość: ${version.message}`);
  console.log('');
}
```

### 2.4 Time Travel

```typescript
// Cofnij się do pierwszej wersji
const firstVersion = history[0];
await manager.restoreVersion(firstVersion.id);

console.log('✅ Przywrócono do pierwszej wersji!');

// Sprawdź zawartość
const content = await manager.getFile('src/index.js');
const text = new TextDecoder().decode(content);
console.log('Zawartość:', text);
```

## 3. Różne Platformy

### 3.1 Browser (IndexedDB)

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

const manager = new JCFManager();
await manager.init(new BrowserAdapter());

// ... użycie jak wyżej ...
```

### 3.2 Node.js

```typescript
import { JCFManager, NodeAdapter } from 'jcf-manager';

const manager = new JCFManager();
await manager.init(new NodeAdapter('./my-project'));

// ... użycie jak wyżej ...
```

### 3.3 Tauri Desktop App

```typescript
import { JCFManager, TauriAdapter } from 'jcf-manager';

const manager = new JCFManager();
await manager.init(new TauriAdapter());

// ... użycie jak wyżej ...
```

## 4. Export i Zapis

### 4.1 Export do Pliku (Browser)

```typescript
// Export całego projektu
const stream = await manager.export();
const blob = await new Response(stream).blob();

// Trigger download
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'moj-projekt.jcf';
a.click();
URL.revokeObjectURL(url);
```

### 4.2 Export do Pliku (Node.js)

```typescript
import { createWriteStream } from 'fs';
import { Readable } from 'stream';

const stream = await manager.export();
const nodeStream = Readable.fromWeb(stream);
const writeStream = createWriteStream('moj-projekt.jcf');

nodeStream.pipe(writeStream);

await new Promise((resolve, reject) => {
  writeStream.on('finish', resolve);
  writeStream.on('error', reject);
});

console.log('✅ Projekt zapisany do moj-projekt.jcf');
```

## 5. Otwieranie Istniejącego Projektu

### 5.1 Z Pliku (Browser)

```typescript
// HTML: <input type="file" id="fileInput" accept=".jcf">

const fileInput = document.getElementById('fileInput');
fileInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const arrayBuffer = await file.arrayBuffer();
  const data = new Uint8Array(arrayBuffer);
  
  const manager = new JCFManager();
  await manager.init(new BrowserAdapter(), data);
  
  console.log('✅ Projekt wczytany!');
  
  // Wyświetl statystyki
  const stats = await manager.getStats();
  console.log(`Plików: ${stats.totalFiles}`);
  console.log(`Wersji: ${stats.totalVersions}`);
});
```

### 5.2 Z Pliku (Node.js)

```typescript
import { readFile } from 'fs/promises';

const data = await readFile('projekt.jcf');

const manager = new JCFManager();
await manager.init(new NodeAdapter('./workspace'), data);

console.log('✅ Projekt wczytany!');
```

## 6. Praca z Plikami Binarnymi

```typescript
// Dodaj obraz
const imageResponse = await fetch('logo.png');
const imageBlob = await imageResponse.blob();
await manager.addFile('assets/logo.png', imageBlob);

// Dodaj wiele plików naraz
const files = [
  { path: 'images/photo1.jpg', data: photo1Data },
  { path: 'images/photo2.jpg', data: photo2Data },
  { path: 'videos/intro.mp4', data: videoData }
];

for (const file of files) {
  await manager.addFile(file.path, file.data);
}

await manager.saveCheckpoint('Add media files');
```

## 7. Porównywanie Wersji

```typescript
// Pobierz listę wersji
const versions = manager.getVersionHistory();

// Porównaj pierwszą i ostatnią
const diff = await manager.compareVersions(
  versions[0].id,
  versions[versions.length - 1].id
);

console.log('📊 Zmiany:');
console.log(`  Dodane pliki: ${diff.added.length}`);
console.log(`  Zmodyfikowane: ${diff.modified.length}`);
console.log(`  Usunięte: ${diff.deleted.length}`);

// Szczegóły
diff.added.forEach(path => console.log(`  + ${path}`));
diff.modified.forEach(path => console.log(`  ~ ${path}`));
diff.deleted.forEach(path => console.log(`  - ${path}`));
```

## 8. Garbage Collection

```typescript
// Po wielu zmianach, uruchom GC aby zwolnić miejsce
const report = await manager.runGC();

console.log('🗑️  Garbage Collection:');
console.log(`  Usunięte bloby: ${report.blobsRemoved}`);
console.log(`  Zwolniona przestrzeń: ${formatBytes(report.spaceFreed)}`);
console.log(`  Czas: ${report.duration}ms`);

// Helper function
function formatBytes(bytes: number): string {
  const sizes = ['B', 'KB', 'MB', 'GB'];
  if (bytes === 0) return '0 B';
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
}
```

## 9. Obsługa Błędów

```typescript
import { 
  FileTooLargeError,
  FileNotFoundError,
  VersionNotFoundError
} from 'jcf-manager';

try {
  await manager.addFile('huge-file.bin', hugeData);
} catch (error) {
  if (error instanceof FileTooLargeError) {
    console.error('Plik jest za duży!');
    console.error(`Maksymalny rozmiar: ${error.maxSize}`);
  } else {
    console.error('Nieznany błąd:', error);
  }
}

try {
  const content = await manager.getFile('nie-istnieje.txt');
} catch (error) {
  if (error instanceof FileNotFoundError) {
    console.error('Plik nie został znaleziony');
  }
}
```

## 10. Cleanup

```typescript
// Zawsze wywołaj dispose() przed zakończeniem
async function cleanup() {
  await manager.dispose();
  console.log('✅ Manager zamknięty poprawnie');
}

// Browser
window.addEventListener('beforeunload', cleanup);

// Node.js
process.on('SIGINT', async () => {
  await cleanup();
  process.exit(0);
});
```

## 11. Kompletny Przykład

```typescript
import { JCFManager, BrowserAdapter } from 'jcf-manager';

async function main() {
  // === SETUP ===
  const manager = new JCFManager({
    author: 'Jan Kowalski <jan@example.com>',
    compressionLevel: 6
  });
  
  try {
    await manager.init(new BrowserAdapter());
    console.log('✅ Manager zainicjalizowany');
    
    // === CREATE PROJECT ===
    await manager.addFile('README.md', '# Mój Projekt');
    await manager.addFile('package.json', JSON.stringify({
      name: 'my-project',
      version: '1.0.0'
    }, null, 2));
    
    const v1 = await manager.saveCheckpoint('Initial commit');
    console.log(`✅ Utworzono v1: ${v1}`);
    
    // === MAKE CHANGES ===
    await manager.addFile('src/index.js', 'console.log("Hello");');
    await manager.addFile('src/utils.js', 'export const add = (a,b) => a+b;');
    
    const v2 = await manager.saveCheckpoint('Add source files');
    console.log(`✅ Utworzono v2: ${v2}`);
    
    // === VIEW HISTORY ===
    const history = manager.getVersionHistory();
    console.log(`📜 Historia: ${history.length} wersji`);
    
    // === STATS ===
    const stats = await manager.getStats();
    console.log('📊 Statystyki:');
    console.log(`  Plików: ${stats.totalFiles}`);
    console.log(`  Rozmiar: ${formatBytes(stats.totalSize)}`);
    console.log(`  Deduplikacja: ${stats.deduplicationRatio.toFixed(2)}x`);
    
    // === EXPORT ===
    const stream = await manager.export();
    const blob = await new Response(stream).blob();
    console.log(`💾 Eksport: ${formatBytes(blob.size)}`);
    
    // === TIME TRAVEL ===
    await manager.restoreVersion(v1);
    console.log('⏪ Cofnięto do v1');
    
    const files = await manager.listFiles();
    console.log(`📁 Pliki w v1: ${files.map(f => f.path).join(', ')}`);
    
  } catch (error) {
    console.error('❌ Błąd:', error);
  } finally {
    await manager.dispose();
    console.log('👋 Cleanup wykonany');
  }
}

function formatBytes(bytes: number): string {
  const sizes = ['B', 'KB', 'MB', 'GB'];
  if (bytes === 0) return '0 B';
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
}

main().catch(console.error);
```

## 12. Następne Kroki

- Przeczytaj [Zaawansowane Operacje](./02-advanced.md)
- Zobacz [Streaming Dużych Plików](./03-streaming.md)
- Sprawdź [API Reference](../api/JCFManager.md)

---

**Ostatnia aktualizacja**: 2025-12-18  
**Wersja dokumentu**: 1.0.0

