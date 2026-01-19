# Kamaros Python Library

Python bindings for Kamaros - intelligent ZIP with Time-Travel versioning.

## Development

This library uses `pyo3` and `maturin` to bind to the Rust `kamaros-corelib`.

### Prerequisites
- Rust (latest stable)
- Python 3.8+
- `maturin`

### Building locally

```bash
# Install maturin
pip install maturin

# Build and install in current venv
maturin develop
```

Or manually with Cargo (not recommended for production):
```bash
cargo build -p kamaros-py
cp target/debug/libkamaros.so python/kamaros/_native.so
```

### Running Tests

```bash
pytest
```
Note: Tests require the native extension to be built and available.
