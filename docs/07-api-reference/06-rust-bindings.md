# Rust ABI / WASM Bindings

## Przegląd

Kamaros wykorzystuje WebAssembly (WASM) do wykonywania kluczowych operacji logicznych (core logic) w przeglądarce i Node.js, zapewniając spójność algorytmiczną z natywną implementacją Rust oraz wysoką wydajność.

Moduł WASM (`kamaros-wasm`) jest wrapperem na `kamaros-corelib`.

## Architektura

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   JavaScript    │     │    WebAssembly   │     │      Rust       │
│   (TypeScript)  │◄───►│   Runtime        │◄───►│   Core Logic    │
│                 │     │   (wasm-bindgen) │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
         └────── JS Adapter ─────┴────── Native Calls ──┘
```

## API Reference (WASM)

Moduł eksportuje następujące elementy:

### Funkcje inicjalizacyjne

```typescript
// Inicjalizacja panic hook (dla lepszego debugowania błędów Rust w konsoli JS)
export function init_panic_hook(): void;

// Zwraca wersję biblioteki WASM
export function version(): string;
```

### Klasa `WasmJCFManager`

Główny punkt wejścia do logiki Core w WASM.

```rust
#[wasm_bindgen]
pub struct WasmJCFManager {
    // ... internal state
}
```

Metody (w TypeScript):

#### `save_checkpoint`

```typescript
save_checkpoint(
    manifest_json: string,  // Aktualny manifest
    message: string,        // Wiadomość commit
    author: string,         // Autor
    storage: JsStorageAdapter // Adapter do odczytu/zapisu plików
): Promise<ISaveCheckpointResult>
```

Zwraca `ISaveCheckpointResult`, zawierający zaktualizowany manifest JSON.

#### `restore_version`

```typescript
restore_version(
    manifest_json: string,
    current_files: string[],
    version_id: string,
    storage: JsStorageAdapter
): Promise<IRestoreVersionResult>
```

Zwraca plan przywracania lub wykonuje operacje (zależnie od implementacji `JsStorageAdapter`).

### Interfejs `JsStorageAdapter`

Aby WASM mógł korzystać z systemu plików (którego nie ma bezpośrednio), JavaScript musi dostarczyć obiekt implementujący ten interfejs. Kod Rust woła te metody np. by zapisać blob lub odczytać plik do diffowania.

```typescript
interface JsStorageAdapter {
    read(path: string): Promise<Uint8Array>;
    write(path: string, data: Uint8Array): Promise<void>;
    delete(path: string): Promise<void>;
    exists(path: string): Promise<boolean>;
    list(dir: string): Promise<string[]>;
}
```

## Budowanie i Dystrybucja

Projekt używa `wasm-pack` do budowania.

### Wymagania
- Rust 1.70+
- `wasm-pack` (`cargo install wasm-pack`)

### Komendy Build

```bash
# Budowanie dla przeglądarki (generuje ES modules)
npm run build:wasm
# co wykonuje:
cd wasm && wasm-pack build --target web --out-dir pkg

# Weryfikacja
cd wasm && cargo test
```

### Integracja z TypeScript

W kodzie TypeScript (`js/src/wasm.ts`) znajduje się logika ładowania modułu WASM:

```typescript
import init, { InitOutput } from "../wasm/pkg/kamaros_wasm.js";

let wasmInit: InitOutput | null = null;

export async function initWasm(): Promise<InitOutput> {
    if (wasmInit) return wasmInit;
    wasmInit = await init();
    return wasmInit;
}
```

Dzięki temu WASM jest ładowany leniwie (lazy loading) dopiero przy pierwszym użyciu `JCFManager`.

## Wydajność i Bezpieczeństwo

1.  **Shared Logic**: Ta sama logika (`SaveCheckpointUseCase`, `RestoreVersionUseCase`) jest używana w WASM i Python Extension.
2.  **Memory Management**: Dane binarne są przekazywane jako `Uint8Array`. Rust przejmuje własność nad danymi wewnątrz funkcji lub kopiuje je w zależności od potrzeb.
3.  **Panic Handling**: Używamy `console_error_panic_hook`, aby paniki Rusta były widoczne jako czytelne błędy w konsoli przeglądarki.

---

**Zobacz również:**
- [TypeScript Types](05-typescript-types.md)
- [Python Bindings](07-python-bindings.md)