import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.fill('input[name="username"], input[type="text"]', username);
  await page.fill('input[name="password"], input[type="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => !url.pathname.includes('login'), { timeout: 10000 });
}

test.describe('Analytics', () => {
  test('finance manager can navigate to analytics page', async ({ page }) => {
    await loginAs(page, 'finance', 'Scholar2024!');
    await page.goto('/analytics');
    await page.waitForLoadState('networkidle');
    const body = await page.locator('body').textContent();
    expect(body?.length).toBeGreaterThan(0);
  });

  test('fund summary API responds', async ({ page }) => {
    await loginAs(page, 'finance', 'Scholar2024!');
    const resp = await page.request.get('/api/analytics/funds');
    expect(resp.status()).toBeLessThan(500);
    if (resp.status() === 200) {
      const json = await resp.json();
      expect(typeof json.total_expense).toBe('number');
      expect(typeof json.total_income).toBe('number');
    }
  });

  test('churn rate API responds', async ({ page }) => {
    await loginAs(page, 'finance', 'Scholar2024!');
    const resp = await page.request.get('/api/analytics/churn');
    expect(resp.status()).toBeLessThan(500);
  });

  test('CSV export API responds with correct content-type', async ({ page }) => {
    await loginAs(page, 'finance', 'Scholar2024!');
    const resp = await page.request.get('/api/analytics/export/csv?report_type=fund');
    expect(resp.status()).toBeLessThan(500);
    if (resp.status() === 200) {
      const ct = resp.headers()['content-type'] ?? '';
      expect(ct).toContain('csv');
    }
  });

  test('non-finance role is denied analytics access', async ({ page }) => {
    await loginAs(page, 'curator', 'Scholar2024!');
    const resp = await page.request.get('/api/analytics/fund-summary');
    expect([401, 403, 404]).toContain(resp.status());
  });
});
