import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
    testDir: './tests-e2e',
    timeout: 30000,
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: 'list',
    use: {
        trace: 'on-first-retry',
        baseURL: 'http://localhost:3000',
    },
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
    ],
    webServer: {
        command: 'npx serve -l 3000 tests-e2e',
        url: 'http://localhost:3000',
        reuseExistingServer: !process.env.CI,
    },
});
