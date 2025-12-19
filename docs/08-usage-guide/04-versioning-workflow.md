# Workflow wersjonowania

## Podstawowy workflow

### 1. Inicjalizacja projektu
```typescript
const manager = new JCFManager({
  author: 'Developer Name',
  autoGC: true
});
await manager.init(new BrowserAdapter());
```

### 2. Rozwój iteracyjny
```typescript
// Dodanie funkcji
await manager.addFile('src/auth.js', `
export function login(username, password) {
  // Implementation
}
`);

// Test
await manager.addFile('src/auth.test.js', `
import { login } from './auth.js';
// Tests
`);

// Commit
await manager.saveCheckpoint('Add authentication module');
```

### 3. Refactoring
```typescript
// Zmiana struktury
await manager.addFile('src/utils/auth.js', ''); // Nowy plik
await manager.deleteFile('src/auth.js'); // Usuń stary

// Commit z opisem zmian
await manager.saveCheckpoint('Refactor: Move auth to utils/');
```

## Zaawansowane wzorce

### Branching (przyszłość)
```typescript
// Tworzenie brancha
const branchId = await manager.createBranch('feature/login');

// Przełączanie
await manager.checkoutBranch(branchId);

// Merge (przyszłość)
await manager.mergeBranch('main');
```

### Tagowanie wersji
```typescript
// Tag dla release
await manager.createTag('v1.0.0', 'Production release');

// Tag dla milestone
await manager.createTag('beta', 'Beta testing ready');
```

## Historia i analiza

### Analiza zmian
```typescript
const history = await manager.getHistory();

// Znajdź commits autora
const myCommits = history.filter(v => v.author === 'Jan Kowalski');

// Znajdź commits zawierające plik
const fileHistory = await manager.getFileHistory('src/index.js');
```

### Diff między wersjami
```typescript
// Porównaj dwie wersje
const diff = await manager.compareVersions('v1', 'v2');

// Pokaż zmienione pliki
diff.changedFiles.forEach(file => {
  console.log(`${file.path}: ${file.changeType}`);
});
```

## Backup i recovery

### Regularne backup
```typescript
async function backupProject() {
  const stream = await manager.export();
  const blob = new Blob([await new Response(stream).arrayBuffer()]);

  // Zapisz do IndexedDB lub pobierz
  await saveToStorage('backup.jcf', blob);
}
```

### Recovery
```typescript
async function restoreFromBackup(backupBlob: Blob) {
  const manager = new JCFManager();
  await manager.init(new BrowserAdapter());

  const buffer = await backupBlob.arrayBuffer();
  await manager.import(new Uint8Array(buffer));
}
```

## Współpraca

### Udostępnianie projektu
```typescript
// Eksport do współdzielenia
const shareableBlob = await manager.exportToBlob();
const shareUrl = await uploadToCloud(shareableBlob);

// Import od współpartnera
const downloadedBlob = await downloadFromCloud(shareUrl);
await manager.import(downloadedBlob);
```

### Merge conflicts (przyszłość)
```typescript
// Wykrywanie konfliktów
const conflicts = await manager.detectConflicts(otherProject);

// Ręczne rozwiązywanie
await manager.resolveConflict('src/index.js', 'manual', mergedContent);

// Auto-merge gdzie możliwe
await manager.autoMerge(otherProject);
```