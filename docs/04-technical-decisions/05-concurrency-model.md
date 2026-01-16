# Model współbieżności

## Decyzja: Web Workers

### Problem
CPU-intensive operations (hashing, diffing, compression) powodują freeze UI w przeglądarce.

### Rozwiązanie
Offload ciężkich operacji do Web Workers.

### Architektura

```typescript
// Main thread
class JCFManager {
  private worker: Worker;

  async saveCheckpoint(message: string): Promise<string> {
    // Send to worker
    const result = await this.callWorker('saveCheckpoint', {
      message,
      files: this.changedFiles
    });

    return result.versionId;
  }

  private callWorker(method: string, data: any): Promise<any> {
    return new Promise((resolve, reject) => {
      const id = Math.random();

      const handler = (event: MessageEvent) => {
        if (event.data.id === id) {
          this.worker.removeEventListener('message', handler);
          if (event.data.error) {
            reject(new Error(event.data.error));
          } else {
            resolve(event.data.result);
          }
        }
      };

      this.worker.addEventListener('message', handler);
      this.worker.postMessage({ id, method, data });
    });
  }
}

// Worker thread
self.addEventListener('message', async (event) => {
  const { id, method, data } = event.data;

  try {
    let result;

    switch (method) {
      case 'saveCheckpoint':
        result = await saveCheckpointWorker(data);
        break;
      case 'hashFile':
        result = await hashFileWorker(data);
        break;
      case 'computeDiff':
        result = await computeDiffWorker(data);
        break;
    }

    self.postMessage({ id, result });
  } catch (error) {
    self.postMessage({ id, error: error.message });
  }
});
```

### Workers używane dla

- **SHA-256 hashing**: Large files
- **Diff computation**: Text changes
- **ZIP compression**: Large archives
- **JSON parsing**: Large manifests

### Fallback
Graceful degradation - jeśli Web Workers nie są dostępne, operacje wykonują się na main thread z progress indicators.

### Performance impact
- **UI responsiveness**: Main thread <10% CPU during operations
- **Overall speed**: Slight overhead from message passing (~5-10%)
- **Memory**: Separate heap for worker (additional ~50MB)