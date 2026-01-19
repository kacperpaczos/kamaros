# Kamaros Usage Examples

## Python

The Python examples demonstrate the Core logic binding via PyO3.

### Prerequisites

- Python 3.8+
- Rust toolchain (for building the extension)
- Virtual environment recommended

### Running `simple_workflow.py`

1. Create and activate a virtual environment:
   ```bash
   python3 -m venv .venv
   source .venv/bin/activate
   ```

2. Install dependencies and build the package (from project root):
   ```bash
   pip install maturin
   maturin develop --manifest-path python/Cargo.toml
   # OR if running from root with pyproject.toml in python/ dir:
   (cd python && maturin develop)
   ```

3. Run the example:
   ```bash
   python3 examples/python/simple_workflow.py
   ```

   Expected output:
   - Creation of `demo-project-store` directory.
   - Initial commit of README.md.
   - Update of README.md.
   - Successful restoration of the initial version.

## TypeScript / JavaScript

Examples for the JS/TS library are located in `ts/`.
To run them, ensure the `kamaros-ts` library is built.
