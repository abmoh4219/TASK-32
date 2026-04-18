import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('login page loads', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('input[name="username"], input[type="text"]').first()).toBeVisible();
  });

  test('admin login succeeds and reaches dashboard', async ({ page }) => {
    await page.goto('/');
    await page.fill('input[name="username"], input[type="text"]', 'admin');
    await page.fill('input[name="password"], input[type="password"]', 'ScholarAdmin2024!');
    await page.click('button[type="submit"]');
    await expect(page).not.toHaveURL(/login/, { timeout: 10000 });
  });

  test('wrong credentials show error', async ({ page }) => {
    await page.goto('/');
    await page.fill('input[name="username"], input[type="text"]', 'admin');
    await page.fill('input[name="password"], input[type="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');
    const body = page.locator('body');
    await expect(body).toContainText(/invalid|incorrect|failed|error|authentication required/i, { timeout: 5000 });
  });

  test('curator login succeeds', async ({ page }) => {
    await page.goto('/');
    await page.fill('input[name="username"], input[type="text"]', 'curator');
    await page.fill('input[name="password"], input[type="password"]', 'Scholar2024!');
    await page.click('button[type="submit"]');
    await expect(page).not.toHaveURL(/login/, { timeout: 10000 });
  });

  test('finance login succeeds', async ({ page }) => {
    await page.goto('/');
    await page.fill('input[name="username"], input[type="text"]', 'finance');
    await page.fill('input[name="password"], input[type="password"]', 'Scholar2024!');
    await page.click('button[type="submit"]');
    await expect(page).not.toHaveURL(/login/, { timeout: 10000 });
  });
});
