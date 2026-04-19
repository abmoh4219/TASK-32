import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './frontend/tests/e2e',
  fullyParallel: false,
  retries: 1,
  workers: 1,
  reporter: 'list',
  timeout: 30000,
  globalTimeout: 300000,
  use: {
    baseURL: process.env.BASE_URL ?? 'http://localhost:3000',
    headless: true,
    ignoreHTTPSErrors: true,
    trace: 'on-first-retry',
    actionTimeout: 15000,
    navigationTimeout: 15000,
  },
  projects: [
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
  ],
});
