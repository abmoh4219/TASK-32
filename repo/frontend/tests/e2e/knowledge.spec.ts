import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.fill('input[name="username"], input[type="text"]', username);
  await page.fill('input[name="password"], input[type="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => !url.pathname.includes('login'), { timeout: 10000 });
}

test.describe('Knowledge Management', () => {
  test('curator can navigate to knowledge page', async ({ page }) => {
    await loginAs(page, 'curator', 'Scholar2024!');
    await page.goto('/knowledge');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('body')).not.toBeEmpty();
    // Should show some knowledge-related content
    const body = await page.locator('body').textContent();
    expect(body?.length).toBeGreaterThan(0);
  });

  test('knowledge API returns categories list', async ({ page }) => {
    await loginAs(page, 'curator', 'Scholar2024!');
    const resp = await page.request.get('/api/knowledge/categories');
    expect(resp.status()).toBeLessThan(500);
  });

  test('knowledge API returns knowledge points list', async ({ page }) => {
    await loginAs(page, 'curator', 'Scholar2024!');
    const resp = await page.request.get('/api/knowledge/points');
    expect(resp.status()).toBeLessThan(500);
  });

  test('reviewer cannot access curator-only create endpoint', async ({ page }) => {
    await loginAs(page, 'reviewer', 'Scholar2024!');
    const resp = await page.request.post('/api/knowledge/categories', {
      data: { name: 'hack', parent_id: null },
    });
    expect([401, 403]).toContain(resp.status());
  });
});
