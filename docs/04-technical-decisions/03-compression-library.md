# Compression Library Choice

[← Back: Hybrid Architecture](02-hybrid-architecture.md) | [Next: Diff Algorithm →](04-diff-algorithm.md)

## Decision: fflate

**Why?**
- 20x faster compression than JSZip
- Native streaming support
- Smaller bundle size (8KB min+gzip)
- Tree-shakeable

See IMPLEMENTATION_SPEC.md Section 2.1.1 for details.

[← Back: Hybrid Architecture](02-hybrid-architecture.md) | [Next: Diff Algorithm →](04-diff-algorithm.md)
