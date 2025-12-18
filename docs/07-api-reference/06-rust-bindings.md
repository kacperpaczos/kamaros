# Rust Bindings (WASM)

[← Back: TypeScript Types](05-typescript-types.md) | [Next: Python Bindings →](07-python-bindings.md)

## WASM FFI

Uses wasm-bindgen for Rust ↔ JavaScript interop.

```rust
#[wasm_bindgen]
pub fn hash_content(data: &[u8]) -> String {
    // ...
}
```

[← Back: TypeScript Types](05-typescript-types.md) | [Next: Python Bindings →](07-python-bindings.md)
