import { expect, test } from '@playwright/test';

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3000';
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

test.describe('Sites - API', () => {
  test('GET /sites returns empty list initially', async ({ request }) => {
    const response = await request.get(`${BACKEND_URL}/sites`);
    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body).toHaveProperty('data');
    expect(body).toHaveProperty('meta');
    expect(Array.isArray(body.data)).toBe(true);
  });

  test('POST /sites creates a site and returns it with a git URL', async ({ request }) => {
    test.setTimeout(120_000); // npm create astro takes time

    const response = await request.post(`${BACKEND_URL}/sites`, {
      data: { name: 'API Test Blog', slug: 'api-test-blog' },
    });
    expect(response.status()).toBe(201);

    const body = await response.json();
    expect(body).toHaveProperty('data');
    expect(body).toHaveProperty('meta');
    expect(body.data).toMatchObject({ name: 'API Test Blog', slug: 'api-test-blog' });
    expect(body.data).toHaveProperty('gitUrl');
  });

  test('created site appears in GET /sites list', async ({ request }) => {
    test.setTimeout(120_000);

    await request.post(`${BACKEND_URL}/sites`, {
      data: { name: 'List Test Blog', slug: 'list-test-blog' },
    });

    const response = await request.get(`${BACKEND_URL}/sites`);
    const body = await response.json();
    const site = body.data.find((s: { slug: string }) => s.slug === 'list-test-blog');
    expect(site).toBeDefined();
    expect(site.name).toBe('List Test Blog');
  });
});

test.describe('Sites - UI', () => {
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

  test('creating a blog via the UI shows the new site in the list', async ({ page }) => {
    test.setTimeout(300_000);

    await page.goto(FRONTEND_URL);
    await page.getByRole('button', { name: 'Create a new blog' }).click();
    await page.getByLabel('Blog name').fill('UI Test Blog');
    await page.getByRole('button', { name: 'Create', exact: true }).click();

    await expect(page.getByText('UI Test Blog')).toBeVisible({ timeout: 280_000 });
  });
});
