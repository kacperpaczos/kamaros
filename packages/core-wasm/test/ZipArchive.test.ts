
import { describe, it, expect, beforeEach } from 'vitest';
import { JCFManager } from '../src/JCFManager';
import { MemoryAdapter } from '../src/MemoryAdapter';

describe('ZipArchive Integration', () => {
    let adapter: MemoryAdapter;
    let manager: JCFManager;

    beforeEach(async () => {
        adapter = new MemoryAdapter();
        manager = await JCFManager.create(adapter);
    });

    it('should export and import a project via ZIP', async () => {
        // 1. Create project
        const projectName = 'ZipTestProject';
        await manager.createProject(projectName);

        // 2. Add files
        const content1 = 'Hello ZIP';
        await manager.addFile('hello.txt', new TextEncoder().encode(content1));
        await manager.saveCheckpoint('v1');

        const content2 = 'Updated content';
        await manager.addFile('hello.txt', new TextEncoder().encode(content2));
        await manager.saveCheckpoint('v2');

        // 3. Export ZIP
        const zipData = await manager.exportZip();
        expect(zipData).toBeDefined();
        expect(zipData.length).toBeGreaterThan(0);

        // 4. Import ZIP into NEW manager
        const newAdapter = new MemoryAdapter();
        const newManager = await JCFManager.create(newAdapter);

        const importResult = await newManager.importZip(zipData);

        // 5. Verify Import Result
        expect(importResult.projectName).toBe(projectName);
        expect(importResult.filesImported).toBeGreaterThan(0);

        // 6. Verify Project State
        const info = newManager.getProjectInfo();
        expect(info).toBeDefined();
        expect(info?.name).toBe(projectName);
        expect(info?.versionCount).toBe(2);

        // Verify working dir file
        const fileData = newManager.getFile('hello.txt');
        expect(fileData).toBeDefined();
        expect(new TextDecoder().decode(fileData)).toBe(content2);

        // Verify history (by restoring v1)
        const manifest = newManager.getManifest();
        const v1Id = manifest?.versionHistory[0].id;
        expect(v1Id).toBeDefined();

        if (v1Id) {
            await newManager.restoreVersion(v1Id);

            const v1Data = newManager.getFile('hello.txt');
            expect(v1Data).toBeDefined();
            expect(new TextDecoder().decode(v1Data)).toBe(content1);
        }
    });
});
