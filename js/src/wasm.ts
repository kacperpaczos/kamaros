/**
 * WASM module lazy loading
 * 
 * Loads WASM only when first needed to reduce initial bundle impact.
 */

// Type for WASM exports
interface WasmExports {
    version(): string;
    greet(name: string): string;
    create_empty_manifest(projectName: string): unknown;
    parse_manifest(manifest: unknown): unknown;
    get_manifest_info(manifest: unknown): { name: string; versionCount: number; fileCount: number };
}

let wasmModule: WasmExports | null = null;
let initPromise: Promise<WasmExports> | null = null;

/**
 * Initialize WASM module (lazy loading)
 * 
 * @returns Promise that resolves when WASM is ready
 */
export async function initWasm(): Promise<WasmExports> {
    if (wasmModule) {
        return wasmModule;
    }

    if (initPromise) {
        return initPromise;
    }

    initPromise = (async () => {
        // Dynamic import for lazy loading
        const wasm = await import('../../wasm/pkg/kamaros_wasm.js');
        await wasm.default();

        wasmModule = {
            version: wasm.version,
            greet: wasm.greet,
            create_empty_manifest: wasm.create_empty_manifest,
            parse_manifest: wasm.parse_manifest,
            get_manifest_info: wasm.get_manifest_info,
        };

        return wasmModule;
    })();

    return initPromise;
}

/**
 * Get WASM module (throws if not initialized)
 */
export function getWasm(): WasmExports {
    if (!wasmModule) {
        throw new Error('WASM not initialized. Call initWasm() first.');
    }
    return wasmModule;
}

/**
 * Check if WASM is initialized
 */
export function isWasmReady(): boolean {
    return wasmModule !== null;
}
