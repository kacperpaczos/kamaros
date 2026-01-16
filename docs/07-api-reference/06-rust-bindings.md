# Rust ABI / WASM Bindings

## Przegląd

Rust ABI zapewnia wysokowydajny core systemu JCF poprzez WebAssembly (WASM). Rust obsługuje operacje wymagające dużej mocy obliczeniowej jak kompresja, haszowanie, diffing i walidacja integralności.

## Architektura

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   JavaScript    │     │    WebAssembly   │     │      Rust       │
│   (TypeScript)  │◄───►│   Runtime        │◄───►│   Core Logic    │
│                 │     │   (wasm-bindgen) │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
         └────── FFI Interface ──┴────── Native Calls ──┘
```

## WASM Module Structure

### Główny moduł

```rust
// lib.rs
mod core;
mod compression;
mod hashing;
mod diffing;
mod validation;
mod utils;

use wasm_bindgen::prelude::*;

// Re-export dla JavaScript
#[wasm_bindgen]
extern "C" {
    // Import funkcji JS jeśli potrzebne
}

// Export funkcji do JavaScript
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Główna struktura WASM
#[wasm_bindgen]
pub struct JCFCore {
    // ...
}
```

### Core Operations

```rust
#[wasm_bindgen]
impl JCFCore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JCFCore {
        // Inicjalizacja
    }

    #[wasm_bindgen]
    pub fn process_file(&self, data: &[u8]) -> Result<JsValue, JsValue> {
        // Przetwarzanie pliku
    }
}
```

## Exported Functions

### Hashing & Cryptography

```rust
// SHA-256 haszowanie
#[wasm_bindgen]
pub fn sha256_hash(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// Batch haszowanie dla wydajności
#[wasm_bindgen]
pub fn sha256_batch(data: Vec<JsValue>) -> Vec<String> {
    data.into_iter()
        .map(|item| {
            let bytes: Vec<u8> = item.into_serde().unwrap();
            sha256_hash(&bytes)
        })
        .collect()
}
```

### Compression/Decompression

```rust
use fflate::{deflate, inflate};

#[wasm_bindgen]
pub fn compress_zlib(data: &[u8], level: i32) -> Result<Vec<u8>, JsValue> {
    deflate(data, level as u8).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    inflate(data).map_err(|e| JsValue::from_str(&e.to_string()))
}

// Streaming compression
#[wasm_bindgen]
pub fn create_compressor(level: i32) -> Compressor {
    Compressor::new(level as u8)
}

#[wasm_bindgen]
pub struct Compressor {
    // Implementacja streaming compression
}
```

### Diff & Patch Operations

```rust
use diff_match_patch::Dmp;

#[wasm_bindgen]
pub fn create_text_diff(old_text: &str, new_text: &str) -> String {
    let dmp = Dmp::new();
    let patches = dmp.patch_make_text(old_text, new_text);
    dmp.patch_to_text(&patches)
}

#[wasm_bindgen]
pub fn apply_text_patch(text: &str, patch: &str) -> Result<String, JsValue> {
    let dmp = Dmp::new();
    let patches = dmp.patch_from_text(patch)
        .map_err(|e| JsValue::from_str(&format!("Invalid patch: {}", e)))?;

    let (result, _) = dmp.patch_apply(&patches, text);
    Ok(result.concat())
}
```

### Binary Diff (rsync algorithm)

```rust
#[wasm_bindgen]
pub fn create_binary_diff(old_data: &[u8], new_data: &[u8]) -> Vec<u8> {
    // Implementacja binary diff algorithm
    // Zwraca deltę do zastosowania na old_data aby otrzymać new_data
}

#[wasm_bindgen]
pub fn apply_binary_diff(base_data: &[u8], delta: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Stosuje deltę do base_data
}
```

## Memory Management

### Zero-Copy Operations

```rust
// Bez kopiowania danych między JS i WASM
#[wasm_bindgen]
pub fn process_data(data: &mut [u8]) {
    // Modyfikacja danych in-place
    for byte in data.iter_mut() {
        *byte = byte.wrapping_add(1);
    }
}

// Transfer ownership
#[wasm_bindgen]
pub fn take_ownership(data: Vec<u8>) -> Vec<u8> {
    // WASM przejmuje ownership
    data.into_iter().map(|b| b + 1).collect()
}
```

### Memory Pool

```rust
#[wasm_bindgen]
pub struct MemoryPool {
    buffers: Vec<Vec<u8>>,
}

#[wasm_bindgen]
impl MemoryPool {
    #[wasm_bindgen]
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        let mut buffer = vec![0u8; size];
        let ptr = buffer.as_mut_ptr();
        self.buffers.push(buffer);
        ptr
    }

    #[wasm_bindgen]
    pub fn free(&mut self, ptr: *mut u8) {
        self.buffers.retain(|buf| buf.as_ptr() != ptr);
    }
}
```

## Error Handling

### WASM Error Types

```rust
#[derive(Debug, Clone)]
pub enum WasmError {
    InvalidInput(String),
    CompressionError(String),
    DecompressionError(String),
    HashError(String),
    DiffError(String),
    MemoryError(String),
}

