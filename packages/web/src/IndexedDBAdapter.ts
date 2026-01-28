/**
 * IndexedDBAdapter - Browser storage adapter using IndexedDB
 * 
 * Provides persistent storage for JCF files in the browser.
 * Uses a single object store with path as key and Uint8Array as value.
 */

import type { StorageAdapter } from '@kamaros/core-wasm';

const DB_NAME = 'kamaros-jcf';
const DB_VERSION = 1;
const STORE_NAME = 'files';

export class IndexedDBAdapter implements StorageAdapter {
    private dbPromise: Promise<IDBDatabase> | null = null;
    private prefix: string;

    /**
     * Create IndexedDB adapter
     * @param prefix - Optional prefix for all paths (e.g., project ID)
     */
    constructor(prefix: string = '') {
        this.prefix = prefix ? `${prefix}/` : '';
    }

    private getDB(): Promise<IDBDatabase> {
        if (this.dbPromise) {
            return this.dbPromise;
        }

        this.dbPromise = new Promise((resolve, reject) => {
            const request = indexedDB.open(DB_NAME, DB_VERSION);

            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result);

            request.onupgradeneeded = (event) => {
                const db = (event.target as IDBOpenDBRequest).result;
                if (!db.objectStoreNames.contains(STORE_NAME)) {
                    db.createObjectStore(STORE_NAME);
                }
            };
        });

        return this.dbPromise;
    }

    private fullPath(path: string): string {
        return this.prefix + path;
    }

    async read(path: string): Promise<Uint8Array> {
        const db = await this.getDB();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readonly');
            const store = tx.objectStore(STORE_NAME);
            const request = store.get(this.fullPath(path));

            request.onerror = () => reject(request.error);
            request.onsuccess = () => {
                if (request.result === undefined) {
                    reject(new Error(`File not found: ${path}`));
                } else {
                    resolve(request.result);
                }
            };
        });
    }

    async write(path: string, data: Uint8Array): Promise<void> {
        const db = await this.getDB();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);
            const request = store.put(data, this.fullPath(path));

            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve();
        });
    }

    async delete(path: string): Promise<void> {
        const db = await this.getDB();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);
            const request = store.delete(this.fullPath(path));

            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve();
        });
    }

    async exists(path: string): Promise<boolean> {
        const db = await this.getDB();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readonly');
            const store = tx.objectStore(STORE_NAME);
            const request = store.getKey(this.fullPath(path));

            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result !== undefined);
        });
    }

    async list(dir: string): Promise<string[]> {
        const db = await this.getDB();
        const fullDir = this.fullPath(dir);

        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readonly');
            const store = tx.objectStore(STORE_NAME);
            const request = store.getAllKeys();

            request.onerror = () => reject(request.error);
            request.onsuccess = () => {
                const keys = request.result as string[];
                const files = keys
                    .filter(key => key.startsWith(fullDir))
                    .map(key => key.slice(fullDir.length));
                resolve(files);
            };
        });
    }

    /**
     * Clear all data for this prefix
     */
    async clear(): Promise<void> {
        const db = await this.getDB();
        const allFiles = await this.list('');

        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);

            let completed = 0;
            const total = allFiles.length;

            if (total === 0) {
                resolve();
                return;
            }

            for (const file of allFiles) {
                const request = store.delete(this.fullPath(file));
                request.onerror = () => reject(request.error);
                request.onsuccess = () => {
                    completed++;
                    if (completed === total) {
                        resolve();
                    }
                };
            }
        });
    }

    /**
     * Get database size info (approximate)
     */
    async getStats(): Promise<{ fileCount: number; estimatedSize: number }> {
        const allFiles = await this.list('');
        let estimatedSize = 0;

        for (const file of allFiles) {
            try {
                const data = await this.read(file);
                estimatedSize += data.byteLength;
            } catch {
                // Skip files we can't read
            }
        }

        return {
            fileCount: allFiles.length,
            estimatedSize,
        };
    }
}
