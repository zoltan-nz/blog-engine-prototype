# Backend Architecture

Companion to the root [ARCHITECTURE.md](../ARCHITECTURE.md). This file is the
module map for `backend/` as agents and humans extend the crate.

## Binary and library

| Target | Path | Role |
|--------|------|------|
| `blog-engine-api` | `src/main.rs` | Process entry: tracing, config, bind `:8080` |
| `export-types` | `src/bin/export-types.rs` | Write specta TS bindings to the frontend |
| library `backend` | `src/lib.rs` | App construction, modules, shared helpers |

Feature flag:

- `embed` — compile `../frontend/build` into the binary via `rust-embed` and serve it; without the feature, SPA is `ServeDir` from `FRONTEND_DIR`.

## Module map

| Module | Responsibility |
|--------|----------------|
| `app` | Build the Axum `Router`, hydrate sites from disk into `AppState`, start a watcher per site |
| `config` | `SITES_DIR`, `PREVIEW_PORT`, `FRONTEND_DIR` via envy + dotenvy |
| `routes` | HTTP route table: `/healthz`, `/ws`, `/site-files/{slug}/{*path}` |
| `handlers/healthz` | Liveness probe |
| `handlers/site_files` | Read-only `ServeDir` over a known site's `src/`; 404 for unknown slugs |
| `ws/socket` | WebSocket upgrade and connection loop |
| `ws/dispatch` | `Command` → FSM + `astro::*` → broadcast `Event` |
| `fsm/site` | Pure `SiteState` transitions |
| `fsm/preview` | Pure `PreviewState` transitions |
| `astro/sites` | Manifest (`sites.json`), scaffold (`pnpm create astro`), delete |
| `astro/posts` | Post CRUD: frontmatter parse/render, list/read/create/update/delete, atomic writes |
| `astro/preview` | Spawn / stop Astro `pnpm dev`, readiness poll |
| `astro/build` | Production build + log streaming |
| `astro/watch` | Per-site debounced watcher on `src/content/blog/`; emits `PostChanged` / `PostRemoved` for external edits |
| `astro/error` | Typed process / IO errors |
| `state` | Shared `AppState` (sites map with posts cache, preview handle, per-site watchers, broadcast channel) |
| `types` | Specta wire types: `Command`, `Event`, `WsEnvelope`, views |
| `telemetry` | Tracing subscriber setup |

## Request paths

```
HTTP GET /healthz  → handlers::healthz
HTTP GET /site-files/{slug}/{*path} → handlers::site_files::site_file
HTTP GET /ws       → ws::socket::upgrade_ws
                     → parse WsEnvelope
                     → ws::dispatch (async work + Event fan-out)
fallback           → SPA (disk or embedded)
```

## Invariants when extending

1. New application ops are new `Command` / `Event` variants in `types.rs`, not new REST routes.
2. Lifecycle changes go through `fsm::*::transition`; do not mutate “effective state” only in handlers.
3. After wire-type changes: `mise run export-types`.
4. Keep `main.rs` free of domain logic; put it in modules with unit tests.
5. Sites on disk are the persistence model today — not Git.
6. Any client-supplied id that gets joined into a filesystem path (post id, site slug) must pass `validate_id` (`^[a-z0-9-]{1,64}$`) at the dispatch boundary first — reject with `ErrorCode::InvalidInput` otherwise.

## Tests

- Unit: `cargo test` (FSM arms, wire serde, astro helpers, WS dispatch where covered)
- Process-level HTTP/WS: `axum-test` in `backend/tests/` (`e2e_healthz`, `e2e_site_files`, `ws_protocol`)
- Full UI lifecycle: `integration-tests/` via `mise run test`
