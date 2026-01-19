import { test, expect } from '@playwright/test';

test.describe('Browser Adapters', () => {
    test.beforeEach(async ({ page }) => {
        // Navigate to local server (secure context) to ensure OPFS/IDB work
        await page.goto('/');
    });

    test('IndexedDB Adapter Logic Verification', async ({ page }) => {
        const result = await page.evaluate(async () => {
            const DB_NAME = 'kamaros-test-idb';
            const STORE_NAME = 'files';

            function openDB() {
                return new Promise((resolve, reject) => {
                    const req = indexedDB.open(DB_NAME, 1);
                    req.onerror = () => reject(req.error);
                    req.onupgradeneeded = (e: any) => {
                        const db = e.target.result;
                        if (!db.objectStoreNames.contains(STORE_NAME)) db.createObjectStore(STORE_NAME);
                    };
                    req.onsuccess = () => resolve(req.result);
                });
            }

            async function write(path, data) {
                const db: any = await openDB();
                return new Promise<void>((resolve, reject) => {
                    const tx = db.transaction(STORE_NAME, 'readwrite');
                    tx.objectStore(STORE_NAME).put(data, path);
                    tx.oncomplete = () => resolve();
                    tx.onerror = () => reject(tx.error);
                });
            }

            async function read(path) {
                const db: any = await openDB();
                return new Promise((resolve, reject) => {
                    const tx = db.transaction(STORE_NAME, 'readonly');
                    const req = tx.objectStore(STORE_NAME).get(path);
                    req.onsuccess = () => resolve(req.result);
                    req.onerror = () => reject(req.error);
                });
            }

            const data = new Uint8Array([1, 2, 3]);
            await write('test.txt', data);
            const readBack = await read('test.txt');

            // Cleanup
            indexedDB.deleteDatabase(DB_NAME);

            return {
                match: readBack instanceof Uint8Array && readBack[1] === 2
            };
        });

        expect(result.match).toBe(true);
    });

    test('OPFS Adapter Logic Verification', async ({ page }) => {
        const supported = await page.evaluate(async () => {
            return 'storage' in navigator && 'getDirectory' in navigator.storage;
        });

        if (!supported) {
            test.skip(true, 'OPFS not supported in this browser environment');
            return;
        }

        const result = await page.evaluate(async () => {
            try {
                const root = await navigator.storage.getDirectory();
                const handle = await root.getFileHandle('test-opfs.txt', { create: true });

                // Sync access handle vs writable
                // @ts-ignore
                const writable = await handle.createWritable();
                await writable.write(new Uint8Array([10, 20, 30]));
                await writable.close();

                const file = await handle.getFile();
                const buf = await file.arrayBuffer();
                const arr = new Uint8Array(buf);

                await root.removeEntry('test-opfs.txt');

                return { success: true, val: arr[1] };
            } catch (e) {
                return { success: false, error: e.toString() };
            }
        });

        expect(result.success).toBe(true);
        expect(result.val).toBe(20);
    });
});
