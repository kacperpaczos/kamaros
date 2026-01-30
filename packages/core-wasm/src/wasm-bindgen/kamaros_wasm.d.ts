/* tslint:disable */
/* eslint-disable */

/**
 * Create empty manifest
 */
export function create_empty_manifest(project_name: string): any;

export function debug_diff(old: string, _new: string): string;

/**
 * Derive key from passphrase
 */
export function derive_key(passphrase: string, salt: Uint8Array): Uint8Array;

/**
 * Export project as ZIP archive
 */
export function export_zip(js_storage: JsStorageAdapter): Promise<Uint8Array>;

/**
 * Run Garbage Collection
 */
export function gc(js_manifest: any, js_storage: JsStorageAdapter): Promise<any>;

/**
 * Get manifest info (project name, version count)
 */
export function get_manifest_info(js_manifest: any): any;

/**
 * Simple test function
 */
export function greet(name: string): string;

/**
 * Import project from ZIP archive
 */
export function import_zip(js_storage: JsStorageAdapter, archive_data: Uint8Array): Promise<any>;

export function init(): void;

/**
 * Parse manifest from JavaScript object
 */
export function parse_manifest(js_manifest: any): any;

/**
 * Restore version - checkout specific version
 *
 * Restores working directory to state at version_id.
 *
 * @param js_manifest - current manifest object
 * @param js_storage - StorageAdapter
 * @param version_id - target version ID
 */
export function restore_version(js_manifest: any, js_storage: JsStorageAdapter, version_id: string, encryption_key?: Uint8Array | null): Promise<any>;

/**
 * Save checkpoint - create a new version
 *
 * Returns updated manifest with new version in history.
 *
 * @param js_manifest - current manifest object
 * @param js_storage - StorageAdapter with read/write/exists/list methods
 * @param message - commit message
 * @param author - author name
 */
export function save_checkpoint(js_manifest: any, js_storage: JsStorageAdapter, message: string, author: string, encryption_key?: Uint8Array | null): Promise<any>;

/**
 * Get library version
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly create_empty_manifest: (a: number, b: number) => [number, number, number];
    readonly debug_diff: (a: number, b: number, c: number, d: number) => [number, number];
    readonly derive_key: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly export_zip: (a: any) => any;
    readonly gc: (a: any, b: any) => any;
    readonly get_manifest_info: (a: any) => [number, number, number];
    readonly greet: (a: number, b: number) => [number, number];
    readonly import_zip: (a: any, b: number, c: number) => any;
    readonly parse_manifest: (a: any) => [number, number, number];
    readonly restore_version: (a: any, b: any, c: number, d: number, e: number, f: number) => any;
    readonly save_checkpoint: (a: any, b: any, c: number, d: number, e: number, f: number, g: number, h: number) => any;
    readonly version: () => [number, number];
    readonly init: () => void;
    readonly wasm_bindgen__closure__destroy__h47003da822dc748c: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h750fcbb69eed9bfd: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hfcca14c9a64256a5: (a: number, b: number, c: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
