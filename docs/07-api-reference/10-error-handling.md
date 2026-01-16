# Error Handling & Exception Management

## Przegląd

System obsługi błędów JCF Manager zapewnia kompleksowe zarządzanie wyjątkami w całym stosie technologicznym. Błędy są kategoryzowane, kontekstualizowane i wyposażone w mechanizmy recovery.

## Architektura obsługi błędów

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Application   │────►│   Error Handler  │────►│   Recovery      │
│   Code          │     │   & Translator   │     │   Strategies     │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
         └────── Context ────────┴─────── Logging ──────┘
```

## Hierarchia błędów

### Base Error Classes

Wszystkie błędy dziedziczą po `JCFError`:

```typescript
class JCFError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: Record<string, any>,
    public recoverable: boolean = false
  ) {
    super(message);
    this.name = this.constructor.name;
  }

  // Helper methods
  isRecoverable(): boolean {
    return this.recoverable;
  }

  getContext(): Record<string, any> {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      details: this.details,
      stack: this.stack,
      timestamp: new Date().toISOString()
    };
  }
}
```

### Kategorie błędów

#### 1. Validation Errors

Błędy walidacji danych wejściowych:

```typescript
class ValidationError extends JCFError {
  constructor(message: string, public field?: string, public value?: any) {
    super(message, 'VALIDATION_ERROR', { field, value });
    this.name = 'ValidationError';
  }
}

class InvalidPathError extends ValidationError {
  constructor(path: string) {
    super(`Invalid path: ${path}`, 'path', path);
    this.name = 'InvalidPathError';
  }
}

class InvalidVersionIdError extends ValidationError {
  constructor(versionId: string) {
    super(`Invalid version ID: ${versionId}`, 'versionId', versionId);
    this.name = 'InvalidVersionIdError';
  }
}
```

#### 2. Resource Errors

Błędy dostępu do zasobów:

```typescript
class FileNotFoundError extends JCFError {
  constructor(path: string, public versionId?: string) {
    super(
      `File not found: ${path}${versionId ? ` in version ${versionId}` : ''}`,
      'FILE_NOT_FOUND',
      { path, versionId },
      true // recoverable
    );
    this.name = 'FileNotFoundError';
  }
}

class FileExistsError extends JCFError {
  constructor(path: string) {
    super(`File already exists: ${path}`, 'FILE_EXISTS', { path });
    this.name = 'FileExistsError';
  }
}

class VersionNotFoundError extends JCFError {
  constructor(versionId: string) {
    super(`Version not found: ${versionId}`, 'VERSION_NOT_FOUND', { versionId }, true);
    this.name = 'VersionNotFoundError';
  }
}
```

#### 3. Storage Errors

Błędy systemu plików i przechowywania:

```typescript
class StorageError extends JCFError {
  constructor(message: string, public originalError?: Error, public adapter?: string) {
    super(message, 'STORAGE_ERROR', {
      originalError: originalError?.message,
      adapter
    });
    this.name = 'StorageError';
  }
}

class InsufficientSpaceError extends StorageError {
  constructor(required: number, available: number) {
    super(
      `Insufficient space: ${required} required, ${available} available`,
      undefined,
      undefined
    );
    this.details = { required, available };
    this.name = 'InsufficientSpaceError';
  }
}

class FileTooLargeError extends StorageError {
  constructor(path: string, size: number, maxSize: number) {
    super(
      `File too large: ${path} (${size} bytes, max ${maxSize})`,
      undefined,
      undefined
    );
    this.details = { path, size, maxSize };
    this.name = 'FileTooLargeError';
  }
}
```

#### 4. Data Integrity Errors

Błędy związane z uszkodzeniem danych:

```typescript
class CorruptionError extends JCFError {
  constructor(message: string, public corruptedItem: string, public expected?: any, public actual?: any) {
    super(message, 'CORRUPTION_ERROR', {
      corruptedItem,
      expected,
      actual
    });
    this.name = 'CorruptionError';
  }
}

class ManifestCorruptionError extends CorruptionError {
  constructor(details: string) {
    super(`Manifest corruption: ${details}`, 'manifest');
    this.name = 'ManifestCorruptionError';
  }
}

