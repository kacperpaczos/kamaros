/**
 * E2E Tests for Kamaros Browser OPFS Adapter
 * 
 * Tests the full workflow: project creation, file operations, snapshots, restore.
 */
import { test, expect } from '@playwright/test';

test.describe('Kamaros OPFS Workflow', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        // Wait for WASM to initialize
        await page.waitForFunction(() => (window as any).kamaros !== undefined, { timeout: 10000 });
    });

    test('creates a new project', async ({ page }) => {
        // Find and click "Create Project" button
        const createBtn = page.getByRole('button', { name: /create|new/i });
        await createBtn.click();

        // Fill project name
        await page.fill('[name="projectName"]', 'TestProject');
        await page.click('button[type="submit"]');

        // Verify project appears
        await expect(page.getByText('TestProject')).toBeVisible();
    });

    test('adds file and creates checkpoint', async ({ page }) => {
        // Assuming project already exists or create it first
        await page.getByRole('button', { name: /create|new/i }).click();
        await page.fill('[name="projectName"]', 'TestProject');
        await page.click('button[type="submit"]');

        // Add a file (drag & drop or button)
        // This is a placeholder - actual implementation depends on UI
        const fileInput = page.locator('input[type="file"]');
        if (await fileInput.count() > 0) {
            await fileInput.setInputFiles({
                name: 'test.txt',
                mimeType: 'text/plain',
                buffer: Buffer.from('Hello Kamaros!'),
            });
        }

        // Create checkpoint
        const saveBtn = page.getByRole('button', { name: /save|checkpoint|commit/i });
        if (await saveBtn.count() > 0) {
            await saveBtn.click();
            await page.fill('[name="message"]', 'Initial commit');
            await page.click('button[type="submit"]');
        }

        // Verify version appears in history
        await expect(page.getByText(/v1|initial/i)).toBeVisible({ timeout: 5000 });
    });

    test('handles errors gracefully', async ({ page }) => {
        // Try to restore non-existent version
        // This tests the structured error handling
        const result = await page.evaluate(async () => {
            try {
                const kamaros = (window as any).kamaros;
                if (!kamaros) return { error: 'Kamaros not loaded' };

                // Attempt invalid operation
                await kamaros.restoreVersion('invalid-version-id');
                return { error: 'Expected error but succeeded' };
            } catch (e: any) {
                return {
                    code: e.code || 'UNKNOWN',
                    message: e.message || String(e)
                };
            }
        });

        // Verify structured error
        expect(result.code).toBe('NOT_FOUND');
    });
});
