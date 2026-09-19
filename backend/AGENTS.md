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

## Rust pattern checklist

Drawn from the [rust-unofficial/patterns](https://rust-unofficial.github.io/patterns/) catalog. Apply the relevant group before writing or reviewing code.

### Adding a new type

| Check | Rule |
|-------|------|
| Primitive wrapper | Newtype over raw primitive (`struct PostId(String)`, not bare `String`) |
| Constructors | Use `new()` or a builder; keep fields private |
| `Default` | Derive or implement `Default` when a zero-value makes sense — reduces test boilerplate |
| Function signatures | Accept `&str` / `&Path`, not `String` / `PathBuf` — callers should not clone just to call you |

### Adding a new feature

| Check | Rule |
|-------|------|
| Resource cleanup | RAII: tie teardown to `Drop`, not to a manual `close()` call the caller might forget |
| Map mutation | Use `HashMap::entry()` instead of a get-then-insert pair — avoids a double lookup |
| Complex construction | Builder pattern when a struct has more than three optional fields |
| Error propagation | `thiserror` + `?`; no `.unwrap()` outside tests; `.expect("invariant reason")` for true panics |
| Shared mutable state | `Arc<RwLock<T>>` for read-heavy state; `Arc<Mutex<T>>` when writes dominate |

### Refactoring existing code

| Check | Rule |
|-------|------|
| Behaviour variation | Strategy via trait objects or generics — not a match arm per variant |
| Gratuitous `.clone()` | A clone to satisfy the borrow checker is a design smell; restructure ownership instead |
| Boolean parameters | Replace `fn f(flag: bool)` with `enum Direction { Forward, Backward }` |
| Arrow anti-pattern | Early return / `?` / guard clauses over nested `if`/`match` |

## Two-step self-review

Run this before finalising any new module or non-trivial change.

**Step 1 — write the code.** Implement the feature, following the standards above.

**Step 2 — idiom pass.** Read what you just wrote and answer each question:

1. Does any function accept a `String` or `PathBuf` where `&str` or `&Path` would do?
2. Is there a `.clone()` that exists only to satisfy the borrow checker?
3. Does any `HashMap` mutation use get-then-insert instead of `entry()`?
4. Is any resource cleanup done via an explicit method call rather than `Drop`?
5. Does any struct have a boolean field that should be an enum?
6. Are there nested `if` / `match` arms that an early return would flatten?
7. Does any new public type lack a `Default` impl it would naturally support?

Fix every "yes" before the code is considered done.
