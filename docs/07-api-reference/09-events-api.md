# Event System & Callbacks API

## Przegląd

System zdarzeń JCF Manager umożliwia reagowanie na operacje w czasie rzeczywistym, monitorowanie postępu oraz implementację custom logiki. Wszystkie główne operacje emitują zdarzenia z szczegółowymi informacjami.

## Architektura Event System

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Operation     │────►│   Event Emitter  │────►│   Subscribers   │
│   (JCFManager)  │     │                  │     │   (Callbacks)   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
         └────────────── Queue ──┴─── Async Delivery ───┘
```

### Event Emitter

Każdy `JCFManager` ma wbudowany event emitter:

```typescript
class EventEmitter {
  on(event: string, callback: Function): void
  off(event: string, callback?: Function): void
  once(event: string, callback: Function): void
  emit(event: string, data: any): void
  removeAllListeners(event?: string): void
  listenerCount(event: string): number
}
```

## Typy zdarzeń

### Operacyjne zdarzenia (Operations)

#### Checkpoint Events

```typescript
// Rozpoczęcie checkpoint
manager.on('checkpoint:start', (event: CheckpointStartEvent) => {
  console.log(`Starting checkpoint: ${event.message}`);
  // { type: 'checkpoint:start', message: string, timestamp: string }
});

// Postęp checkpoint
manager.on('checkpoint:progress', (event: CheckpointProgressEvent) => {
  console.log(`Checkpoint progress: ${event.percent}% - ${event.phase}`);
  // {
  //   type: 'checkpoint:progress',
  //   percent: number,
  //   phase: 'analyzing' | 'hashing' | 'saving' | 'updating',
  //   current: number,
  //   total: number,
  //   bytesProcessed: number,
  //   bytesTotal: number
  // }
});

// Zakończenie checkpoint
manager.on('checkpoint:complete', (event: CheckpointCompleteEvent) => {
  console.log(`Checkpoint complete: ${event.versionId}`);
  // {
  //   type: 'checkpoint:complete',
  //   versionId: string,
  //   message: string,
  //   filesChanged: number,
  //   filesAdded: number,
  //   filesModified: number,
  //   filesDeleted: number,
  //   duration: number
  // }
});

// Błąd checkpoint
manager.on('checkpoint:error', (event: CheckpointErrorEvent) => {
  console.error(`Checkpoint failed: ${event.error.message}`);
  // { type: 'checkpoint:error', error: Error, operation: string, timestamp: string }
});
```

#### Restore Events

```typescript
manager.on('restore:start', (event: RestoreStartEvent) => {
  console.log(`Starting restore to: ${event.versionId}`);
});

manager.on('restore:progress', (event: RestoreProgressEvent) => {
  console.log(`Restore: ${event.percent}% (${event.phase})`);
  // phases: 'loading', 'analyzing', 'applying', 'verifying'
});

manager.on('restore:complete', (event: RestoreCompleteEvent) => {
  console.log('Restore complete');
});

manager.on('restore:error', (event: RestoreErrorEvent) => {
  console.error('Restore failed:', event.error);
});
```

#### File Operation Events

```typescript
manager.on('file:add', (event: FileAddEvent) => {
  console.log(`File added: ${event.path} (${event.size} bytes)`);
});

manager.on('file:modify', (event: FileModifyEvent) => {
  console.log(`File modified: ${event.path}`);
});

manager.on('file:delete', (event: FileDeleteEvent) => {
  console.log(`File deleted: ${event.path}`);
});

manager.on('file:change', (event: FileChangeEvent) => {
  // Uniwersalne zdarzenie dla wszystkich zmian plików
  console.log(`File ${event.changeType}: ${event.path}`);
  // changeType: 'added' | 'modified' | 'deleted' | 'renamed'
});
```

### Maintenance Events

#### Garbage Collection Events

```typescript
manager.on('gc:start', (event: GCStartEvent) => {
  console.log('Starting garbage collection');
});

manager.on('gc:progress', (event: GCProgressEvent) => {
  console.log(`GC: ${event.percent}% (${event.phase})`);
  // phases: 'marking', 'sweeping', 'compacting', 'verifying'
});

manager.on('gc:complete', (event: GCCompleteEvent) => {
  console.log(`GC complete: ${event.spaceFreed} bytes freed`);
  // { blobsRemoved: number, deltasRemoved: number, spaceFreed: number, duration: number }
});
```

#### Verification Events

```typescript
manager.on('verify:start', (event: VerifyStartEvent) => {
  console.log('Starting integrity verification');
});

