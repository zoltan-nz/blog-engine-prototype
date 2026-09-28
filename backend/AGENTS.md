# Backend Rules

System map for the whole product: root [`ARCHITECTURE.md`](../ARCHITECTURE.md).
Backend module map: [`ARCHITECTURE.md`](./ARCHITECTURE.md).

## Module style

- No `mod.rs`. Use named files (`handlers.rs` + `handlers/`).
- Registry files (`handlers.rs`, `ws.rs`, `fsm.rs`, `astro.rs`) declare child modules only, no logic.
- Logic lives in named children: `handlers/healthz.rs`, `ws/dispatch.rs`, `fsm/site.rs`, `astro/preview.rs`, …
- `main.rs` holds startup only: tracing init, config load, app construction, server bind.

## Library stack

| Concern         | Library                                       | Notes                                    |
|-----------------|-----------------------------------------------|------------------------------------------|
| Config          | `envy` + `dotenvy`                            | `Config::from_env()` after dotenvy load  |
| Structured log  | `tracing` + `tracing-subscriber`              | `EnvFilter` from env, `fmt::layer()`     |
| HTTP middleware | `tower-http` `TraceLayer` / CORS / `ServeDir` | Request logging + SPA serving            |
| Error types     | `thiserror`                                   | Domain errors under `astro::error`, etc. |

- Config fields live in `src/config.rs` (`SITES_DIR`, `PREVIEW_PORT`, `FRONTEND_DIR`). Never call `std::env::var`
  directly; go through `Config`.
- `dotenvy::dotenv().ok()` loads `.env` in dev and is silently ignored when no file exists.
- Never mutate global env in tests. Inject via constructors or `envy::from_iter()`.
- Tracing: init first in `main.rs` via `registry().with(EnvFilter).with(fmt::layer())`; default filter
  `backend=debug`. Every handler logs at least one structured event
  (`tracing::info!(slug = %slug, "preview started")`).
- Tests: `axum-test` (with the `ws` feature) for HTTP and WebSocket handlers.

## Protocol surface

- HTTP handlers: `healthz` and read-only `site_files` (plus static SPA fallback outside handlers).
- Domain ops: `ws::dispatch` on `Command` variants; outcomes are `Event` broadcasts.
- Verb-first names: `upgrade_ws`, `dispatch_command`, `start_preview`, `scaffold_site`.
- Wire types: `src/types.rs`; after changes run `mise run export-types`.
- FSM transitions: pure functions in `src/fsm/`; illegal commands → `Event::Error` with `InvalidTransition`.
- Reconnect sends a fresh `Snapshot`, not event replay.

## Rust idioms

Clippy runs `all`, `pedantic`, `nursery` (see `lib.rs`). Before calling a change done, check:

- Newtype over raw primitives (`struct PostId(String)`); private fields behind `new()` or a builder.
- Accept `&str` / `&Path`, not `String` / `PathBuf`.
- No `.clone()` just to satisfy the borrow checker; restructure ownership.
- `HashMap::entry()` over get-then-insert.
- Cleanup via `Drop` (RAII), not a manual `close()`.
- `thiserror` + `?`; no `.unwrap()` outside tests; `.expect("reason")` for invariants.
- No `get_` prefix on getters.
- Derive `Default` where a zero value makes sense.
- `Arc<RwLock<T>>` for read-heavy shared state, `Arc<Mutex<T>>` when writes dominate.
