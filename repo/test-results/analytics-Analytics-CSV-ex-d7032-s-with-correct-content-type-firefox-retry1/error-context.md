# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: analytics.spec.ts >> Analytics >> CSV export API responds with correct content-type
- Location: frontend/tests/e2e/analytics.spec.ts:37:7

# Error details

```
Error: page.goto: NS_ERROR_UNKNOWN_HOST
Call log:
  - navigating to "http://app:3000/", waiting until "load"

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - heading [level=1] [ref=e5]
  - paragraph
  - paragraph
```

# Test source

```ts
  1  | import { test, expect, Page } from '@playwright/test';
  2  | 
  3  | async function loginAs(page: Page, username: string, password: string) {
> 4  |   await page.goto('/');
     |              ^ Error: page.goto: NS_ERROR_UNKNOWN_HOST
  5  |   await page.fill('input[name="username"], input[type="text"]', username);
  6  |   await page.fill('input[name="password"], input[type="password"]', password);
  7  |   await page.click('button[type="submit"]');
  8  |   await page.waitForURL((url) => !url.pathname.includes('login'), { timeout: 10000 });
  9  | }
  10 | 
  11 | test.describe('Analytics', () => {
  12 |   test('finance manager can navigate to analytics page', async ({ page }) => {
  13 |     await loginAs(page, 'finance', 'Scholar2024!');
  14 |     await page.goto('/analytics');
  15 |     await page.waitForLoadState('networkidle');
  16 |     const body = await page.locator('body').textContent();
  17 |     expect(body?.length).toBeGreaterThan(0);
  18 |   });
  19 | 
  20 |   test('fund summary API responds', async ({ page }) => {
  21 |     await loginAs(page, 'finance', 'Scholar2024!');
  22 |     const resp = await page.request.get('/api/analytics/funds');
  23 |     expect(resp.status()).toBeLessThan(500);
  24 |     if (resp.status() === 200) {
  25 |       const json = await resp.json();
  26 |       expect(typeof json.total_expense).toBe('number');
  27 |       expect(typeof json.total_income).toBe('number');
  28 |     }
  29 |   });
  30 | 
  31 |   test('churn rate API responds', async ({ page }) => {
  32 |     await loginAs(page, 'finance', 'Scholar2024!');
  33 |     const resp = await page.request.get('/api/analytics/churn');
  34 |     expect(resp.status()).toBeLessThan(500);
  35 |   });
  36 | 
  37 |   test('CSV export API responds with correct content-type', async ({ page }) => {
  38 |     await loginAs(page, 'finance', 'Scholar2024!');
  39 |     const resp = await page.request.get('/api/analytics/export/csv?report_type=fund');
  40 |     expect(resp.status()).toBeLessThan(500);
  41 |     if (resp.status() === 200) {
  42 |       const ct = resp.headers()['content-type'] ?? '';
  43 |       expect(ct).toContain('csv');
  44 |     }
  45 |   });
  46 | 
  47 |   test('non-finance role is denied analytics access', async ({ page }) => {
  48 |     await loginAs(page, 'curator', 'Scholar2024!');
  49 |     const resp = await page.request.get('/api/analytics/fund-summary');
  50 |     expect([401, 403, 404]).toContain(resp.status());
  51 |   });
  52 | });
  53 | 
```