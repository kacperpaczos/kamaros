# Building from Source

[← Back: Setup Environment](01-setup-environment.md) | [Next: Running Tests →](03-running-tests.md)

## Build Steps

```bash
# Rust → WASM
cd core && wasm-pack build

# TypeScript
cd ../js && npm run build

# Python
cd ../python && maturin build
```

[← Back: Setup Environment](01-setup-environment.md) | [Next: Running Tests →](03-running-tests.md)
