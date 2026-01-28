# @kamaros/node

Adapter pamięci masowej dla środowiska Node.js.

Pakiet zawiera:
- **NodeAdapter**: Adapter używający systemowego `fs`.
- Re-export wszystkich typów i klas z `@kamaros/core-wasm`.

## Instalacja

```bash
npm install @kamaros/node
```

## Użycie

```typescript
import { JCFManager, NodeAdapter } from '@kamaros/node';

// Inicjalizacja w katalogu './projects'
const manager = await JCFManager.create(new NodeAdapter('./projects'));
```
