import { rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

// Wipe the scaffold workspace before the run so each `mise run test` starts
// hermetic. The suite always scaffolds fresh sites (names carry Date.now()),
// so nothing here is reused — leftover sites only interfere: the backend runs
// a single global preview, and a stale site still owning it makes the
// live-preview assertions flap.
//
// Runs as the `pretest` hook, i.e. strictly before `playwright test`. It can't
// live in Playwright's globalSetup: Playwright launches the webServer (the
// backend) before globalSetup, so cleaning from there deletes the manifest out
// from under the already-hydrated backend and every CreateSite then fails
// ENOENT. Cleaning here guarantees the backend hydrates from an empty dir.
//
// Matches SITES_DIR in backend/.env and the path the specs read post files from.
const __dirname = dirname(fileURLToPath(import.meta.url));
const SITES_DIR = join(__dirname, '..', 'tmp', 'astro-sites');

await rm(SITES_DIR, { recursive: true, force: true });
