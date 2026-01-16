# Biblioteka kompresji

## Decyzja: fflate

### Dlaczego?
- 20x szybsza kompresja niż JSZip
- Natywny streaming support
- Mniejszy rozmiar bundle (8KB min+gzip)
- Tree-shakeable

### Alternatywy rozważane

| Biblioteka | Performance | Bundle Size | Streaming | Winner |
|------------|-------------|-------------|-----------|--------|
| JSZip | 3/10 | 8/10 | 4/10 | ❌ |
| fflate | 9/10 | 9/10 | 10/10 | ✅ |
| pako | 7/10 | 7/10 | 8/10 | ❌ |

### Użycie

```typescript
import { Zip, Unzip } from 'fflate';

// Compression
const zip = new Zip();
zip.add('file.txt', new TextEncoder().encode('content'));
const compressed = await new Response(zip).arrayBuffer();

// Streaming compression
const zipStream = new ZipPassThrough('archive.zip');
zipStream.push(chunk1);
zipStream.push(chunk2, true); // Final chunk
```

### Integracja z systemem

- **Text files**: DEFLATE level 6 (balance speed/size)
- **Binary files**: STORE (already compressed)
- **Manifest**: DEFLATE level 9 (maximum compression)
- **Deltas**: DEFLATE level 9 (small files, maximum compression)

### Performance benchmarks

- **Compression ratio**: 2-5x depending on content
- **Speed**: 50-200 MB/s depending on level
- **Memory usage**: ~50MB for 500MB input (streaming)