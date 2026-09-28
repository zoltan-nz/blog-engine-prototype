Global rules (workflow, style, coding standards) live in `~/.claude/CLAUDE.md`. This file adds only
project-specific context.

## Tech stack

| Layer               | Technology                              |
|---------------------|-----------------------------------------|
| Env and task runner | `mise`                                  |
| Frontend            | SvelteKit                               |
| UI components       | Skeleton v5 + Tailwind v4               |
| Backend             | Rust/Axum                               |
| Integration tests   | Playwright (run on host, not in Docker) |
| Unit tests          | Vitest (JS/TS), cargo test (Rust)       |
| Package managers    | cargo, pnpm                             |

## Repo shape (not microservices)

One backend binary and one frontend SPA.

| Path | Role |
|------|------|
| `backend/` | Axum API, WS protocol, FSMs, Astro process control |
| `frontend/` | SvelteKit admin UI |
| `integration-tests/` | Playwright (host, not Docker) |
| `.claude/specs/` | Historical design notes |

Don't invent service names or `test-{service}` tasks that aren't in `mise.toml`.

## Commands (source of truth: `mise.toml`)

| Intent | Command |
|--------|---------|
| Run backend | `mise run backend` |
| Run frontend | `mise run frontend` |
| Export wire types | `mise run export-types` |
| Unit tests once | `mise run test-unit` |
| Integration tests | `mise run test` |
| Release binary | `mise run build` |
| Format | `mise run format` |

Layer READMEs document env vars and package-local `cargo` / `pnpm` commands. Keep them aligned with `mise.toml`.

## Architecture invariants (do not regress without an explicit decision)

1. Application protocol is **WebSocket-only** (`/ws`); HTTP is `/healthz`, read-only `/site-files/{slug}/{*path}` (a site's `src/`, so the editor can show post images), and the static SPA.
2. Wire types live in `backend/src/types.rs` and are exported with specta. Never hand-edit `bindings.ts`.
3. Domain transitions go through FSMs in `backend/src/fsm/`; reject illegal commands with typed errors.
4. No Docker-based workflow in this repo currently.
5. **Docs match code.** If behaviour changes, update README / ARCHITECTURE / AGENTS in the same change. Specs are history, not live config.

## Documentation map

- Product + run: `README.md`
- System design (as built): `ARCHITECTURE.md`
- Project context: this file
- Backend Rust rules: `backend/AGENTS.md` and `backend/ARCHITECTURE.md`
- Frontend Svelte rules: `frontend/AGENTS.md` and `frontend/ARCHITECTURE.md`
