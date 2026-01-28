/**
 * @kamaros/web
 * 
 * Browser-specific storage adapters for Kamaros.
 * - OPFSAdapter: Origin Private File System (high performance)
 * - IndexedDBAdapter: IndexedDB fallback (wider compatibility)
 */

export { OPFSAdapter } from './OPFSAdapter';
export { IndexedDBAdapter } from './IndexedDBAdapter';

// Re-export core for convenience
export { JCFManager, MemoryAdapter, initWasm, getWasm } from '@kamaros/core-wasm';
export type { StorageAdapter, Manifest } from '@kamaros/core-wasm';