class BlobCorruptionError extends CorruptionError {
  constructor(hash: string, expectedHash?: string, actualHash?: string) {
    super(
      `Blob corruption: ${hash}`,
      `blob:${hash}`,
      expectedHash,
      actualHash
    );
    this.name = 'BlobCorruptionError';
  }
}
```

#### 5. Operation Errors

Błędy operacyjne i runtime:

```typescript
class OperationTimeoutError extends JCFError {
  constructor(operation: string, timeout: number) {
    super(
      `Operation timeout: ${operation} (${timeout}ms)`,
      'OPERATION_TIMEOUT',
      { operation, timeout },
      true // recoverable - can retry
    );
    this.name = 'OperationTimeoutError';
  }
}

class OperationCancelledError extends JCFError {
  constructor(operation: string) {
    super(`Operation cancelled: ${operation}`, 'OPERATION_CANCELLED', { operation });
    this.name = 'OperationCancelledError';
  }
}

class ConcurrentModificationError extends JCFError {
  constructor(resource: string, conflictingOperation: string) {
    super(
      `Concurrent modification of ${resource} by ${conflictingOperation}`,
      'CONCURRENT_MODIFICATION',
      { resource, conflictingOperation }
    );
    this.name = 'ConcurrentModificationError';
  }
}
```

#### 6. Network Errors (dla adapterów)

```typescript
class NetworkError extends JCFError {
  constructor(message: string, public url?: string, public statusCode?: number) {
    super(message, 'NETWORK_ERROR', { url, statusCode }, true);
    this.name = 'NetworkError';
  }
}

class ConnectionTimeoutError extends NetworkError {
  constructor(url: string, timeout: number) {
    super(`Connection timeout: ${url} (${timeout}ms)`, url);
    this.name = 'ConnectionTimeoutError';
  }
}
```

## Strategie obsługi błędów

### 1. Try-Catch Patterns

#### Podstawowa obsługa

```typescript
try {
  await manager.saveCheckpoint('My changes');
} catch (error) {
  if (error instanceof FileNotFoundError) {
    console.error('File not found:', error.details.path);
    // Handle missing file
  } else if (error instanceof ValidationError) {
    console.error('Invalid input:', error.details.field);
    // Show validation error to user
  } else if (error instanceof StorageError) {
    console.error('Storage error:', error.originalError);
    // Handle storage issues
  } else {
    console.error('Unknown error:', error);
    // Log for debugging
  }
}
```

#### Kaskadowa obsługa

```typescript
async function saveWithFallback(message: string) {
  try {
    return await manager.saveCheckpoint(message);
  } catch (error) {
    if (error instanceof InsufficientSpaceError) {
      // Try to free space and retry
      await manager.runGC({ aggressive: true });
      return await manager.saveCheckpoint(message);
    } else if (error instanceof OperationTimeoutError) {
      // Increase timeout and retry
      const config = manager.getConfig();
      config.operationTimeout *= 2;
      manager.updateConfig(config);
      return await manager.saveCheckpoint(message);
    } else {
      throw error; // Re-throw unhandled errors
    }
  }
}
```

### 2. Error Recovery Patterns

#### Automatic Retry

```typescript
class RetryHandler {
  async execute<T>(
    operation: () => Promise<T>,
    options: {
      maxAttempts?: number;
      backoffMs?: number;
      retryableErrors?: string[];
    } = {}
  ): Promise<T> {
    const { maxAttempts = 3, backoffMs = 1000, retryableErrors = [] } = options;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        return await operation();
      } catch (error) {
        const shouldRetry =
          attempt < maxAttempts &&
          (retryableErrors.length === 0 ||
           retryableErrors.includes(error.code) ||
           error.recoverable);

        if (!shouldRetry) {
          throw error;
        }

        console.log(`Attempt ${attempt} failed, retrying in ${backoffMs}ms...`);
        await new Promise(resolve => setTimeout(resolve, backoffMs));
        backoffMs *= 2; // Exponential backoff
      }
    }
  }
}

// Usage
const retryHandler = new RetryHandler();
await retryHandler.execute(
  () => manager.saveCheckpoint('Important changes'),
  { maxAttempts: 5, retryableErrors: ['OPERATION_TIMEOUT', 'NETWORK_ERROR'] }
);
```

#### Circuit Breaker

```typescript
class CircuitBreaker {
  private failures = 0;
  private lastFailureTime = 0;
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';

