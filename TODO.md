# Kamaros TODO

## 🔴 Priorytet WYSOKI

### Browser Adapters
- [x] `IndexedDBAdapter` dla przeglądarek
- [x] `OPFSAdapter` (Origin Private File System)
- [x] Testy w przeglądarce (Playwright)
- [ ] Dokumentacja na docs.rs

---

## 🟡 Priorytet ŚREDNI

### Performance
- [ ] Streaming dla plików >50MB
- [ ] Web Workers dla hash/diff
- [ ] LRU Cache dla blobów

### Testy
- [ ] Testy integracyjne (end-to-end)
- [ ] Benchmarki performance
- [ ] Testy w przeglądarce

### CI/CD
- [ ] GitHub Actions workflow
- [ ] Automatyczne buildy WASM
- [ ] Automatyczne testy na PR

---

## 🟢 Priorytet NISKI (przyszłość)

### Features
- [ ] Branching support
- [ ] Merge conflict resolution
- [ ] Periodic snapshots (co N wersji)
- [ ] Garbage Collection CLI

### Bezpieczeństwo
- [ ] Encryption at rest
- [ ] Cloud sync (WebDAV, S3)

### Tooling
- [ ] CLI tool (`kamaros-cli`)
- [ ] VS Code extension
- [ ] GUI explorer

---

## ✅ Zrealizowane

- [x] Rust Core (domain, application, ports, infrastructure)
- [x] WASM bindings (`kamaros-wasm`)
- [x] TypeScript API (`kamaros-ts`)
- [x] Python API (`kamaros`)
- [x] Dokumentacja (74 pliki)
- [x] Testy jednostkowe (Rust 31, TS 21, Python 32)
- [x] Merge PR #4 (wasm-ts-bindings)
- [x] Merge PR #5 (unit-tests)
- [x] Integracja `SaveCheckpoint` / `RestoreVersion` (WASM & Python)
- [x] Implementacja Snapshot Storage (CAS)
- [x] Fix timestamp (`chrono`)

---

INne: dodać wspracie trzymania róznych wersji (róznych hedaów) i mozlwiość przęłacznaia isę zmiedzy nimi, jeden dokument moze mieć rózne "wersje" i nie jest to coś co trzeba mergować (nie iwem jak to nazwać).

Finalizacja: - [ ] npm publish `kamaros-ts`
- [ ] pip publish `kamaros`
*Ostatnia aktualizacja: 2026-01-18*
