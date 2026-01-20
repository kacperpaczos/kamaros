# Kamaros Examples

This directory contains examples demonstrating the Kamaros library across all supported platforms.

## Structure

| Directory | Language | Status |
|-----------|----------|--------|
| [python/](python/) | Python 3.8+ | ✅ Complete |
| [js/](js/) | TypeScript/JavaScript | 📋 Planned |
| [core/](core/) | Rust | 📋 Planned |

## Python Examples

Full API demonstration with 16 functions tested:

| Example | Description |
|---------|-------------|
| `simple_workflow.py` | Basic create/save/restore workflow |
| `comprehensive_demo.py` | Full workflow with images from internet |
| `api_reference_demo.py` | Tests all 16 API functions |

**Run:**
```bash
source .venv/bin/activate
python examples/python/api_reference_demo.py
```

## JavaScript Examples

Coming soon. Will include:
- Node.js examples with `NodeAdapter`
- Browser examples with `IndexedDBAdapter` and `OPFSAdapter`

## Core (Rust) Examples

Coming soon. Will include:
- Integration tests with Rust Core
- WASM bindings examples

---

See [python/demos.md](python/demos.md) for API function reference and roadmap.
