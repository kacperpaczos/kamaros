import { describe, it, expect, beforeEach } from 'vitest';
import { MemoryAdapter } from '../src/MemoryAdapter';

describe('MemoryAdapter', () => {
    let adapter: MemoryAdapter;

    beforeEach(() => {
        adapter = new MemoryAdapter();
    });

    it('should write and read files', async () => {
        const data = new TextEncoder().encode('Hello World');
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
        await adapter.write('other/file3.txt', new Uint8Array([3]));

        const files = await adapter.list('dir');
        expect(files).toHaveLength(2);
        // list() returns relative filenames inside the directory
        expect(files).toContain('file1.txt');
        expect(files).toContain('file2.txt');
    });

    it('should delete files', async () => {
        await adapter.write('delete.me', new Uint8Array([0]));
        expect(await adapter.exists('delete.me')).toBe(true);

        await adapter.delete('delete.me');
        expect(await adapter.exists('delete.me')).toBe(false);

        // Deleting non-existent file should not throw (usually, but let's check impl)
        // Implementation: this.storage.delete(path) returns boolean but returns void. 
        // JavaScript Map.delete returns true/false. Implementation ignores it.
        await adapter.delete('nonexistent.me');
    });
});
