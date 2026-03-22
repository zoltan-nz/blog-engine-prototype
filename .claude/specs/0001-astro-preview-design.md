# Astro Site Preview — Design Spec

**Date:** 2026-03-14
**Status:** Approved
**Phase:** C (as defined in CONTEXT.md)

---

## Problem

After creating a blog site through the CMS, there is no way to view it. The site is scaffolded and committed but never served. The user needs a "Preview" button that launches the Astro dev server for a selected site and opens it in a new browser tab.

---

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Preview mechanism | Live Astro dev server | Enables hot-reload for future content editing, not just static viewing |
| UX on click | Blocking POST → new tab | POST waits until port ready (~3 s), prevents tab opening to "connection refused" |
| Active site model | Single dev server at port 4321 | Matches decisions log; port is fixed, no dynamic allocation needed |
| Previous preview | Killed on new POST | Simplest lifecycle for UX; explicit `DELETE /preview` added for test teardown |
| `previewUrl` placement | Field on `SiteData` | Single `GET /sites` call gives full state; no extra polling endpoint |
| Error/log access | Docker container logs (MVP) | `docker logs blog-engine-astro-server-1`; full log streaming is future work |

---

## Architecture

```
Browser               Admin Backend           Management API (:4320)    Astro Dev Server (:4321)
  │                       │                          │                           │
  │─ POST /sites/:slug/preview ──────────────────▶  │                           │
  │                       │─ POST /sites/:slug/preview ──────────────────────▶  │
  │                       │                          │ kill existing dev server  │
  │                       │                          │ spawn: pnpm dev --host    │
  │                       │                          │ poll localhost:4321...    │
  │                       │                          │ (ready in ~3 s)           │
  │                       │ ◀── { previewUrl } ──────│                           │
  │ ◀── 200 { data: SiteData, meta } ───────────────│                           │
  │─ window.open(previewUrl) ────────────────────────────────────────────────▶  │
```

The management API maintains one module-level `activePreview` record: `{ slug, process }` (the Node.js `ChildProcess`). Every new `/preview` call kills the existing process before spawning a new one. The admin backend receives `{ previewUrl }` from the management API and assembles the full `SiteData` response by merging it with the site's stored metadata (`slug`, `name`, `gitUrl`).

`GET /sites` is enriched: each site gets `previewUrl: "http://localhost:4321"` when it matches `activePreview.slug`, `null` otherwise.

---

## API Contract

### Updated `SiteData` schema

```yaml
SiteData:
  type: object
  required: [slug, name, gitUrl]
  properties:
    slug:
      type: string
    name:
      type: string
    gitUrl:
      type: string
    previewUrl:
      type: ["string", "null"]
      description: "URL of the active Astro dev server; null if this site is not currently being previewed. This is the host-side URL (accessible from the browser), not the container-internal network address."
      example: "http://localhost:4321"
```

### New endpoints

```
POST /sites/{slug}/preview
DELETE /preview
```

`DELETE /preview` stops the active dev server (if any) and clears `activePreview`. No body, no slug required — there is only ever one active preview. The endpoint is idempotent: calling it when no preview is active still returns 204.

**`POST /sites/{slug}/preview` responses:**

| Status | Body | When |
|---|---|---|
| 200 | `{ data: SiteData, meta }` — full `SiteData` object with `previewUrl` populated | Dev server ready |
| 404 | RFC 9457 problem details | Slug does not exist |
| 500 | RFC 9457 problem details | Dev server failed to start within 10 s |

**`DELETE /preview` responses:**

| Status | Body | When |
|---|---|---|
| 204 | _(empty)_ | Active preview stopped, or no preview was running |

The 200 response returns the full `SiteData` object (same schema as entries in `GET /sites`) with `previewUrl` set to the active dev server URL. This avoids introducing a second schema shape for the same data.

---

## Component Changes

### `astro-server/management-api.mjs`

- Add module-level `activePreview = null` (`{ slug, process }`).
- Add `async function startPreview(slug)`:
  1. If the site directory `/app/astro-sites/{slug}` does not exist, return HTTP 404 with an RFC 9457 body (`{ type, title, status: 404 }`). The admin backend must forward this 404 — not convert it to 500.
  2. If `activePreview` exists, call `activePreview.process.kill('SIGTERM')` and await the `close` event before proceeding (prevents port-already-in-use races).
  3. Spawn `pnpm dev --host 0.0.0.0` via `spawn()` (non-blocking, unlike `spawnSync`).
  4. Poll `http://localhost:4321` every 200 ms, timeout after 10 s.
  5. Set `activePreview = { slug, process }`.
  6. Return `{ previewUrl: "http://localhost:${PREVIEW_PORT}" }`.
- Add `POST /sites/:slug/preview` handler calling `startPreview`.
- Add `DELETE /preview` handler: if `activePreview` is null, return 204 immediately; otherwise kill `activePreview.process` with `SIGTERM`, await `close` event, set `activePreview = null`, return 204. Always returns 204 — the endpoint is idempotent.
- Update `listSites()` to set `previewUrl` from `activePreview`.
- Add `PREVIEW_PORT` env var (default `4321`). Parse with `parseInt` (same pattern as `MANAGEMENT_PORT`) before constructing the URL string.

### `compose.yaml`

```yaml
astro-server:
  ports:
    - '4321:4321'        # already present in compose.yaml; included here for completeness
  environment:
    - MANAGEMENT_PORT=4320
    - PREVIEW_PORT=4321  # NEW: add this env var
```

**`astro-server/Dockerfile`:** Add `EXPOSE 4321` alongside the existing `EXPOSE 4320`, to document that the container serves on both ports.

