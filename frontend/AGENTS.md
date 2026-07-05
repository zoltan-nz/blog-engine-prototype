## Frontend Conventions

- **Icons:** `@lucide/svelte` is installed — always import named icons (`Trash2`, `Loader`, etc.). Never write inline
  `<svg>` markup for icons.
- **Svelte 5 `{@const}`:** must be a direct child of `{#each}`, `{#if}`, `{:else}`, etc. — not nested inside `<div>`
  or other HTML elements. Place all `{@const}` declarations at the top of the block they belong to.
- **HTTP 204 and `response.json()`:** a 204 No Content response has no body. Calling `.json()` on it throws
  `SyntaxError`. Guard all fetch wrappers: `response.status === 204 ? null : await response.json()`.
- **Server state comes from the WebSocket store:** use `getSocket()` from `$lib/state/socket.svelte` — commands
  (`createSite()`, `deleteSite()`, `startPreview()`, …) return a `correlation_id`, and state updates arrive as
  broadcast events into `socket.sites` / `socket.preview`. Never fetch server state over HTTP (only `/healthz` is HTTP).
- **Wire types are generated:** import from `$lib/types/bindings.js`; regenerate with `mise run export-types`
  after changing `backend/src/types.rs`. Never edit `bindings.ts` by hand.

## Environment Gotchas

- **pnpm v10 build scripts:** blocked by default. New Astro projects need a `pnpm-workspace.yaml` with
  `allowBuilds: { esbuild: true, sharp: true }` written before `pnpm install` runs.
- **pnpm v10 in Docker without TTY:** `pnpm dev` or `pnpm install` aborts if it detects a stale `node_modules` and
  has no TTY to confirm. Set `CI=true` in the container environment to suppress the prompt.
- **pnpm global bin path:** pnpm v10 puts binaries in `$PNPM_HOME/bin`, not `$PNPM_HOME`. Dockerfile PATH must
  include both: `ENV PATH="${PNPM_HOME}/bin:${PNPM_HOME}:${PATH}"`.
- **Docker layer caching:** copy `pnpm-workspace.yaml` in the same `COPY` instruction as `package.json` and
  `pnpm-lock.yaml` — before the `RUN pnpm install` step. Missing this causes cache misses and install failures.