manager.on('verify:progress', (event: VerifyProgressEvent) => {
  console.log(`Verify: ${event.percent}% (${event.current}/${event.total} items)`);
});

manager.on('verify:complete', (event: VerifyCompleteEvent) => {
  if (event.report.valid) {
    console.log('Integrity check passed');
  } else {
    console.log(`${event.report.errors.length} errors found`);
  }
});
```

### Import/Export Events

```typescript
manager.on('export:start', (event: ExportStartEvent) => {
  console.log(`Starting export (${event.estimatedSize} bytes)`);
});

manager.on('export:progress', (event: ExportProgressEvent) => {
  console.log(`Export: ${event.percent}% (${event.bytesWritten} bytes)`);
});

manager.on('export:complete', (event: ExportCompleteEvent) => {
  console.log('Export complete');
});

manager.on('import:start', (event: ImportStartEvent) => {
  console.log('Starting import');
});

manager.on('import:progress', (event: ImportProgressEvent) => {
  console.log(`Import: ${event.percent}%`);
});

manager.on('import:complete', (event: ImportCompleteEvent) => {
  console.log(`Import complete: ${event.filesImported} files`);
});
```

### System Events

```typescript
// Inicjalizacja
manager.on('init', (event: InitEvent) => {
  console.log('Manager initialized');
});

// Błędy systemowe
manager.on('error', (event: ErrorEvent) => {
  console.error('System error:', event.error);
});

// Ostrzeżenia
manager.on('warning', (event: WarningEvent) => {
  console.warn('Warning:', event.message);
});
```

## Subskrypcja zdarzeń

### Podstawowa subskrypcja

```typescript
// Jednorazowa subskrypcja
manager.once('checkpoint:complete', (event) => {
  console.log('First checkpoint done!');
});

// Wielokrotna subskrypcja
const progressHandler = (event) => {
  console.log(`Progress: ${event.percent}%`);
};
manager.on('checkpoint:progress', progressHandler);

// Subskrypcja wielu zdarzeń
manager.on(['checkpoint:start', 'checkpoint:complete'], (event) => {
  console.log(`Checkpoint ${event.type.split(':')[1]}: ${event.message || event.versionId}`);
});
```

### Odsabskrypcja

```typescript
// Usunięcie konkretnego handler'a
manager.off('checkpoint:progress', progressHandler);

// Usunięcie wszystkich handler'ów dla zdarzenia
manager.off('checkpoint:progress');

// Usunięcie wszystkich subskrypcji
manager.removeAllListeners();

// Usunięcie subskrypcji dla konkretnego zdarzenia
manager.removeAllListeners('checkpoint:progress');
```

### Async Callbacks

```typescript
// Async callback
manager.on('file:add', async (event) => {
  try {
    // Asynchroniczna operacja
    await validateFile(event.path);
    await updateIndex(event.path);

    console.log(`File ${event.path} processed successfully`);
  } catch (error) {
    console.error(`Failed to process ${event.path}:`, error);
  }
});
```

## Event Data Types

Wszystkie zdarzenia implementują wspólny interfejs:

```typescript
interface BaseEvent {
  type: string;           // Typ zdarzenia
  timestamp: string;      // ISO 8601 timestamp
  operationId?: string;   // ID operacji (dla śledzenia)
}

interface ProgressEvent extends BaseEvent {
  percent: number;        // Procent ukończenia (0-100)
  current: number;        // Bieżąca wartość
  total: number;          // Całkowita wartość
  phase?: string;         // Faza operacji
  estimatedTimeLeft?: number; // Szacowany pozostały czas (ms)
}

interface ErrorEvent extends BaseEvent {
  error: Error;           // Obiekt błędu
  operation: string;      // Nazwa operacji która się nie udała
  recoverable: boolean;   // Czy błąd jest odwracalny
}

interface CompletionEvent extends BaseEvent {
  duration: number;       // Czas trwania operacji (ms)
  success: boolean;       // Czy operacja się udała
  result?: any;           // Wynik operacji (jeśli applicable)
}
```

## Advanced Patterns

### Event Filtering

```typescript
// Filtrowanie zdarzeń na podstawie warunków
const largeFileHandler = (event) => {
  if (event.size > 1024 * 1024) { // > 1MB
    console.log(`Large file added: ${event.path}`);
  }
};
manager.on('file:add', largeFileHandler);

