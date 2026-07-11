# Frontend Architecture

Companion to the root [ARCHITECTURE.md](../ARCHITECTURE.md). This file is the
module map for `frontend/` as agents and humans extend the admin UI.

## Shape

- **SvelteKit static SPA** (`adapter-static`). No SSR data loading for server domain state.
- **Served by** the Rust backend in production (`FRONTEND_DIR` or `--features embed`).
- **Dev:** Vite (default port 5173) with the backend expected on `:8080`.

## Data flow

```
UI components
    │  call getSocket().createSite / startPreview / …
    ▼
BlogSocket (src/lib/state/socket.svelte.ts)
    │  send WsEnvelope{ Command }
    │  receive Snapshot | SiteChanged | PreviewChanged | BuildLog | Error | Pong
    ▼
$state: sites, preview, buildLogs, lastError, status
    │
    ▼
Reactive UI (routes, cards, footer connection badge)
```

Rules:

- Server state is **push-only**. Do not introduce HTTP resource clients for sites/preview.
- On (re)connect the server sends a full `Snapshot`; the client replaces local lists — no event replay.
- Commands return a `correlation_id`; match `Event::Error` for that id when showing failures.

## Directory map

| Path | Role |
|------|------|
| `src/routes/+page.svelte` | Main admin UI (site list, create dialog, actions) |
| `src/routes/+layout.svelte` | Shell, theme/font setup |
| `src/routes/+layout.ts` | `ssr = false` (SPA) |
| `src/lib/state/socket.svelte.ts` | WebSocket client, reducers, connection lifecycle |
| `src/lib/state/font.svelte.ts` | Client-only font preference |
| `src/lib/types/bindings.ts` | **Generated** wire types — never hand-edit |
| `src/lib/components/` | Footer, theme selector, font selector |
| `src/lib/test/mocks/` | Env + socket mocks for Vitest |

## Wire types

Source of truth: `backend/src/types.rs`.

```bash
mise run export-types
# → frontend/src/lib/types/bindings.ts
```

Import as `$lib/types/bindings.js` (SvelteKit ESM convention).

## UI stack

- Svelte 5 runes (`$state`, `$derived`, …)
- Skeleton v4 + Tailwind v4
- Icons: `@lucide/svelte`

## Tests

| Kind | How |
|------|-----|
| Component / reducer unit | `pnpm exec vitest run` (browser Playwright provider for `*.svelte.spec.ts`) |
| Type check | `pnpm run check` |
| End-to-end against live backend | `mise run test` from repo root (`integration-tests/`) |

## Invariants when extending

1. New domain fields/actions start in Rust `types.rs` + FSM/dispatch, then export-types, then UI.
2. Keep pure list reducers (`upsertSite`, `removeSite`, …) testable without a browser when possible.
3. Do not reintroduce TanStack Query (or similar) for backend domain state unless the architecture deliberately changes.
4. Prefer Skeleton primitives over one-off markup for dialogs, badges, and layout.
