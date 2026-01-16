# Algorytm diff

## Decyzja: diff-match-patch (Google)

### Dlaczego?
- Battle-tested (Google Docs)
- Complete API (diff + patch + fuzzy)
- Robust dla różnych typów zmian
- Good performance dla typowych plików

### Alternatywy
- **Myers diff**: Faster, but less robust
- **Patience diff**: Good for code, complex implementation
- **Histogram diff**: Slower, better for prose

### Implementacja

```typescript
import { diff_match_patch } from 'diff-match-patch';

const dmp = new diff_match_patch();

// Create diff
const patches = dmp.patch_make(oldText, newText);
const patchText = dmp.patch_toText(patches);

// Apply patch
const patches2 = dmp.patch_fromText(patchText);
const [result, success] = dmp.patch_apply(patches2, oldText);
```

### Użycie w systemie

- **Text files**: Full diff for each version change
- **Binary files**: No diff (CAS only)
- **Performance**: O(ND) where N=length, D=differences
- **Memory**: Linear with file size