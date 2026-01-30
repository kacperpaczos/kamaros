/**
 * In-memory storage adapter for testing
 */

import type { StorageAdapter } from './types';

export class MemoryAdapter implements StorageAdapter {
    private storage = new Map<string, Uint8Array>();

    async read(path: string): Promise<Uint8Array> {
        const data = this.storage.get(path);
        if (!data) {
            throw new Error(`File not found: ${path}`);
        }
        return data;
    }

    async write(path: string, data: Uint8Array): Promise<void> {
        this.storage.set(path, data);
    }

    async delete(path: string): Promise<void> {
        this.storage.delete(path);
    }

    async exists(path: string): Promise<boolean> {
        return this.storage.has(path);
    }

    async list(dir: string): Promise<string[]> {
        const prefix = dir.endsWith('/') ? dir : `${dir}/`;
        return Array.from(this.storage.keys())
            .filter(key => key.startsWith(prefix))
            .map(key => key.slice(prefix.length).split('/')[0])
            .filter((v, i, a) => a.indexOf(v) === i); // unique
    }

    async size(path: string): Promise<number> {
        const data = this.storage.get(path);
        return data ? data.length : 0;
    }

    async listBlobs(): Promise<string[]> {
        return Array.from(this.storage.keys())
            .filter(key => key.startsWith('.store/blobs/'));
    }

    /**
     * Clear all stored data (for testing)
     */
    clear(): void {
        this.storage.clear();
    }

    /**
     * Get all stored paths (for debugging)
     */
    getAllPaths(): string[] {
        return Array.from(this.storage.keys());
    }
}