impl From<WasmError> for JsValue {
    fn from(error: WasmError) -> JsValue {
        match error {
            WasmError::InvalidInput(msg) => js_sys::Error::new(&msg).into(),
            // ... inne przypadki
        }
    }
}
```

### Result Types

```rust
// Wrapper dla Result<T, E> do WASM
#[wasm_bindgen]
pub struct WasmResult {
    success: bool,
    value: Option<JsValue>,
    error: Option<String>,
}

#[wasm_bindgen]
impl WasmResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool { self.success }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Option<JsValue> { self.value.clone() }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> { self.error.clone() }
}

// Helper macro
macro_rules! wasm_result {
    ($expr:expr) => {
        match $expr {
            Ok(value) => WasmResult {
                success: true,
                value: Some(value.into()),
                error: None,
            },
            Err(error) => WasmResult {
                success: false,
                value: None,
                error: Some(error.to_string()),
            },
        }
    };
}
```

## Performance Optimizations

### SIMD Operations

```rust
#[cfg(target_feature = "simd128")]
#[wasm_bindgen]
pub fn sha256_simd(data: &[u8]) -> String {
    // SIMD-accelerated SHA-256
}

#[wasm_bindgen]
pub fn detect_simd() -> bool {
    cfg!(target_feature = "simd128")
}
```

### Web Workers Integration

```rust
#[wasm_bindgen]
pub fn init_worker_pool(size: usize) {
    // Inicjalizacja puli workerów
}

#[wasm_bindgen]
pub fn process_in_worker(data: JsValue, callback: &js_sys::Function) {
    // Przetwarzanie w workerze
}
```

### Memory Layout Optimization

```rust
// Aligned allocations dla lepszej wydajności
#[repr(align(32))]
struct AlignedBuffer([u8; 1024]);

// Cache-line aware structures
#[repr(C)]
struct CacheLineAligned {
    _padding: [u8; 64], // Cache line size
    data: u64,
}
```

## JavaScript Integration

### Loading WASM Module

```javascript
// Dynamic import
const jcfWasm = await import('./pkg/jcf_core.js');

// Inicjalizacja
await jcfWasm.default();

// Użycie
const core = new jcfWasm.JCFCore();
const hash = jcfWasm.sha256_hash(new Uint8Array([1, 2, 3]));
```

### TypeScript Definitions

```typescript
// Auto-generated przez wasm-bindgen
declare module 'jcf-core' {
  export function sha256_hash(data: Uint8Array): string;
  export function compress_zlib(data: Uint8Array, level: number): Uint8Array;
  export function create_text_diff(old: string, new: string): string;

