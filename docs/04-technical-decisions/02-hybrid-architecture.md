# Hybrid Architecture: Rust Core + TypeScript/Python Wrappers

> **Uzasadnienie wyboru architektury hybrydowej**

[← Back: Rust vs TypeScript](01-rust-vs-typescript.md) | [Next: Compression Library →](03-compression-library.md)

---

## Decision: Rust Core + Language Bindings

```
User Application (JS/Python)
         ↓
Language Wrapper (TS/Py)
         ↓
FFI Bindings (WASM/PyO3)
         ↓
Rust Core (kamaros-core)
```

## Benefits

1. **Performance**: Compiled Rust for heavy lifting
2. **Multi-language**: Write once, use everywhere
3. **Type Safety**: Rust's strong types + TS types
4. **Memory Efficient**: No GC in core
5. **Future-proof**: Easy to add more languages

## Trade-offs

- **+2 weeks** development time (Rust setup)
- **+600KB** WASM bundle (but faster)
- **Learning curve** (mitigated by AI/docs)

## Implementation

See [`PROJECT_STRUCTURE.md`](01-rust-vs-typescript.md) for detailed structure.

---

[← Back: Rust vs TypeScript](01-rust-vs-typescript.md) | [Next: Compression Library →](03-compression-library.md)

