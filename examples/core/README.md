# Rust Core Examples

> **Status**: 📋 Planned

## Planned Examples

| Example | Description |
|---------|-------------|
| `basic_usage.rs` | Basic manifest creation and manipulation |
| `save_checkpoint.rs` | SaveCheckpointUseCase integration |
| `restore_version.rs` | RestoreVersionUseCase integration |
| `storage_adapters.rs` | Custom StoragePort implementations |
| `wasm_integration.rs` | WASM bindings demonstration |

## Prerequisites

- Rust 1.70+
- Built `kamaros-corelib`

## Running (when implemented)

```bash
cd core
cargo run --example basic_usage
cargo run --example save_checkpoint
```

## Integration Tests

Core examples will also serve as integration tests:

```bash
cargo test --package kamaros-corelib
```

---

See [../python/demos.md](../python/demos.md) for Python API reference that Core implements.
