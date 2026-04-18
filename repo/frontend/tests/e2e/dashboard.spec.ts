import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.fill('input[name="username"], input[type="text"]', username);
  await page.fill('input[name="password"], input[type="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => !url.pathname.includes('login'), { timeout: 10000 });
}

test.describe('Dashboard', () => {
  test('admin sees dashboard after login', async ({ page }) => {
    await loginAs(page, 'admin', 'ScholarAdmin2024!');
    await expect(page.locator('body')).not.toBeEmpty();
    await expect(page).not.toHaveURL(/login/);
  });

  test('health endpoint responds 200', async ({ page }) => {
    const resp = await page.goto('/api/healthz');
    expect(resp?.status()).toBe(200);
  });

  test('unauthenticated access to protected API endpoint is rejected', async ({ page }) => {
    // API endpoints enforce authentication server-side even if the SPA renders
    const resp = await page.request.get('/api/auth/me');
    expect([401, 403]).toContain(resp.status());
  });
});