  export class JCFCore {
    constructor();
    process_file(data: Uint8Array): WasmResult;
  }
}
```

### Error Handling in JS

```javascript
try {
  const result = core.process_file(data);
  if (!result.success) {
    throw new Error(result.error);
  }
  return result.value;
} catch (error) {
  if (error instanceof WebAssembly.RuntimeError) {
    console.error('WASM runtime error:', error);
  } else {
    console.error('JS error:', error);
  }
}
```

## Build System

### Cargo Configuration

```toml
# Cargo.toml
[package]
name = "jcf-core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = "0.3"
fflate = "0.7"
sha2 = "0.10"
diff-match-patch = "0.1"

[dependencies.wasm-bindgen]
version = "0.2"
features = [
  "serde-serialize",
]

[features]
default = ["console_error_panic_hook"]
console_error_panic_hook = ["dep:console_error_panic_hook"]
```

### Build Script

```bash
#!/bin/bash
# build-wasm.sh

# Build WASM
wasm-pack build --target web --out-dir pkg

# Optimize (optional)
wasm-opt -Oz pkg/jcf_core_bg.wasm -o pkg/jcf_core_bg.wasm

# Generate TypeScript definitions
wasm-bindgen --out-dir pkg --target web
```

### Webpack Integration

```javascript
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
  resolve: {
    fallback: {
      fs: false,
      path: false,
    },
  },
};
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 hex length
    }

    #[wasm_bindgen_test]
    fn test_compression() {
        let data = b"This is some test data for compression";
        let compressed = compress_zlib(data, 6).unwrap();
        let decompressed = decompress_zlib(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }
}
```

### Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sha256(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024]; // 1MB

    c.bench_function("sha256_1mb", |b| {
        b.iter(|| sha256_hash(black_box(&data)))
    });
}

criterion_group!(benches, bench_sha256);
criterion_main!(benches);
```

## Debugowanie

### Console Logging

```rust
use web_sys::console;

#[wasm_bindgen]
pub fn debug_log(message: &str) {
    console::log_1(&JsValue::from_str(message));
}

#[wasm_bindgen]
pub fn debug_error(message: &str) {
    console::error_1(&JsValue::from_str(message));
}
```

### Performance Profiling

```rust
#[wasm_bindgen]
pub fn profile_function<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = js_sys::Date::now();
    let result = f();
    let end = js_sys::Date::now();

    console::log_1(&JsValue::from_str(
        &format!("{} took {:.2}ms", name, end - start)
    ));

    result
}
```

## Distribution

### NPM Package

```json
{
  "name": "jcf-core",
  "version": "0.1.0",
  "files": [
    "pkg/"
  ],
  "main": "pkg/jcf_core.js",
  "types": "pkg/jcf_core.d.ts",
  "sideEffects": false
}
```

### CDN Distribution

```html
<script src="https://cdn.jsdelivr.net/npm/jcf-core@latest/pkg/jcf_core.js"></script>
<script>
  // Module zostanie załadowany automatycznie
  window.wasm_bindgen('https://cdn.jsdelivr.net/npm/jcf-core@latest/pkg/jcf_core_bg.wasm')
    .then(() => {
      // WASM gotowy do użycia
    });
</script>
```

## Security Considerations

### Sandboxing
- WASM działa w sandboxie przeglądarki
- Brak dostępu do host filesystem
- Limited memory access

### Input Validation
```rust
#[wasm_bindgen]
pub fn safe_process_data(input: &str) -> Result<String, JsValue> {
    // Walidacja input
    if input.len() > 10_000 {
        return Err(JsValue::from_str("Input too large"));
    }

    // Sanitization
    let clean_input = input.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>();

    Ok(process(clean_input))
}
```

### Memory Safety
- Rust zapewnia memory safety
- Bounds checking na wszystkich array access
- No null pointer dereferences

---

**Zobacz również:**
- [TypeScript Types](05-typescript-types.md) - Typy używane przez WASM bindings
- [Performance Rationale](../../04-technical-decisions/06-performance-rationale.md) - Uzasadnienie decyzji wydajnościowych