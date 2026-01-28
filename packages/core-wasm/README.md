# @kamaros/core-wasm

Jądro systemu Kamaros dla środowisk JavaScript/TypeScript (WASM Bindings).

Pakiet zawiera:
- **JCFManager**: Główna klasa do zarządzania projektami `.jcf`.
- **MemoryAdapter**: Adapter pamięci RAM (do testów).
- **Types**: Współdzielone definicje typów (`Manifest`, `StorageAdapter`).

## Instalacja

```bash
npm install @kamaros/core-wasm
```

## Użycie (Isomorphic)

```typescript
import { JCFManager, MemoryAdapter } from '@kamaros/core-wasm';

const manager = await JCFManager.create(new MemoryAdapter());
await manager.createProject('Demo');
```
