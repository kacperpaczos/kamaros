# Kamaros JCF Manager

**Inteligentny format pliku ZIP z wersjonowaniem Time-Travel**

[JCF - JSON Content Format](docs/01-introduction/01-overview.md) to inteligentny format pliku oparty na standardowym ZIP archive, zaprojektowany do przechowywania projektów z pełną historią wersji.

## ⚡ Quick Start

### TypeScript (npm)

```typescript
import { JCFManager, MemoryAdapter } from 'kamaros-ts';

const manager = await JCFManager.create(new MemoryAdapter());
await manager.createProject("MyProject");
manager.addFile("main.ts", new TextEncoder().encode("console.log('Hello')"));
await manager.save("project.jcf");
```

### Python (pip)

```python
from kamaros import JCFManager, MemoryAdapter

manager = JCFManager(MemoryAdapter())
manager.create_project("MyProject")
manager.add_file("main.py", b"print('Hello')")
manager.save("project.jcf")
```

## 🔧 Development

### Build TypeScript

```bash
cd js && npm install && npm run build:ts
```

### Build Python (requires maturin)

```bash
pip install maturin
cd python && maturin build --release
```

### Build WASM

```bash
cd wasm && wasm-pack build --target web
```

## ✨ Kluczowe cechy

- **Format pliku**: Standardowy ZIP z inteligentną strukturą
- **Time-Travel**: Natychmiastowy dostęp do dowolnej wersji
- **Content Addressable Storage**: Deduplikacja plików binarnych
- **Reverse Delta**: Efektywne wersjonowanie plików tekstowych
- **Izomorficzny**: Przeglądarka, Node.js, Tauri, Python
- **Streaming**: Obsługa plików >500MB bez ładowania do RAM
- **Warstwy (Layers)**: Możliwość niezależnego dodawania treści do wybranych warstw

## 📁 Struktura projektu

```
kamaros/
├── core/              # Rust core library (Clean Architecture)
├── wasm/              # WASM bindings (wasm-bindgen)
├── js/                # TypeScript package (npm)
├── pyo3/              # PyO3 Rust bindings
├── python/            # Python package (pip)
└── docs/              # Documentation
```

## 📖 Dokumentacja

- [Wprowadzenie](docs/01-introduction/)
- [Architektura](docs/03-architecture/)
- [API Reference](docs/07-api-reference/)
- [Usage Guide](docs/08-usage-guide/)

## License

MIT
