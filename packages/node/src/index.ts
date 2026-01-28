/**
 * @kamaros/node
 * 
 * Node.js-specific storage adapter for Kamaros.
 * Uses fs/promises for file system operations.
 */

export { NodeAdapter } from './NodeAdapter';

// Re-export core for convenience
export { JCFManager, MemoryAdapter, initWasm, getWasm } from '@kamaros/core-wasm';
export type { StorageAdapter, Manifest } from '@kamaros/core-wasm';
