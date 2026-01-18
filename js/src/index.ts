/**
 * Kamaros TypeScript Library
 * 
 * High-level API for managing JCF (JSON Content Format) files.
 * 
 * @example
 * ```typescript
 * import { JCFManager, NodeAdapter } from 'kamaros-ts';
 * 
 * const adapter = new NodeAdapter('./projects');
 * const manager = await JCFManager.create(adapter);
 * await manager.createProject('MyProject');
 * await manager.save('myproject.jcf');
 * ```
 * 
 * @packageDocumentation
 */

// Re-export types
export * from './types';

// Main API
export { JCFManager } from './api/JCFManager';

// Adapters
export { NodeAdapter } from './adapters/NodeAdapter';
export { MemoryAdapter } from './adapters/MemoryAdapter';

// WASM utilities (lazy loaded)
export { initWasm, getWasm } from './wasm';
