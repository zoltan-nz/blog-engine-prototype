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
| `app` | Build the Axum `Router`, hydrate sites from disk into `AppState` |
| `config` | `SITES_DIR`, `PREVIEW_PORT`, `FRONTEND_DIR` via envy + dotenvy |
| `routes` | HTTP route table: `/healthz`, `/ws` only |
| `handlers/healthz` | Liveness probe |
| `ws/socket` | WebSocket upgrade and connection loop |
| `ws/dispatch` | `Command` → FSM + `astro::*` → broadcast `Event` |
| `fsm/site` | Pure `SiteState` transitions |
| `fsm/preview` | Pure `PreviewState` transitions |
| `astro/sites` | Manifest (`sites.json`), scaffold (`pnpm create astro`), delete |
| `astro/preview` | Spawn / stop Astro `pnpm dev`, readiness poll |
| `astro/build` | Production build + log streaming |
| `astro/error` | Typed process / IO errors |
| `state` | Shared `AppState` (sites map, preview handle, broadcast channel) |
| `types` | Specta wire types: `Command`, `Event`, `WsEnvelope`, views |
| `telemetry` | Tracing subscriber setup |

## Request paths

```
HTTP GET /healthz  → handlers::healthz
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

## Tests

- Unit: `cargo test` (FSM arms, wire serde, astro helpers, WS dispatch where covered)
- Process-level HTTP/WS: `axum-test` in-crate where present
- Full UI lifecycle: `integration-tests/` via `mise run test`