  constructor(
    private failureThreshold: number = 5,
    private recoveryTimeout: number = 60000
  ) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime > this.recoveryTimeout) {
        this.state = 'HALF_OPEN';
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess() {
    this.failures = 0;
    this.state = 'CLOSED';
  }

  private onFailure() {
    this.failures++;
    this.lastFailureTime = Date.now();

    if (this.failures >= this.failureThreshold) {
      this.state = 'OPEN';
    }
  }
}
```

#### Graceful Degradation

```typescript
async function saveWithDegradation(message: string) {
  try {
    // Try full operation
    return await manager.saveCheckpoint(message);
  } catch (error) {
    if (error instanceof InsufficientSpaceError) {
      // Degrade: save without compression
      console.warn('Low space, saving without compression');
      const config = manager.getConfig();
      const originalCompression = config.compressionLevel;
      config.compressionLevel = 0; // No compression
      manager.updateConfig(config);

      try {
        return await manager.saveCheckpoint(message);
      } finally {
        // Restore original settings
        config.compressionLevel = originalCompression;
        manager.updateConfig(config);
      }
    } else {
      throw error;
    }
  }
}
```

### 3. Error Context and Enrichment

```typescript
class ErrorEnricher {
  enrich(error: JCFError, context: Record<string, any>): JCFError {
    return new (error.constructor as any)(
      error.message,
      error.code,
      { ...error.details, ...context },
      error.recoverable
    );
  }

  withUser(error: JCFError, userId: string): JCFError {
    return this.enrich(error, { userId });
  }

  withOperation(error: JCFError, operation: string, params?: any): JCFError {
    return this.enrich(error, { operation, params, timestamp: new Date().toISOString() });
  }

  withRequest(error: JCFError, requestId: string, url?: string): JCFError {
    return this.enrich(error, { requestId, url });
  }
}

// Usage
const enricher = new ErrorEnricher();

try {
  await manager.saveCheckpoint(message);
} catch (error) {
  throw enricher
    .withUser(error, currentUser.id)
    .withOperation(error, 'saveCheckpoint', { message })
    .withRequest(error, requestId);
}
```

## Error Monitoring and Logging

### Structured Logging

```typescript
class ErrorLogger {
  log(error: JCFError, level: 'error' | 'warn' | 'info' = 'error') {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level,
      error: error.getContext(),
      userAgent: navigator.userAgent,
      url: window.location.href,
      sessionId: getSessionId(),
      userId: getCurrentUserId()
    };

    // Send to logging service
    this.sendToService(logEntry);

    // Also log to console in development
    if (process.env.NODE_ENV === 'development') {
      console[level](error.message, error.getContext());
    }
  }

  private async sendToService(logEntry: any) {
    try {
      await fetch('/api/logs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(logEntry)
      });
    } catch (loggingError) {
      // Don't let logging errors break the app
      console.error('Failed to send log:', loggingError);
    }
  }
}

const errorLogger = new ErrorLogger();

// Global error handler
manager.on('error', (event) => {
  errorLogger.log(event.error);
});
```

### Error Aggregation

```typescript
class ErrorAggregator {
  private errors: Map<string, { count: number, lastSeen: Date, sample: JCFError }> = new Map();

  add(error: JCFError) {
    const key = `${error.code}:${error.message}`;

    if (this.errors.has(key)) {
      const entry = this.errors.get(key)!;
      entry.count++;
      entry.lastSeen = new Date();
    } else {
      this.errors.set(key, {
        count: 1,
        lastSeen: new Date(),
        sample: error
      });
    }
  }

  getReport(): ErrorReport[] {
    return Array.from(this.errors.entries()).map(([key, data]) => ({
      errorKey: key,
      count: data.count,
      lastSeen: data.lastSeen,
      sampleError: data.sample.getContext()
    }));
  }

  clear() {
    this.errors.clear();
  }
}

interface ErrorReport {
  errorKey: string;
  count: number;
  lastSeen: Date;
  sampleError: any;
}
```

### Metrics and Alerting

```typescript
class ErrorMetrics {
  private counters = new Map<string, number>();
  private alerts: AlertRule[] = [];

