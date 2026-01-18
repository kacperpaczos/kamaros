/**
 * WASM module lazy loading
 * 
 * Loads WASM only when first needed to reduce initial bundle impact.
 */

// Type for WASM storage adapter (passed to WASM functions)
interface JsStorageAdapter {
    read(path: string): Promise<Uint8Array>;
    write(path: string, data: Uint8Array): Promise<void>;
    delete(path: string): Promise<void>;
    exists(path: string): Promise<boolean>;
    list(dir: string): Promise<string[]>;
}

// Type for save_checkpoint result
interface SaveCheckpointResult {
    manifest: Record<string, unknown>;
    versionId: string;
    filesAdded: number;
    filesModified: number;
    filesDeleted: number;
}

// Type for WASM exports
interface WasmExports {
    version(): string;
    greet(name: string): string;
    create_empty_manifest(projectName: string): unknown;
    parse_manifest(manifest: unknown): unknown;
    get_manifest_info(manifest: unknown): { name: string; versionCount: number; fileCount: number };
    save_checkpoint(
        manifest: unknown,
        storage: JsStorageAdapter,
        message: string,
        author: string
    ): Promise<SaveCheckpointResult>;
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
            save_checkpoint: wasm.save_checkpoint,
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