### `admin-cms-app/backend-rust/src/lib.rs`

- `SiteData`: add `preview_url: Option<String>` with `#[serde(rename_all = "camelCase")]`.
- New handler `preview_site(Path(slug), State(state))` → `POST /sites/:slug/preview`.
  - If the management API returns 404, the Rust handler must return 404 (not 500). Check `resp.status()` and forward 404 responses verbatim. Only return 500 for other non-OK statuses or network errors.
- New handler `stop_preview(State(state))` → `DELETE /preview` (proxies to management API, returns 204).
- Update `ApiDoc` OpenAPI registration.

### `admin-cms-app/backend-node/src/handlers/sites.ts`

- Add `preview_site` handler: calls `POST {ASTRO_MANAGEMENT_URL}/sites/:slug/preview`, wraps in envelope.
  - The existing `create_site` handler collapses all non-OK upstream responses to 500. This handler must differ: if the management API returns 404, forward it as 404. Only return 500 for other non-OK statuses or network errors.
- Add `stop_preview` handler: calls `DELETE {ASTRO_MANAGEMENT_URL}/preview`, returns 204.

### OpenAPI + generated clients

- Run `mise spec-gen` after Rust changes to regenerate `api.yaml` and all frontend clients.

### Frontends (Svelte + React)

Each site card gains:
- **"Preview" button**: calls `POST /sites/:slug/preview` mutation; shows spinner while pending; on success calls `window.open(previewUrl, '_blank')` and refetches site list.
- **"▶ Live" badge**: rendered when `site.previewUrl !== null`.

---

## Testing (TDD)

### Integration tests (RED first)

```typescript
// integration-tests/tests/sites.spec.ts — new describe block

test.describe('Sites - Preview', () => {
  // Ensure a site exists before running preview tests.
  // Uses a fixed slug so the beforeAll is idempotent across retries.
  test.beforeAll(async ({ request }) => {
    const resp = await request.post(`${BACKEND_URL}/sites`, {
      data: { name: 'Preview Test Blog', slug: 'preview-test' },
    });
    // 201 = created; 409 = already exists (idempotent across retries). Any other status is a setup failure.
    expect([201, 409]).toContain(resp.status());
  });

  test.afterEach(async ({ request }) => {
    // Stop the active preview server so each test starts from a clean state.
    // Intentionally unconditional — DELETE /preview is idempotent (returns 204 even if no preview is active).
    await request.delete(`${BACKEND_URL}/preview`);
  });

  test('POST /sites/:slug/preview returns previewUrl', async ({ request }) => {
    test.setTimeout(60_000); // dev server startup can be slow in a cold container
    const resp = await request.post(`${BACKEND_URL}/sites/preview-test/preview`);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toHaveProperty('previewUrl');
    expect(body.data.previewUrl).toContain('4321');
  });

  test('previewUrl actually serves the Astro site', async ({ request }) => {
    test.setTimeout(60_000);
    const body = (await (await request.post(`${BACKEND_URL}/sites/preview-test/preview`)).json());
    const preview = await request.get(body.data.previewUrl);
    expect(preview.status()).toBe(200);
  });

  test('GET /sites shows previewUrl set for active site', async ({ request }) => {
    test.setTimeout(60_000);
    await request.post(`${BACKEND_URL}/sites/preview-test/preview`);
    const sites = (await (await request.get(`${BACKEND_URL}/sites`)).json()).data;
    const active = sites.find((s: { slug: string }) => s.slug === 'preview-test');
    expect(active.previewUrl).toBeTruthy();
    const others = sites.filter((s: { slug: string }) => s.slug !== 'preview-test');
    others.forEach((s: { previewUrl: string | null }) => expect(s.previewUrl).toBeNull());
  });

  test('UI: site card shows "▶ Live" badge when preview is active', async ({ page, request }) => {
    test.setTimeout(60_000);
    // Start preview via API first so the badge is visible without clicking the button.
    await request.post(`${BACKEND_URL}/sites/preview-test/preview`);
    await page.goto(FRONTEND_URL);
    const badge = page.getByText('▶ Live');
    await expect(badge).toBeVisible();
  });
});
```

### Rust unit tests

```rust
#[test]
fn site_data_serializes_preview_url_as_null_when_none() {
    let site = SiteData { slug: "s".into(), name: "S".into(), git_url: "g".into(), preview_url: None };
    let json = serde_json::to_value(&site).unwrap();
    assert_eq!(json["previewUrl"], serde_json::Value::Null);
}
```

---

## Port Scheme (updated)

| Service | Port | Notes |
|---|---|---|
| Rust backend | 8080 | Sweet spot backend |
| Node backend | 8081 | Alternative backend |
| SvelteKit frontend | 3000 | Sweet spot frontend |
| React frontend | 3001 | Alternative frontend |
| Management API | 4320 | Internal only (not exposed to host) |
| Astro dev server | 4321 | **NEW: exposed to host for preview** |

---

## Future Considerations

- **Log streaming:** `GET /sites/:slug/preview/logs` via Server-Sent Events. The management API would buffer the dev server's stdout/stderr and stream it to the admin UI. Useful for debugging build errors without shelling into the container.
- **Multiple simultaneous previews:** Dynamic port allocation (e.g. `4321 + index`). Requires tracking multiple `activePreview` entries and exposing a port range in compose.
- **Preview persistence across restarts:** Write `activePreview` to `.cms-meta.json` and auto-restart on container boot.
- **Gitea integration:** When Gitea replaces bare repos as the local git provider, webhooks could trigger automatic preview refreshes on push.
