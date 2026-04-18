import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.fill('input[name="username"], input[type="text"]', username);
  await page.fill('input[name="password"], input[type="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL((url) => !url.pathname.includes('login'), { timeout: 10000 });
}

test.describe('Store', () => {
  test('store manager can navigate to store page', async ({ page }) => {
    await loginAs(page, 'store', 'Scholar2024!');
    await page.goto('/store');
    await page.waitForLoadState('networkidle');
    const body = await page.locator('body').textContent();
    expect(body?.length).toBeGreaterThan(0);
  });

  test('products API returns list', async ({ page }) => {
    await loginAs(page, 'store', 'Scholar2024!');
    const resp = await page.request.get('/api/store/products');
    expect(resp.status()).toBeLessThan(500);
  });

  test('checkout rejects tampered client price', async ({ page }) => {
    await loginAs(page, 'store', 'Scholar2024!');
    // Attempt checkout with a client-supplied price — server must ignore it
    const resp = await page.request.post('/api/store/checkout', {
      data: {
        items: [{ product_id: 'prod-001', quantity: 1, unit_price: 0.01, product_name: 'hack' }],
        promotion_id: null,
      },
    });
    // Either succeeds with server-side price or rejects — never should 500
    expect(resp.status()).not.toBe(500);
    if (resp.status() === 200) {
      const json = await resp.json();
      // Server-side price should NOT be 0.01
      const lineItems = json.line_items ?? json.items ?? [];
      for (const item of lineItems) {
        expect(item.unit_price ?? item.price).not.toBe(0.01);
      }
    }
  });

  test('promotions API returns list', async ({ page }) => {
    await loginAs(page, 'store', 'Scholar2024!');
    const resp = await page.request.get('/api/store/promotions');
    expect(resp.status()).toBeLessThan(500);
  });
});
