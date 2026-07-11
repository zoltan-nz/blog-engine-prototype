# Backend Rules

System map for the whole product: root [`ARCHITECTURE.md`](../ARCHITECTURE.md).
Backend module map: [`ARCHITECTURE.md`](./ARCHITECTURE.md).

## Rust Standards (RFC 430)

| Element            | Convention           | Example        |
|--------------------|----------------------|----------------|
| Crates/modules     | snake_case           | `http_client`  |
| Types/traits/enums | UpperCamelCase       | `HealthStatus` |
| Functions/methods  | snake_case           | `get_status()` |
| Constants          | SCREAMING_SNAKE_CASE | `MAX_RETRIES`  |

- No `get_` prefix on getters: `fn name(&self)` not `fn get_name(&self)`
- Newtype over primitive: `struct UserId(u64)` not raw `u64`
- Enum over boolean: `enum Visibility { Public, Private }` not `is_public: bool`
- Errors: `thiserror`, `?` operator, no `.unwrap()` in production (use `.expect("reason")` for invariants)
- Clippy: `#![warn(clippy::all, clippy::pedantic, clippy::nursery)]` (see `lib.rs`)
- Axum: `impl IntoResponse`, use extractors (`State`, `Json`, `Path`), tower for middleware
- Module files: use named files (`handlers.rs` + `handlers/`) not `mod.rs` (Rust 2018+)

## Module Style

`mod.rs` is forbidden. Use named files only:

- Registry files (`handlers.rs`, `ws.rs`, `fsm.rs`, `astro.rs`) declare child modules only — **no logic**
- Logic lives in named children: `handlers/healthz.rs`, `ws/dispatch.rs`, `fsm/site.rs`, `astro/preview.rs`, …
- `main.rs` contains startup steps only: tracing init, config load, app construction, server bind
- Any logic extracted to its own function or module

## Standard Library Stack

| Concern          | Library                          | Notes                                          |
|------------------|----------------------------------|------------------------------------------------|
| Config           | `envy` + `dotenvy`               | `Config::from_env()` after dotenvy load        |
| Structured log   | `tracing` + `tracing-subscriber` | `EnvFilter` from env, `fmt::layer()`           |
| HTTP middleware  | `tower-http` `TraceLayer` / CORS / `ServeDir` | Request logging + SPA serving     |
| Error types      | `thiserror`                      | Domain errors under `astro::error`, etc.       |

Real config fields (see `src/config.rs`):

```rust
#[derive(serde::Deserialize)]
struct Config {
    #[serde(default = "default_sites_dir")]
    sites_dir: PathBuf,      // env: SITES_DIR
    #[serde(default = "default_preview_port")]
    preview_port: u16,       // env: PREVIEW_PORT
    #[serde(default = "default_frontend_dir")]
    frontend_dir: PathBuf,   // env: FRONTEND_DIR
}
```

- `dotenvy::dotenv().ok()` — loads `.env` in dev, silently ignored when no file
- Never use `std::env::var` directly — always go through the `Config` struct
- Never mutate global env in tests — inject via constructors / `envy::from_iter()` where needed

## Tracing Conventions

- `main.rs` initialises tracing first, before anything else
- Prefer structured fields: `tracing::info!(slug = %slug, "preview started")`
- Filter default: `backend=debug` (crate name from `Cargo.toml`)

## Protocol surface

- HTTP handlers: only `healthz` (plus static SPA fallback outside handlers)
- Domain ops: `ws::dispatch` on `Command` variants; outcomes are `Event` broadcasts
- Prefer verb-first internal names: `dispatch_command`, `start_preview`, `scaffold_site`, `stop_preview`
- Wire types: `src/types.rs` — after changes run `mise run export-types` (or `cargo run --bin export-types`)
- FSM transitions: pure functions in `src/fsm/`; illegal commands → `Event::Error` with `InvalidTransition`

## WebSocket

WS functions: `upgrade_ws`, `dispatch_command` — verb-first. Reconnect strategy is a fresh `Snapshot`, not event replay.
