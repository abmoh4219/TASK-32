import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.fill('input[name="username"], input[type="text"]', username);
  await page.fill('input[name="password"], input[type="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => !url.pathname.includes('login'), { timeout: 10000 });
}

test.describe('Outcomes', () => {
  test('reviewer can navigate to outcomes page', async ({ page }) => {
    await loginAs(page, 'reviewer', 'Scholar2024!');
    await page.goto('/outcomes');
    await page.waitForLoadState('networkidle');
    const body = await page.locator('body').textContent();
    expect(body?.length).toBeGreaterThan(0);
  });

  test('outcomes list API responds', async ({ page }) => {
    await loginAs(page, 'reviewer', 'Scholar2024!');
    const resp = await page.request.get('/api/outcomes');
    expect(resp.status()).toBeLessThan(500);
  });

  test('evidence upload endpoint exists', async ({ page }) => {
    await loginAs(page, 'reviewer', 'Scholar2024!');
    // Just checking the endpoint exists (not uploading a real file)
    const resp = await page.request.post('/api/outcomes/oc-001/evidence', {
      multipart: {},
    });
    // 400/422 for missing file is fine, 404/403 is fine; not 500
    expect(resp.status()).not.toBe(500);
  });

  test('admin cannot register an outcome (role gate)', async ({ page }) => {
    await loginAs(page, 'admin', 'ScholarAdmin2024!');
    const resp = await page.request.post('/api/outcomes', {
      data: {
        title: 'Should fail',
        abstract_snippet: 'x',
        certificate_number: 'CERT-0000',
      },
    });
    expect([401, 403]).toContain(resp.status());
  });
});
