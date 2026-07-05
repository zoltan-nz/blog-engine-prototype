# backend

Rust/Axum backend for the Blog Engine. Serves the SvelteKit SPA, exposes
`GET /healthz` (HTTP) and `GET /ws` — the typed WebSocket protocol that carries
all site/preview/build operations. Domain state is governed by finite state
machines in `src/fsm/`; wire types in `src/types.rs` are exported to
TypeScript via specta.

## Env vars

| Var            | Default             | Purpose                               |
| -------------- | ------------------- | ------------------------------------- |
| `SITES_DIR`    | `/tmp/astro-sites`  | Where Astro projects are generated    |
| `PREVIEW_PORT` | `4321`              | Port for the Astro dev-server preview |
| `FRONTEND_DIR` | `../frontend/build` | SPA directory (dev builds, `ServeDir`)|

## Run

```sh
cargo run                                  # dev: SPA served from FRONTEND_DIR
cargo build --release --features embed     # single binary, SPA embedded
```

## Test

```sh
cargo test
```

## Type bindings

```sh
cargo run --bin export-types    # writes frontend/src/lib/types/bindings.ts
```
