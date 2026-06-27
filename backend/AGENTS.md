# Backend Rules

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
- Clippy: `#![warn(clippy::all, clippy::pedantic, clippy::nursery)]`
- Axum: `impl IntoResponse`, use extractors (`State`, `Json`, `Path`), tower for middleware
- Module files: use named files (`handlers.rs` + `handlers/`) not `mod.rs` (Rust 2018+)

## Module Style

`mod.rs` is forbidden. Use named files only:
- `handlers.rs` declares `pub mod healthz;` etc. — **module registry only, no logic**
- Logic lives in `handlers/healthz.rs`, `handlers/sites.rs`, etc.
- `main.rs` contains startup steps only — tracing init, config load, state construction, server bind. No logic.
- Any logic extracted to its own function or module

## Standard Library Stack (apply to every Rust crate)

| Concern          | Library                          | Notes                                          |
|------------------|----------------------------------|------------------------------------------------|
| Config           | `envy` + `dotenvy`               | `envy::from_env::<Config>()` after dotenvy load |
| Structured log   | `tracing` + `tracing-subscriber` | `EnvFilter` from env, `fmt::layer()`           |
| HTTP middleware  | `tower-http` `TraceLayer`        | Request/response logging on every HTTP service |
| Error types      | `thiserror`                      | One `Error` enum per crate                     |

Config pattern — apply consistently:
```rust
#[derive(serde::Deserialize)]
struct Config {
    backend_ws_url: String,   // env: BACKEND_WS_URL
    preview_port: u16,        // env: PREVIEW_PORT
}
// In main: dotenvy::dotenv().ok(); let config = envy::from_env::<Config>().expect("...");
```
- `dotenvy::dotenv().ok()` — loads `.env` in dev, silently ignored in prod (no `.env` file)
- Never use `std::env::var` directly — always go through the `Config` struct
- Never mutate global env in tests — use `envy::from_iter()` to inject values in tests

## Tracing Conventions

- `main.rs` initialises tracing first, before anything else
- Every handler emits at least one `tracing::info!` or `tracing::debug!` span
- Use structured fields: `tracing::info!(site_id = %id, "preview started")` not string interpolation
- HTTP services add `TraceLayer` from `tower-http` for automatic request/response logging
- Filter default: `"{crate_name}=debug"` (underscored crate name, e.g. `astro_supervisor=debug`)

## Handler Naming Conventions

HTTP handlers are named after their action, not the HTTP method:

| Pattern     | Example                           |
|-------------|-----------------------------------|
| list_*      | `list_sites`, `list_posts`        |
| create_*    | `create_site`, `create_post`      |
| delete_*    | `delete_site`                     |
| preview_*   | `preview_site`                    |
| stop_*      | `stop_preview`                    |

## WebSocket 

WS dispatch functions: `dispatch_command`, `forward_command`, `resolve_event` — verb-first.

