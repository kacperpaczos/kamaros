/**
 * Tests for saveCheckpoint integration
 * 
 * Note: These tests require the WASM module to be built and available.
 * If running in Node.js without WASM support, these might need to be skipped.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { JCFManager } from '../api/JCFManager';
import { MemoryAdapter } from '../adapters/MemoryAdapter';
import { initWasm } from '../wasm';

describe('JCFManager - Checkpoints', () => {
    let adapter: MemoryAdapter;
    let manager: JCFManager;

    beforeAll(async () => {
        // Initialize WASM
        try {
            await initWasm();
        } catch (e) {
            console.warn('Skipping WASM tests - module not found or environment not supported');
            // This allows tests to run even if WASM fails (e.g. CI without build)
            // But we should likely fail if we expect it to work.
            // For now, let's let it throw if it fails, to see the error.
        }
    });

    it('should create a checkpoint and update version history', async () => {
        adapter = new MemoryAdapter();
        manager = await JCFManager.create(adapter);

        // 1. Create Project
        await manager.createProject('CheckpointTest');

        // 2. Add a file
        const content = new TextEncoder().encode('Version 1 Content');
        manager.addFile('main.txt', content);

        // 3. Save Checkpoint
        const versionId = await manager.saveCheckpoint('Initial commit', { author: 'Tester' });

        expect(versionId).toBeDefined();
        expect(versionId.length).toBeGreaterThan(0);

        // 4. Verify Manifest
        const manifest = manager.getManifest();
        expect(manifest).not.toBeNull();
        if (manifest) {
            expect(manifest.versionHistory.length).toBe(1);
            expect(manifest.versionHistory[0].id).toBe(versionId);
            expect(manifest.versionHistory[0].message).toBe('Initial commit');
            expect(manifest.versionHistory[0].fileStates['main.txt']).toBeDefined();
            expect(manifest.refs.head).toBe(versionId);
        }
    });

    it('should detect file modifications in checkpoints', async () => {
        adapter = new MemoryAdapter();
        manager = await JCFManager.create(adapter);
        await manager.createProject('ModTest');

        // V1
        manager.addFile('data.json', new TextEncoder().encode('{"v": 1}'));
        const v1 = await manager.saveCheckpoint('Version 1');

        // V2 - Modify file
        manager.addFile('data.json', new TextEncoder().encode('{"v": 2}'));
        const v2 = await manager.saveCheckpoint('Version 2');

        expect(v1).not.toBe(v2);

        const manifest = manager.getManifest();
        expect(manifest!.versionHistory.length).toBe(2);

        // Check V2 state
        const v2State = manifest!.versionHistory[1];
        const v1State = manifest!.versionHistory[0];

        expect(v2State.fileStates['data.json'].hash).not.toBe(v1State.fileStates['data.json'].hash);
    });
});
