# @kamaros/web

Adaptery pamięci masowej dla przeglądarek internetowych.

Pakiet zawiera:
- **OPFSAdapter**: Adapter używający Origin Private File System (najszybszy, zalecany).
- **IndexedDBAdapter**: Adapter fallbackowy (dla starszych przeglądarek).
- Re-export wszystkich typów i klas z `@kamaros/core-wasm`.

## Instalacja

```bash
npm install @kamaros/web
```

## Użycie

```typescript
import { JCFManager, OPFSAdapter } from '@kamaros/web';

// Inicjalizacja z OPFS (wymaga HTTPS)
const manager = await JCFManager.create(new OPFSAdapter());
```
