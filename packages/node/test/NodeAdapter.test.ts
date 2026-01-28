import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { NodeAdapter } from '../src/NodeAdapter';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as os from 'os';

describe('NodeAdapter', () => {
    let adapter: NodeAdapter;
    let tempDir: string;

    beforeEach(async () => {
        tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'kamaros-test-'));
        adapter = new NodeAdapter(tempDir);
    });

    afterEach(async () => {
        await fs.rm(tempDir, { recursive: true, force: true });
    });

    it('should write and read files', async () => {
        const data = new TextEncoder().encode('Hello Node');
        await adapter.write('test.txt', data);

        const readData = await adapter.read('test.txt');
        expect(readData).toEqual(data);
    });

    it('should throw error for non-existent files', async () => {
        await expect(adapter.read('nonexistent.txt')).rejects.toThrow();
    });

    it('should list files in directory', async () => {
        await adapter.write('dir/file1.txt', new Uint8Array([1]));
        await adapter.write('dir/file2.txt', new Uint8Array([2]));

        // Ensure directory exists for list
        const files = await adapter.list('dir');
        expect(files).toHaveLength(2);
        expect(files).toContain('file1.txt');
        expect(files).toContain('file2.txt');
    });

    it('should return empty list for non-existent directory', async () => {
        const files = await adapter.list('nonexistent');
        expect(files).toEqual([]);
    });

    it('should delete files', async () => {
        await adapter.write('delete.me', new Uint8Array([0]));
        expect(await adapter.exists('delete.me')).toBe(true);

        await adapter.delete('delete.me');
        expect(await adapter.exists('delete.me')).toBe(false);
    });
});
