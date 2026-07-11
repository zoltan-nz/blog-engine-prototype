# Frontend Rules

System map for the whole product: root [`ARCHITECTURE.md`](../ARCHITECTURE.md).
Frontend module map: [`ARCHITECTURE.md`](./ARCHITECTURE.md).

## Frontend Conventions

- **Icons:** `@lucide/svelte` is installed — always import named icons (`Trash2`, `Loader`, etc.). Never write inline
  `<svg>` markup for icons.
- **UI kit:** Skeleton v4 + Tailwind v4 (`@skeletonlabs/skeleton`, `@skeletonlabs/skeleton-svelte`). Not shadcn.
- **Svelte 5 `{@const}`:** must be a direct child of `{#each}`, `{#if}`, `{:else}`, etc. — not nested inside `<div>`
  or other HTML elements. Place all `{@const}` declarations at the top of the block they belong to.
- **Server state comes from the WebSocket store:** use `getSocket()` from `$lib/state/socket.svelte` — commands
  (`createSite()`, `deleteSite()`, `startPreview()`, …) return a `correlation_id`, and state updates arrive as
  broadcast events into `socket.sites` / `socket.preview`. Never fetch server state over HTTP (only `/healthz` is HTTP).
- **Wire types are generated:** import from `$lib/types/bindings.js`; regenerate with `mise run export-types`
  after changing `backend/src/types.rs`. Never edit `bindings.ts` by hand.

## Environment Gotchas

- **pnpm v10 build scripts:** blocked by default for packages that need native postinstalls. Astro projects scaffolded
  by the backend need a `pnpm-workspace.yaml` with `allowBuilds` for `esbuild` / `sharp` before `pnpm install`
  (handled in `backend` scaffold code). The admin frontend app itself is a normal SvelteKit package.
- **Backend URL:** `PUBLIC_API_BACKEND_URL` (see `socket.svelte.ts`); defaults to `http://localhost:8080`.

## Tests

- Unit / component: `pnpm exec vitest run` (browser mode via Playwright provider for `*.svelte.spec.ts`)
- Integration against a live backend: repo-root `mise run test` (Playwright in `integration-tests/`)
