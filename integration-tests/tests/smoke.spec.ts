import { expect, test } from '@playwright/test';

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:5173';
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

test.describe('Smoke Tests - All Services Respond', () => {
  test('frontend is accessible', async ({ page }) => {
    const response = await page.goto(FRONTEND_URL);
    expect(response?.status()).toBe(200);
  });

  test('backend healthz endpoint returns envelope', async ({ request }) => {
    const response = await request.get(`${BACKEND_URL}/healthz`);
    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body).toHaveProperty('data');
    expect(body).toHaveProperty('meta');
    expect(body.data).toHaveProperty('status', 'healthy');
  });
});

test.describe('Frontend can connect to the backend', () => {
  test('frontend footer contains the backend URL', async ({ page }) => {
    await page.goto(FRONTEND_URL);

    const footer = page.getByTestId('footer');

    await expect(footer).toContainText(BACKEND_URL);
  });

  test('frontend shows Connected status', async ({ page }) => {
    await page.goto(FRONTEND_URL);

    const footer = page.getByTestId('footer');

    // Wait for the connection check to complete (shows "Connected" when successful)
    await expect(footer).toContainText('Connected', { timeout: 10000 });
  });
});