  addAlert(rule: AlertRule) {
    this.alerts.push(rule);
  }

  record(error: JCFError) {
    const key = error.code;
    this.counters.set(key, (this.counters.get(key) || 0) + 1);

    // Check alert rules
    this.checkAlerts(error);
  }

  private checkAlerts(error: JCFError) {
    for (const alert of this.alerts) {
      if (alert.condition(error)) {
        this.triggerAlert(alert, error);
      }
    }
  }

  private triggerAlert(alert: AlertRule, error: JCFError) {
    console.warn(`Alert triggered: ${alert.name}`, error.getContext());

    // Send alert to monitoring system
    fetch('/api/alerts', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        alert: alert.name,
        error: error.getContext(),
        timestamp: new Date().toISOString()
      })
    });
  }

  getMetrics(): Record<string, number> {
    return Object.fromEntries(this.counters);
  }
}

interface AlertRule {
  name: string;
  condition: (error: JCFError) => boolean;
  cooldownMs?: number;
}
```

## Best Practices

### 1. Error Classification

```typescript
function classifyError(error: JCFError): 'user' | 'system' | 'transient' {
  // User errors (4xx equivalent)
  if (error.code.startsWith('VALIDATION_') ||
      error.code === 'FILE_NOT_FOUND' ||
      error.code === 'VERSION_NOT_FOUND') {
    return 'user';
  }

  // Transient errors (retryable)
  if (error.code === 'OPERATION_TIMEOUT' ||
      error.code === 'NETWORK_ERROR' ||
      error.recoverable) {
    return 'transient';
  }

  // System errors (5xx equivalent)
  return 'system';
}

function handleByCategory(error: JCFError) {
  const category = classifyError(error);

  switch (category) {
    case 'user':
      showUserError(error.message);
      break;
    case 'transient':
      retryWithBackoff(() => operation());
      break;
    case 'system':
      logError(error);
      showGenericError();
      break;
  }
}
```

### 2. Error Boundaries

```typescript
// React-style error boundary
class JCFErrorBoundary {
  private errorHandler?: (error: JCFError) => void;

  setErrorHandler(handler: (error: JCFError) => void) {
    this.errorHandler = handler;
  }

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      if (error instanceof JCFError && this.errorHandler) {
        this.errorHandler(error);
      }
      throw error;
    }
  }
}

// Usage
const boundary = new JCFErrorBoundary();
boundary.setErrorHandler((error) => {
  if (error.code === 'FILE_NOT_FOUND') {
    // Handle gracefully
    return;
  }
  throw error; // Re-throw unhandled errors
});

await boundary.execute(() => manager.saveCheckpoint('test'));
```

### 3. Error Context Propagation

```typescript
class ErrorContext {
  private context: Record<string, any> = {};

  add(key: string, value: any): ErrorContext {
    this.context[key] = value;
    return this;
  }

  user(userId: string): ErrorContext {
    return this.add('userId', userId);
  }

  operation(op: string, params?: any): ErrorContext {
    return this.add('operation', { name: op, params });
  }

  wrap<T>(operation: () => Promise<T>): Promise<T> {
    return operation().catch(error => {
      if (error instanceof JCFError) {
        throw new (error.constructor as any)(
          error.message,
          error.code,
          { ...error.details, ...this.context },
          error.recoverable
        );
      }
      throw error;
    });
  }
}

// Usage
const context = new ErrorContext()
  .user('user123')
  .operation('saveCheckpoint', { message: 'test' });

