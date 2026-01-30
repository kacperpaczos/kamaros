import { describe, it, expect, beforeEach, vi } from 'vitest';
import { JCFManager } from '../src/JCFManager';
import { MemoryAdapter } from '../src/MemoryAdapter';
import { initWasm } from '../src/wasm';

describe('JCFManager Integration (Encryption & GC)', () => {
    let adapter: MemoryAdapter;
    let manager: JCFManager;

    vi.setConfig({ testTimeout: 60000 });

    beforeEach(async () => {
        // Ensure WASM is initialized
        await initWasm();
        adapter = new MemoryAdapter();
        manager = await JCFManager.create(adapter);
    });

    it('should create a project and save a checkpoint', async () => {
        await manager.createProject('TestProject');
        await manager.addFile('test.txt', new TextEncoder().encode('hello world'));
        const versionId = await manager.saveCheckpoint('Initial commit');

        expect(versionId).toBeDefined();
        const info = manager.getProjectInfo();
        expect(info?.versionCount).toBe(1);
    });

    it('should support encryption and PBKDF2 key derivation', async () => {
        await manager.createProject('EncryptedProject');

        // Derive key
        const salt = new Uint8Array(16).fill(1);
        const key = await manager.deriveKey('password123', salt);
        expect(key.length).toBe(32);

        // Add file and save with encryption
        await manager.addFile('secret.txt', new TextEncoder().encode('top secret content'));
        const versionId = await manager.saveCheckpoint('Encrypted commit', { encryptionKey: key });

        expect(versionId).toBeDefined();

        // Verify that the blob is in storage
        const paths = adapter.getAllPaths();
        const blobPath = paths.find(p => p.startsWith('.store/blobs/'));
        expect(blobPath).toBeDefined();

        // Verify that we can restore (it should use the key from manager state)
        await manager.restoreVersion(versionId);
        const content = manager.getFile('secret.txt');
        expect(new TextDecoder().decode(content)).toBe('top secret content');
    });

    it('should run garbage collection', async () => {
        await manager.createProject('GcProject');

        // Save version 1
        await manager.addFile('v1.txt', new TextEncoder().encode('v1'));
        const v1 = await manager.saveCheckpoint('v1');

        // Save version 2 (modifying file)
        await manager.addFile('v1.txt', new TextEncoder().encode('v2'));
        const v2 = await manager.saveCheckpoint('v2');

        const preGc = await manager.gc();
        expect(preGc.blobsChecked).toBeGreaterThan(0);

        // Currently all versions are kept, so blobsDeleted should be 0 unless we have orphans.
        // Let's assume GC works.
        expect(preGc.blobsDeleted).toBe(0);
    });
});
