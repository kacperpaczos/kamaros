/**
 * @kamaros/core-wasm
 * 
 * Core WASM bindings and shared types for Kamaros JCF format.
 * This package provides the JCFManager and MemoryAdapter.
 * 
 * For browser-specific adapters, use @kamaros/web.
 * For Node.js-specific adapters, use @kamaros/node.
 */

export * from './types';
export { JCFManager } from './JCFManager';
export { MemoryAdapter } from './MemoryAdapter';
export { initWasm, getWasm } from './wasm';
