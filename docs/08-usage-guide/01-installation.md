# Instalacja

## JavaScript/TypeScript

### npm
```bash
npm install kamaros
```

### yarn
```bash
yarn add kamaros
```

### pnpm
```bash
pnpm add kamaros
```

## Python (przyszłość)

### pip
```bash
pip install kamaros
```

### poetry
```bash
poetry add kamaros
```

## Rust (development)

### cargo
```bash
cargo add kamaros-core
```

## Wymagania systemowe

### JavaScript/TypeScript
- Node.js 16+ lub przeglądarka z ES2020
- Dla Web Workers: przeglądarka z Worker support
- Dla streaming: przeglądarka z ReadableStream support

### Python (przyszłość)
- Python 3.8+
- pip

### Rust (development)
- Rust 1.70+
- cargo

## Platformy wspierane

### JavaScript/TypeScript
- **Browser**: Chrome 90+, Firefox 88+, Safari 14+
- **Node.js**: 16+
- **Deno**: 1.20+
- **Bun**: 0.2+

### Python (przyszłość)
- CPython 3.8+
- PyPy 3.8+

## Rozmiar pakietu

### JavaScript/TypeScript
- **Core library**: ~50KB minified
- **WASM binary**: ~600KB
- **Total**: ~650KB (można zmniejszyć tree-shaking)

### Python (przyszłość)
- **Core library**: ~100KB
- **Compiled extension**: ~800KB
- **Total**: ~900KB

## Troubleshooting

### Browser: WASM loading fails
```javascript
// Ensure proper MIME type for .wasm files
// In webpack.config.js:
{
  test: /\.wasm$/,
  type: 'asset/resource',
}
```

### Node.js: Native dependencies
```bash
# For better performance on some operations
npm install @kamaros/native
```

### Python: Compilation issues
```bash
# Install build dependencies
pip install --upgrade pip setuptools wheel
pip install kamaros
```