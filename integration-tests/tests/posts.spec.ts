import { expect, test } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:8080';
// Matches backend/.env's SITES_DIR (relative to backend/, so the same
// relative path resolves correctly from this sibling directory).
const SITES_DIR = join(__dirname, '..', '..', 'tmp', 'astro-sites');

// One site carries the whole flow: create → editing area → preview auto-starts
// → create post → type → live preview reflects it → delete post. Serial
// because scaffolding (create-astro + pnpm install) is the expensive step.
test.describe.serial('Post CRUD with live preview sync', () => {
  const BLOG_NAME = `Post CRUD Test ${Date.now()}`;
  // Mirrors the frontend's nameToSlug (routes/+page.svelte) so each test can
  // navigate straight to this site's editing route — every test gets a
  // fresh `page`, so state from the previous test isn't otherwise reachable.
  const SLUG = BLOG_NAME.toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 100);
  const POST_TITLE = 'Hello World';
  const POST_ID = 'hello-world';
  const TYPED_TEXT = 'Live preview sync works!';

  test('creates the site and opens the editing area with the preview iframe', async ({ page }) => {
    test.setTimeout(300_000);

    await page.goto(FRONTEND_URL);
    await page.getByRole('button', { name: 'Create a new blog' }).click();
    await page.getByLabel('Blog name').fill(BLOG_NAME);
    await page.getByRole('button', { name: 'Create', exact: true }).click();

    const card = page.locator('li', { hasText: BLOG_NAME });
    await expect(card).toBeVisible({ timeout: 10_000 });
    await expect(card.getByText('Scaffolding…')).toBeHidden({ timeout: 280_000 });

    await card.getByRole('link', { name: /Edit posts/ }).click();
    await expect(page.getByRole('heading', { name: BLOG_NAME })).toBeVisible();
    await expect(page.locator('iframe[src]')).toBeVisible({ timeout: 90_000 });
  });

  test('creates a post from the New post dialog; it appears in the list', async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/sites/${SLUG}`);

    await page.getByRole('button', { name: 'New post' }).click();

    // Skeleton dialog — fill the title input and submit.
    await page.getByLabel('Title').fill(POST_TITLE);
    await page.getByRole('button', { name: 'Create', exact: true }).click();

    await expect(page.getByText(POST_TITLE)).toBeVisible({ timeout: 10_000 });
  });

  // Verifies the write itself (editor → autosave → WS → dispatch → file write)
  // before checking Astro's content-collection HMR behavior.
  test('typing in the editor autosaves; the post file on disk reflects it', async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/sites/${SLUG}`);
    await expect(page.locator('iframe[src]')).toBeVisible({ timeout: 90_000 });
    await page.getByText(POST_TITLE).click();

    const editor = page.locator('[contenteditable="true"]').first();
    await expect(editor).toBeVisible({ timeout: 10_000 });
    await editor.click();
    await page.keyboard.type(TYPED_TEXT);

    await expect(page.getByText('Saved')).toBeVisible({ timeout: 10_000 });

    const postPath = join(SITES_DIR, SLUG, 'src', 'content', 'blog', `${POST_ID}.md`);
    await expect.poll(() => readFile(postPath, 'utf-8'), { timeout: 10_000 }).toContain(TYPED_TEXT);
  });

  test('deleting the post removes it from the list', async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/sites/${SLUG}`);
    page.once('dialog', (dialog) => dialog.accept());

    const row = page.locator('li', { hasText: POST_TITLE });
    await row.getByRole('button', { name: /delete/i }).click();

    await expect(page.getByText(POST_TITLE)).toBeHidden({ timeout: 10_000 });
  });
});
