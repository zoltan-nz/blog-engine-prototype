import { expect, test } from '@playwright/test';

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:8080';

test.describe('Sites - UI basics', () => {
  test('home page shows "Create a new blog" button', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    await expect(page.getByRole('button', { name: 'Create a new blog' })).toBeVisible();
  });

  test('clicking the button opens a modal with a name field', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    await page.getByRole('button', { name: 'Create a new blog' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.getByLabel('Blog name')).toBeVisible();
  });

  test('footer shows Connected once the socket is open', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    await expect(page.getByTestId('footer')).toContainText('Connected');
  });
});

// One site flows through its whole lifecycle: create → preview → stop → destroy.
// Serial because scaffolding (create-astro + pnpm install) is the expensive step
// and every later test reuses it.
test.describe.serial('Sites - full lifecycle over WS', () => {
  const BLOG_NAME = `Lifecycle Test ${Date.now()}`;
  const SLUG_PATTERN = /lifecycle-test-\d+/;

  test('creating a blog shows Scaffolding, then settles Ready', async ({ page }) => {
    test.setTimeout(300_000);

    await page.goto(FRONTEND_URL);
    await page.getByRole('button', { name: 'Create a new blog' }).click();
    await page.getByLabel('Blog name').fill(BLOG_NAME);
    await page.getByRole('button', { name: 'Create', exact: true }).click();

    // Card appears immediately in Creating state (broadcast, not refetch).
    const card = page.locator('li', { hasText: BLOG_NAME });
    await expect(card).toBeVisible({ timeout: 10_000 });
    await expect(card).toContainText(SLUG_PATTERN);

    // Scaffold finishes → badge disappears, actions enabled.
    await expect(card.getByText('Scaffolding…')).toBeHidden({ timeout: 280_000 });
    await expect(card.getByRole('button', { name: /Start Preview/ })).toBeEnabled();
  });

  test('starting a preview shows the Live badge with a working URL', async ({ page }) => {
    test.setTimeout(120_000);

    await page.goto(FRONTEND_URL);
    const card = page.locator('li', { hasText: BLOG_NAME });
    await card.getByRole('button', { name: /Start Preview/ }).click();

    const live = card.getByRole('link', { name: 'Live' });
    await expect(live).toBeVisible({ timeout: 90_000 });

    const previewUrl = await live.getAttribute('href');
    expect(previewUrl).toBeTruthy();
    const response = await page.request.get(previewUrl!);
    expect(response.status()).toBe(200);
  });

  test('stopping the preview removes the Live badge', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    const card = page.locator('li', { hasText: BLOG_NAME });
    await card.getByRole('button', { name: 'Stop Preview' }).click();
    await expect(card.getByRole('link', { name: 'Live' })).toBeHidden({ timeout: 30_000 });
  });

  test('destroying the blog removes the card', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    const card = page.locator('li', { hasText: BLOG_NAME });

    page.on('dialog', (dialog) => dialog.accept());
    await card.getByRole('button', { name: 'Destroy' }).click();

    await expect(card).toBeHidden({ timeout: 30_000 });
  });
});