// Chain of responsibility
const eventFilters = [
  (event) => event.type.startsWith('checkpoint:'),
  (event) => event.timestamp > new Date(Date.now() - 60000), // Ostatnie 60s
];

const filteredHandler = (event) => {
  if (eventFilters.every(filter => filter(event))) {
    console.log('Filtered event:', event);
  }
};
```

### Event Buffering

```typescript
class EventBuffer {
  private buffer: any[] = [];
  private maxSize: number;

  constructor(maxSize = 100) {
    this.maxSize = maxSize;
  }

  add(event: any) {
    this.buffer.push(event);
    if (this.buffer.length > this.maxSize) {
      this.buffer.shift();
    }
  }

  getEvents(since?: Date): any[] {
    if (!since) return [...this.buffer];

    return this.buffer.filter(event =>
      new Date(event.timestamp) > since
    );
  }

  clear() {
    this.buffer = [];
  }
}

const eventBuffer = new EventBuffer(1000);

// Buffer wszystkich zdarzeń
manager.on('*', (event) => {
  eventBuffer.add(event);
});

// Pobierz zdarzenia z ostatniej godziny
const recentEvents = eventBuffer.getEvents(
  new Date(Date.now() - 60 * 60 * 1000)
);
```

### Event Aggregation

```typescript
class EventAggregator {
  private stats = {
    checkpoints: 0,
    filesAdded: 0,
    filesModified: 0,
    totalSize: 0
  };

  constructor(manager: JCFManager) {
    manager.on('checkpoint:complete', () => {
      this.stats.checkpoints++;
      this.report();
    });

    manager.on('file:add', (event) => {
      this.stats.filesAdded++;
      this.stats.totalSize += event.size;
    });

    manager.on('file:modify', (event) => {
      this.stats.filesModified++;
    });
  }

  report() {
    console.log('Session stats:', this.stats);
  }

  reset() {
    this.stats = { checkpoints: 0, filesAdded: 0, filesModified: 0, totalSize: 0 };
  }
}

const aggregator = new EventAggregator(manager);
```

### Custom Events

```typescript
// Emitowanie custom zdarzeń
class ExtendedJCFManager extends JCFManager {
  async customOperation() {
    this.emit('custom:start', { operation: 'my-op' });

    try {
      // Operacja...
      await this.doSomething();

      this.emit('custom:complete', {
        operation: 'my-op',
        result: 'success'
      });
    } catch (error) {
      this.emit('custom:error', {
        operation: 'my-op',
        error: error.message
      });
    }
  }
}

// Użycie
const manager = new ExtendedJCFManager();
manager.on('custom:*', (event) => {
  console.log('Custom event:', event.type, event);
});
```

## Performance Considerations

### Efficient Event Handling

```typescript
// ❌ Nieefektywne - wiele subskrypcji
manager.on('checkpoint:progress', () => updateProgressBar());
manager.on('restore:progress', () => updateProgressBar());
manager.on('gc:progress', () => updateProgressBar());

// ✅ Efektywne - jedna subskrypcja z filtrowaniem
manager.on('*', (event) => {
  if (event.type.endsWith(':progress')) {
    updateProgressBar(event.percent);
  }
});
```

### Debouncing

```typescript
function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

// Debounced logging
const debouncedLog = debounce((event) => {
  console.log('Progress update:', event.percent);
}, 100);

manager.on('*:progress', debouncedLog);
```

### Memory Management

```typescript
// Cleanup przy zniszczeniu managera
class ManagedEventHandler {
  private handlers: Array<{event: string, handler: Function}> = [];

  constructor(private manager: JCFManager) {}

  on(event: string, handler: Function) {
    this.manager.on(event, handler);
    this.handlers.push({event, handler});
  }

  destroy() {
    this.handlers.forEach(({event, handler}) => {
      this.manager.off(event, handler);
    });
    this.handlers = [];
  }
}

const handler = new ManagedEventHandler(manager);
// ... użycie ...
handler.destroy(); // Cleanup
```

## Error Handling in Events

### Event Handler Errors

```typescript
// Bezpieczna obsługa błędów w callbackach
manager.on('file:add', (event) => {
  try {
    processFile(event.path);
  } catch (error) {
    // Nie pozwól żeby błąd w jednym handlerze zatrzymał inne
    console.error('File processing failed:', error);

    // Opcjonalnie emituj błąd dalej
    manager.emit('handler:error', {
      originalEvent: event,
      error: error,
      handler: 'file:add'
    });
  }
});

