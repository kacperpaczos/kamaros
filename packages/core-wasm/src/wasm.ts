import type { SaveCheckpointResult, GcResult } from './types';

// Type for WASM storage adapter (passed to WASM functions)
interface JsStorageAdapter {
    read(path: string): Promise<Uint8Array>;
    write(path: string, data: Uint8Array): Promise<void>;
    delete(path: string): Promise<void>;
    exists(path: string): Promise<boolean>;
    list(dir: string): Promise<string[]>;
    size(path: string): Promise<number>;
    list_blobs(): Promise<string[]>;
}

// Type for WASM exports
interface WasmExports {
    version(): string;
    greet(name: string): string;
    create_empty_manifest(projectName: string): unknown;
    parse_manifest(manifest: unknown): unknown;
    get_manifest_info(manifest: unknown): { name: string; versionCount: number; fileCount: number };
    derive_key(passphrase: string, salt: Uint8Array): Uint8Array;
    save_checkpoint(
        manifest: unknown,
        storage: JsStorageAdapter,
        message: string,
        author: string,
        encryption_key?: Uint8Array
    ): Promise<SaveCheckpointResult>;
    restore_version(
        manifest: unknown,
        storage: JsStorageAdapter,
        version_id: string,
        encryption_key?: Uint8Array
    ): Promise<{
        manifest: Record<string, unknown>;
        restoredVersionId: string;
        filesRestored: number;
        filesDeleted: number;
    }>;
    gc(
        manifest: unknown,
        storage: JsStorageAdapter
    ): Promise<GcResult>;
    export_zip(storage: JsStorageAdapter): Promise<Uint8Array>;
    import_zip(storage: JsStorageAdapter, archive_data: Uint8Array): Promise<{
        projectName: string;
        filesImported: number;
        totalSize: number;
    }>;
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
        const wasm = await import('./wasm-bindgen/kamaros_wasm.js');

        // Check if running in Node.js
        if (typeof process !== 'undefined' && process.versions && process.versions.node) {
            const fs = await import('fs/promises');
            const path = await import('path');
            const { fileURLToPath } = await import('url');

            // Resolve path to wasm file relative to this module
            const __dirname = path.dirname(fileURLToPath(import.meta.url));
            const wasmPath = path.resolve(__dirname, './wasm-bindgen/kamaros_wasm_bg.wasm');
            const buffer = await fs.readFile(wasmPath);
            await wasm.default(buffer);
        } else {
            await wasm.default();
        }

        wasmModule = {
            version: wasm.version,
            greet: wasm.greet,
            create_empty_manifest: wasm.create_empty_manifest,
            parse_manifest: wasm.parse_manifest,
            get_manifest_info: wasm.get_manifest_info,
            save_checkpoint: wasm.save_checkpoint,
            restore_version: wasm.restore_version,
            gc: wasm.gc,
            derive_key: wasm.derive_key,
            export_zip: wasm.export_zip,
            import_zip: wasm.import_zip,
        };

        return wasmModule;
    })();

    return initPromise as Promise<WasmExports>;
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
