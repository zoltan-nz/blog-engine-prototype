# Backend Rules

System map: root [`ARCHITECTURE.md`](../ARCHITECTURE.md). Module map and extension invariants:
[`ARCHITECTURE.md`](./ARCHITECTURE.md). Generic Rust style lives in `~/.claude/rules/rust.md`.

## Project specifics

- Config fields live in `src/config.rs` (`SITES_DIR`, `PREVIEW_PORT`, `FRONTEND_DIR`).
- Tracing default filter (when `RUST_LOG` is unset): `backend=debug,tower_http=debug` (`src/telemetry.rs`).
- Domain errors live under `astro::error`.

## Protocol surface

- HTTP handlers: `healthz` and read-only `site_files` (plus static SPA fallback outside handlers).
- Domain ops: `ws::dispatch` on `Command` variants; outcomes are `Event` broadcasts.
- Wire types: `src/types.rs`; after changes run `mise run export-types`.
- FSM transitions: pure functions in `src/fsm/`; illegal commands → `Event::Error` with `InvalidTransition`.
- Reconnect sends a fresh `Snapshot`, not event replay.
