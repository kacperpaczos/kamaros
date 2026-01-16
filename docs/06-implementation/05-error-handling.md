# Obsługa błędów

## Rust Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum JCFError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Version not found: {version_id}")]
    VersionNotFound { version_id: String },

    #[error("Invalid manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Storage error: {source}")]
    Storage { #[from] source: std::io::Error },

    #[error("ZIP error: {source}")]
    Zip { #[from] source: zip::Error },

    #[error("JSON error: {source}")]
    Json { #[from] source: serde_json::Error },
}
```

## TypeScript Error Classes

```typescript
class JCFError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = 'JCFError';
  }
}

class FileNotFoundError extends JCFError {
  constructor(path: string) {
    super(`File not found: ${path}`, 'FILE_NOT_FOUND');
  }
}

class ValidationError extends JCFError {
  constructor(message: string) {
    super(`Validation error: ${message}`, 'VALIDATION_ERROR');
  }
}
```

## Python Exceptions

```python
class JCFError(Exception):
    def __init__(self, message: str, code: str):
        super().__init__(message)
        self.code = code

class FileNotFoundError(JCFError):
    def __init__(self, path: str):
        super().__init__(f"File not found: {path}", "FILE_NOT_FOUND")
```

## Error Propagation

### Rust → TypeScript
```rust
#[wasm_bindgen]
pub fn save_checkpoint(message: String) -> Result<String, JsValue> {
    // Implementation that returns Result
    Ok(version_id)
}
```

### TypeScript → User
```typescript
try {
  const versionId = await manager.saveCheckpoint(message);
} catch (error) {
  if (error instanceof FileNotFoundError) {
    // Handle file not found
  } else if (error instanceof ValidationError) {
    // Handle validation error
  } else {
    // Handle other errors
  }
}
```

## Best Practices

### Error Messages
- User-friendly and actionable
- Include context (file paths, version IDs)
- Suggest solutions when possible

### Error Recovery
- Retry logic for transient errors
- Fallback mechanisms
- Graceful degradation

### Logging
- Structured logging with error codes
- Include stack traces in development
- User-safe messages in production