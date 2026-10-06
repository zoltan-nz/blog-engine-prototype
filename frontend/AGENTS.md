# Frontend Rules

System map: root [`ARCHITECTURE.md`](../ARCHITECTURE.md). Module map: [`ARCHITECTURE.md`](./ARCHITECTURE.md).
Generic Svelte/pnpm rules live in `~/.claude/rules/frontend.md`.

## Conventions

- **UI kit:** Skeleton v5 + Tailwind v4 (`@skeletonlabs/skeleton`, `@skeletonlabs/skeleton-svelte`). Not shadcn.
- **Server state comes from the WebSocket store:** `getSocket()` from `$lib/state/socket.svelte`. Fire-and-forget
  commands (`createSite()`, `startPreview()`, `createPost()`, …) return a `correlation_id`; state arrives as
  broadcasts into `socket.sites` / `socket.preview` / `socket.posts`. `updatePost()`, `draftPost()` and `requestPost()`
  return promises settled by the matching event. Never fetch server state over HTTP; the only other HTTP route is
  read-only `/site-files/…` for post-editor images.
- **Wire types are generated:** import from `$lib/types/bindings.js`; regenerate with `mise run export-types`.
- **Backend URL:** `PUBLIC_API_BACKEND_URL` (see `socket.svelte.ts`); defaults to `http://localhost:8080`.

## Tests

- Unit / component: `pnpm exec vitest run` (browser mode via Playwright provider for `*.svelte.spec.ts`)
- Type check: `pnpm run check`
- Integration against a live backend: repo-root `mise run test` (Playwright in `integration-tests/`)
