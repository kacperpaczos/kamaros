/**
 * Node.js storage adapter using fs/promises
 */

import { promises as fs } from 'fs';
import * as path from 'path';
import type { StorageAdapter } from '@kamaros/core-wasm';

export class NodeAdapter implements StorageAdapter {
    constructor(private basePath: string) { }

    async read(filePath: string): Promise<Uint8Array> {
        const fullPath = path.join(this.basePath, filePath);
        const buffer = await fs.readFile(fullPath);
        return new Uint8Array(buffer);
    }

    async write(filePath: string, data: Uint8Array): Promise<void> {
        const fullPath = path.join(this.basePath, filePath);
        await fs.mkdir(path.dirname(fullPath), { recursive: true });
        await fs.writeFile(fullPath, data);
    }

    async delete(filePath: string): Promise<void> {
        const fullPath = path.join(this.basePath, filePath);
        await fs.unlink(fullPath);
    }

    async exists(filePath: string): Promise<boolean> {
        try {
            await fs.access(path.join(this.basePath, filePath));
            return true;
        } catch {
            return false;
        }
    }

    async list(dir: string): Promise<string[]> {
        const fullPath = path.join(this.basePath, dir);
        try {
            return await fs.readdir(fullPath);
        } catch {
            return [];
        }
    }
}