await context.wrap(() => manager.saveCheckpoint('test'));
```

## Testing Error Scenarios

### Unit Tests for Errors

```typescript
describe('Error Handling', () => {
  let manager: JCFManager;

  beforeEach(async () => {
    manager = new JCFManager();
    await manager.init(new MemoryAdapter());
  });

  it('should throw ValidationError for invalid path', async () => {
    await expect(manager.addFile('', 'content'))
      .rejects.toThrow(ValidationError);

    await expect(manager.addFile('', 'content'))
      .rejects.toMatchObject({
        code: 'VALIDATION_ERROR',
        details: { field: 'path' }
      });
  });

  it('should throw FileNotFoundError for missing file', async () => {
    await expect(manager.getFileContent('nonexistent.txt'))
      .rejects.toThrow(FileNotFoundError);
  });

  it('should recover from InsufficientSpaceError', async () => {
    // Mock insufficient space
    const mockAdapter = new MemoryAdapter();
    Object.defineProperty(mockAdapter, 'maxFileSize', { value: 10 });

    const manager = new JCFManager();
    await manager.init(mockAdapter);

    // This should trigger GC and retry
    const result = await saveWithFallback('test message');
    expect(result).toBeDefined();
  });
});
```

### Integration Tests

```typescript
describe('Error Recovery Integration', () => {
  it('should handle network failures gracefully', async () => {
    // Mock network adapter with failures
    const failingAdapter = new NetworkAdapter('http://unreliable-server.com');

    // Mock fetch to sometimes fail
    global.fetch = jest.fn()
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValueOnce({ ok: true, arrayBuffer: () => Promise.resolve(new ArrayBuffer(0)) });

    const manager = new JCFManager();
    await manager.init(failingAdapter);

    // Should retry and succeed
    await expect(manager.listFiles()).resolves.toBeDefined();
  });
});
```

### Error Injection for Testing

```typescript
class ErrorInjector {
  private injectedErrors: Map<string, JCFError> = new Map();

  inject(operation: string, error: JCFError) {
    this.injectedErrors.set(operation, error);
  }

  clear() {
    this.injectedErrors.clear();
  }

  async execute<T>(operation: string, fn: () => Promise<T>): Promise<T> {
    if (this.injectedErrors.has(operation)) {
      throw this.injectedErrors.get(operation)!;
    }
    return fn();
  }
}

// Usage in tests
const injector = new ErrorInjector();

beforeEach(() => {
  injector.inject('saveCheckpoint', new InsufficientSpaceError(1000, 500));
});

it('should handle injected errors', async () => {
  await expect(
    injector.execute('saveCheckpoint', () => manager.saveCheckpoint('test'))
  ).rejects.toThrow(InsufficientSpaceError);
});
```

## Internationalization

### Localized Error Messages

```typescript
class LocalizedError extends JCFError {
  constructor(
    key: string,
    public locale: string = 'en',
    details?: Record<string, any>
  ) {
    const message = ErrorMessages.get(key, locale);
    super(message, key, details);
  }
}

class ErrorMessages {
  private static messages = {
    en: {
      FILE_NOT_FOUND: 'File not found: {path}',
      VALIDATION_ERROR: 'Invalid input: {field}',
      INSUFFICIENT_SPACE: 'Not enough space: {required} required, {available} available'
    },
    pl: {
      FILE_NOT_FOUND: 'Plik nie znaleziony: {path}',
      VALIDATION_ERROR: 'Nieprawidłowe dane: {field}',
      INSUFFICIENT_SPACE: 'Brak miejsca: wymagane {required}, dostępne {available}'
    }
  };

  static get(key: string, locale: string = 'en'): string {
    const localeMessages = this.messages[locale] || this.messages.en;
    return localeMessages[key] || key;
  }
}

// Usage
throw new LocalizedError('FILE_NOT_FOUND', 'pl', { path: '/test.txt' });
```

### Error Code Mapping

```typescript
const HTTPCodes = {
  // 4xx Client Errors
  VALIDATION_ERROR: 400,
  FILE_NOT_FOUND: 404,
  VERSION_NOT_FOUND: 404,
  FILE_EXISTS: 409,
  INVALID_PATH: 400,

  // 5xx Server Errors
  STORAGE_ERROR: 500,
  CORRUPTION_ERROR: 500,
  OPERATION_TIMEOUT: 504,
  INSUFFICIENT_SPACE: 507, // Insufficient Storage
  NETWORK_ERROR: 502
};

function getHTTPStatus(error: JCFError): number {
  return HTTPCodes[error.code] || 500;
}
```

---

**Zobacz również:**
- [Events API](09-events-api.md) - Event system i error events
- [TypeScript Types](05-typescript-types.md) - Error type definitions
- [Testing Guide](../../10-development/06-testing-strategy.md) - Error testing patterns