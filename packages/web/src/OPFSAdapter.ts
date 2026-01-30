/**
 * OPFSAdapter - Browser storage adapter using Origin Private File System
 * 
 * OPFS provides a high-performance, synchronous file system API for browsers.
 * Best for large files and frequent access patterns.
 * 
 * Note: OPFS is only available in secure contexts (HTTPS) and modern browsers.
 */

import type { StorageAdapter } from '@kamaros/core-wasm';

export class OPFSAdapter implements StorageAdapter {
    private rootPromise: Promise<FileSystemDirectoryHandle> | null = null;
    private prefix: string;

    /**
     * Create OPFS adapter
     * @param prefix - Optional prefix/subdirectory for all paths
     */
    constructor(prefix: string = 'kamaros') {
        this.prefix = prefix;
    }

    private async getRoot(): Promise<FileSystemDirectoryHandle> {
        if (this.rootPromise) {
            return this.rootPromise;
        }

        this.rootPromise = (async () => {
            const opfsRoot = await navigator.storage.getDirectory();
            // Create or get the prefix directory
            return await opfsRoot.getDirectoryHandle(this.prefix, { create: true });
        })();

        return this.rootPromise;
    }

    /**
     * Get directory handle for a path, creating intermediate directories
     */
    private async getDirectoryForPath(path: string): Promise<{
        dir: FileSystemDirectoryHandle;
        fileName: string;
    }> {
        const root = await this.getRoot();
        const parts = path.split('/').filter(p => p);
        const fileName = parts.pop()!;

        let dir = root;
        for (const part of parts) {
            dir = await dir.getDirectoryHandle(part, { create: true });
        }

        return { dir, fileName };
    }

    async read(path: string): Promise<Uint8Array> {
        try {
            const { dir, fileName } = await this.getDirectoryForPath(path);
            const fileHandle = await dir.getFileHandle(fileName);
            const file = await fileHandle.getFile();
            const buffer = await file.arrayBuffer();
            return new Uint8Array(buffer);
        } catch (e) {
            if ((e as DOMException).name === 'NotFoundError') {
                throw new Error(`File not found: ${path}`);
            }
            throw e;
        }
    }

    async write(path: string, data: Uint8Array): Promise<void> {
        const { dir, fileName } = await this.getDirectoryForPath(path);
        const fileHandle = await dir.getFileHandle(fileName, { create: true });

        // Use synchronous access handle for better performance if available
        // @ts-expect-error - createSyncAccessHandle is not in all TS types yet
        if (typeof fileHandle.createSyncAccessHandle === 'function') {
            // Sync access (Web Worker only)
            // @ts-expect-error
            const accessHandle = await fileHandle.createSyncAccessHandle();
            try {
                accessHandle.truncate(0);
                accessHandle.write(data);
                accessHandle.flush();
            } finally {
                accessHandle.close();
            }
        } else {
            // Async writable stream (main thread compatible)
            const writable = await fileHandle.createWritable();
            try {
                // @ts-expect-error - TS has issues matching Uint8Array with FileSystemWriteChunkType in some envs
                await writable.write(data);
            } finally {
                await writable.close();
            }
        }
    }

    async delete(path: string): Promise<void> {
        try {
            const { dir, fileName } = await this.getDirectoryForPath(path);
            await dir.removeEntry(fileName);
        } catch (e) {
            if ((e as DOMException).name === 'NotFoundError') {
                // Already deleted, ignore
                return;
            }
            throw e;
        }
    }

    async exists(path: string): Promise<boolean> {
        try {
            const { dir, fileName } = await this.getDirectoryForPath(path);
            await dir.getFileHandle(fileName);
            return true;
        } catch {
            return false;
        }
    }

    async list(dir: string): Promise<string[]> {
        const files: string[] = [];

        try {
            const root = await this.getRoot();
            await this.listRecursive(root, dir, '', files);
        } catch {
            // Directory doesn't exist
        }

        return files;
    }

    async size(path: string): Promise<number> {
        const { dir, fileName } = await this.getDirectoryForPath(path);
        const fileHandle = await dir.getFileHandle(fileName);
        const file = await fileHandle.getFile();
        return file.size;
    }

    async listBlobs(): Promise<string[]> {
        return this.list('.store/blobs');
    }

    private async listRecursive(
        handle: FileSystemDirectoryHandle,
        prefix: string,
        currentPath: string,
        results: string[]
    ): Promise<void> {
        // Navigate to prefix directory if specified
        if (prefix && currentPath === '') {
            const parts = prefix.split('/').filter(p => p);
            let dir = handle;
            for (const part of parts) {
                try {
                    dir = await dir.getDirectoryHandle(part);
                } catch {
                    return; // Prefix doesn't exist
                }
            }
            handle = dir;
        }

        for await (const [name, entry] of (handle as any).entries()) {
            const fullPath = currentPath ? `${currentPath}/${name}` : name;

            if (entry.kind === 'file') {
                results.push(fullPath);
            } else if (entry.kind === 'directory') {
                await this.listRecursive(entry, '', fullPath, results);
            }
        }
    }

    /**
     * Clear all data for this prefix
     */
    async clear(): Promise<void> {
        try {
            const opfsRoot = await navigator.storage.getDirectory();
            await opfsRoot.removeEntry(this.prefix, { recursive: true });
            // Reset the root promise so it gets recreated
            this.rootPromise = null;
        } catch (e) {
            if ((e as DOMException).name === 'NotFoundError') {
                return;
            }
            throw e;
        }
    }

    /**
     * Check if OPFS is available in current environment
     */
    static isAvailable(): boolean {
        return typeof navigator !== 'undefined'
            && 'storage' in navigator
            && 'getDirectory' in navigator.storage;
    }
}