// Global error handler dla event handler'ów
manager.on('handler:error', (event) => {
  console.error('Event handler failed:', event);
  // Log to monitoring system
  reportError(event);
});
```

### Async Error Propagation

```typescript
manager.on('operation:complete', async (event) => {
  try {
    await postProcess(event.result);
  } catch (error) {
    // Async błędy nie zatrzymują innych handler'ów
    console.error('Post-processing failed:', error);

    // Możesz emitować async błąd
    setImmediate(() => {
      manager.emit('postprocess:error', { originalEvent: event, error });
    });
  }
});
```

## Testing Event Handlers

```typescript
describe('Event System', () => {
  let manager: JCFManager;

  beforeEach(async () => {
    manager = new JCFManager();
    await manager.init(new MemoryAdapter());
  });

  it('should emit checkpoint events', async () => {
    const events: any[] = [];

    manager.on('checkpoint:start', (e) => events.push(e));
    manager.on('checkpoint:complete', (e) => events.push(e));

    await manager.save_checkpoint('test');

    expect(events).toHaveLength(2);
    expect(events[0].type).toBe('checkpoint:start');
    expect(events[1].type).toBe('checkpoint:complete');
  });

  it('should handle event handler errors gracefully', async () => {
    const errorHandler = jest.fn();
    manager.on('handler:error', errorHandler);

    manager.on('test:event', () => {
      throw new Error('Test error');
    });

    manager.emit('test:event', {});

    expect(errorHandler).toHaveBeenCalled();
  });
});
```

## Integration Examples

### UI Progress Bars

```typescript
class ProgressUI {
  constructor(manager: JCFManager) {
    this.setupProgressHandlers(manager);
  }

  private setupProgressHandlers(manager: JCFManager) {
    const progressBars = new Map<string, HTMLElement>();

    manager.on('*', (event) => {
      if (event.type.includes(':progress')) {
        this.updateProgressBar(event, progressBars);
      } else if (event.type.includes(':complete')) {
        this.hideProgressBar(event, progressBars);
      }
    });
  }

  private updateProgressBar(event: ProgressEvent, bars: Map<string, HTMLElement>) {
    const key = event.operationId || event.type;
    let bar = bars.get(key);

    if (!bar) {
      bar = this.createProgressBar(event);
      bars.set(key, bar);
    }

    this.setProgress(bar, event.percent);
  }

  private createProgressBar(event: ProgressEvent): HTMLElement {
    // Create and show progress bar
    const bar = document.createElement('div');
    bar.className = 'progress-bar';
    // ... setup UI ...
    return bar;
  }

  private setProgress(bar: HTMLElement, percent: number) {
    bar.style.width = `${percent}%`;
  }

  private hideProgressBar(event: CompletionEvent, bars: Map<string, HTMLElement>) {
    const key = event.operationId || event.type.replace(':complete', ':progress');
    const bar = bars.get(key);
    if (bar) {
      bar.style.display = 'none';
      bars.delete(key);
    }
  }
}
```

### Logging System

```typescript
class EventLogger {
  private logs: any[] = [];

  constructor(manager: JCFManager) {
    manager.on('*', (event) => {
      this.logEvent(event);
    });
  }

  private logEvent(event: any) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      event: event.type,
      data: event,
      sessionId: this.sessionId
    };

    this.logs.push(logEntry);

    // Optional: persist to storage
    if (this.shouldPersist(event)) {
      this.persistLog(logEntry);
    }
  }

  private shouldPersist(event: any): boolean {
    // Persist important events
    return ['error', 'complete'].some(type => event.type.includes(type));
  }

  getLogs(since?: Date): any[] {
    return this.logs.filter(log =>
      !since || new Date(log.timestamp) > since
    );
  }

  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }
}
```

---

**Zobacz również:**
- [JCFManager Class](01-jcf-manager-class.md) - Metody emitujące zdarzenia
- [Error Handling](10-error-handling.md) - Obsługa błędów w zdarzeniach
- [Performance Guide](../../04-technical-decisions/06-performance-rationale.md) - Wydajność event system