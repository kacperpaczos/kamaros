/**
 * Tests for Storage Adapters
 * 
 * Each test focuses on a single aspect of adapter functionality.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { MemoryAdapter } from '../adapters/MemoryAdapter';

describe('MemoryAdapter', () => {
    let adapter: MemoryAdapter;

    beforeEach(() => {
        adapter = new MemoryAdapter();
    });

    // =========================================================================
    // HAPPY PATH TESTS
    // =========================================================================

    /** Test: Can write and read data */
    it('should write and read data', async () => {
        const data = new Uint8Array([1, 2, 3, 4, 5]);

        await adapter.write('test.bin', data);
        const retrieved = await adapter.read('test.bin');

        expect(retrieved).toEqual(data);
    });

    /** Test: Can check if file exists */
    it('should check file existence', async () => {
        await adapter.write('exists.txt', new Uint8Array([1]));

        expect(await adapter.exists('exists.txt')).toBe(true);
        expect(await adapter.exists('notexists.txt')).toBe(false);
    });

    /** Test: Can delete file */
    it('should delete file', async () => {
        await adapter.write('test.txt', new Uint8Array([1]));

        await adapter.delete('test.txt');

        expect(await adapter.exists('test.txt')).toBe(false);
    });

    /** Test: Can list files in directory */
    it('should list files in directory', async () => {
        await adapter.write('dir/file1.txt', new Uint8Array([1]));
        await adapter.write('dir/file2.txt', new Uint8Array([2]));
        await adapter.write('other/file3.txt', new Uint8Array([3]));

        const files = await adapter.list('dir');

        expect(files).toHaveLength(2);
        expect(files).toContain('file1.txt');
        expect(files).toContain('file2.txt');
    });

    /** Test: Clear removes all data */
    it('should clear all data', async () => {
        await adapter.write('file1.txt', new Uint8Array([1]));
        await adapter.write('file2.txt', new Uint8Array([2]));

        adapter.clear();

        expect(await adapter.exists('file1.txt')).toBe(false);
        expect(await adapter.exists('file2.txt')).toBe(false);
    });

    // =========================================================================
    // EDGE CASE TESTS
    // =========================================================================

    /** Edge case: Read non-existent file throws */
    it('should throw when reading non-existent file', async () => {
        await expect(adapter.read('nonexistent.txt')).rejects.toThrow();
    });

    /** Edge case: Delete non-existent file does not throw */
    it('should not throw when deleting non-existent file', async () => {
        await expect(adapter.delete('nonexistent.txt')).resolves.not.toThrow();
    });

    /** Edge case: Empty file */
    it('should handle empty file', async () => {
        await adapter.write('empty.txt', new Uint8Array(0));

        const data = await adapter.read('empty.txt');
        expect(data.length).toBe(0);
    });

    /** Edge case: Large file */
    it('should handle large file', async () => {
        const largeData = new Uint8Array(1024 * 1024); // 1MB
        for (let i = 0; i < largeData.length; i++) {
            largeData[i] = i % 256;
        }

        await adapter.write('large.bin', largeData);
        const retrieved = await adapter.read('large.bin');

        expect(retrieved.length).toBe(largeData.length);
    });

    /** Edge case: Overwrite existing file */
    it('should overwrite existing file', async () => {
        await adapter.write('test.txt', new TextEncoder().encode('Version 1'));
        await adapter.write('test.txt', new TextEncoder().encode('Version 2'));

        const data = await adapter.read('test.txt');
        expect(new TextDecoder().decode(data)).toBe('Version 2');
    });

    /** Edge case: Nested directories */
    it('should handle nested directories', async () => {
        await adapter.write('a/b/c/deep.txt', new Uint8Array([1]));

        expect(await adapter.exists('a/b/c/deep.txt')).toBe(true);
    });

    /** Edge case: List empty directory */
    it('should return empty array for non-existent directory', async () => {
        const files = await adapter.list('nonexistent');

        expect(files).toEqual([]);
    });

    /** Edge case: GetAllPaths helper */
    it('should return all paths with getAllPaths', async () => {
        await adapter.write('file1.txt', new Uint8Array([1]));
        await adapter.write('dir/file2.txt', new Uint8Array([2]));

        const paths = adapter.getAllPaths();

        expect(paths).toHaveLength(2);
    });
});
