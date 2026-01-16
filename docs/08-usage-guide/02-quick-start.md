# Szybki start

## JavaScript/TypeScript

### 1. Instalacja
```bash
npm install kamaros
```

### 2. Import
```typescript
import { JCFManager, BrowserAdapter } from 'kamaros';
```

### 3. Inicjalizacja
```typescript
const manager = new JCFManager();
await manager.init(new BrowserAdapter());
```

### 4. Dodanie plików
```typescript
await manager.addFile('README.md', '# My Project');
await manager.addFile('src/index.js', 'console.log("Hello World");');
```

### 5. Checkpoint
```typescript
const versionId = await manager.saveCheckpoint('Initial commit');
console.log('Created version:', versionId);
```

### 6. Przywracanie wersji
```typescript
await manager.restoreVersion(versionId);
```

## Python (przyszłość)

### 1. Instalacja
```bash
pip install kamaros
```

### 2. Import
```python
from kamaros import JCFManager, FileAdapter
```

### 3. Inicjalizacja
```python
manager = JCFManager()
await manager.init(FileAdapter('./my-project'))
```

### 4. Dodanie plików
```python
await manager.add_file('README.md', b'# My Project')
await manager.add_file('src/main.py', b'print("Hello World")')
```

### 5. Checkpoint
```python
version_id = await manager.save_checkpoint('Initial commit')
print(f'Created version: {version_id}')
```

## Przykład kompletny

### Tworzenie nowego projektu
```typescript
import { JCFManager, BrowserAdapter } from 'kamaros';

async function createProject() {
  // Inicjalizacja
  const manager = new JCFManager({
    author: 'Jan Kowalski',
    autoGC: true
  });

  await manager.init(new BrowserAdapter());

  // Dodanie struktury projektu
  await manager.addFile('package.json', JSON.stringify({
    name: 'my-app',
    version: '1.0.0',
    scripts: {
      start: 'node index.js'
    }
  }));

  await manager.addFile('index.js', `
const express = require('express');
const app = express();

app.get('/', (req, res) => {
  res.send('Hello World!');
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
  `.trim());

  // Pierwszy commit
  const v1 = await manager.saveCheckpoint('Initial setup');
  console.log('Version 1 created:', v1);

  // Dodanie funkcji
  await manager.addFile('utils.js', `
function formatDate(date) {
  return date.toISOString().split('T')[0];
}

module.exports = { formatDate };
  `.trim());

  // Drugi commit
  const v2 = await manager.saveCheckpoint('Add utility functions');
  console.log('Version 2 created:', v2);

  // Wyświetlenie historii
  const history = await manager.getHistory();
  console.log('Project history:');
  history.forEach(version => {
    console.log(`- ${version.id}: ${version.message}`);
  });
}

createProject().catch(console.error);
```

### Otwieranie istniejącego projektu
```typescript
async function openProject(jcfFile: File) {
  const manager = new JCFManager();
  await manager.init(new BrowserAdapter());

  // Import z pliku
  const buffer = await jcfFile.arrayBuffer();
  await manager.import(new Uint8Array(buffer));

  // Sprawdzenie zawartości
  const files = await manager.listFiles();
  console.log('Project files:', files);

  const currentVersion = await manager.getCurrentVersion();
  console.log('Current version:', currentVersion);
}
```

## Obsługa błędów

```typescript
try {
  await manager.saveCheckpoint('My changes');
} catch (error) {
  if (error.code === 'FILE_NOT_FOUND') {
    console.error('File not found:', error.message);
  } else if (error.code === 'VALIDATION_ERROR') {
    console.error('Invalid data:', error.message);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Zdarzenia

```typescript
// Monitorowanie postępu
manager.on('checkpoint:progress', (data) => {
  console.log(`Progress: ${data.percent}% - ${data.phase}`);
});

manager.on('checkpoint:complete', (data) => {
  console.log(`Checkpoint complete: ${data.versionId}`);
});
```

## Następne kroki

1. Przeczytaj [Basic Operations](03-basic-operations.md) dla więcej przykładów
2. Zobacz [Versioning Workflow](04-versioning-workflow.md) dla zaawansowanych operacji
3. Sprawdź [Platform Specific](07-platform-specific.md) dla szczegółów platformy