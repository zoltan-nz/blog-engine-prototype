# Blog Engine Prototype

Admin app that creates and controls **Astro** static sites on the local machine.
One Rust binary serves the SPA and a typed WebSocket control plane. No database.

## Current capabilities

- Create / delete Astro sites under a configurable directory (`SITES_DIR`)
- Start / stop a single Astro preview (`PREVIEW_PORT`, default `4321`)
- Build a site and stream build logs over the WebSocket
- Domain lifecycle via pure FSMs (`SiteState`, `PreviewState`)

## Not yet

- Git as source of truth / commits per edit
- Content editing CMS UI
- Deploy to GitHub Pages / Cloudflare Pages
- Multi-preview or reverse-proxy of the preview through the backend

## Prerequisites

- [mise](https://mise.jdx.dev/) (Node, pnpm, Rust via `mise.toml`)
- Node + pnpm on `PATH` for the backend process (Astro scaffold and preview spawn `pnpm`)

## Quick start

```bash
# Terminal 1 — backend on :8080 (SPA from FRONTEND_DIR when built)
mise run backend

# Terminal 2 — SvelteKit dev server (hot reload UI)
mise run frontend

# Optional: regenerate TS wire types when backend types change
mise run export-types
```

| Surface | URL |
|---------|-----|
| Backend (API + production SPA) | http://localhost:8080 |
| Frontend dev (Vite) | http://localhost:5173 |
| Astro preview (when running) | http://localhost:4321 |
| Health | `GET http://localhost:8080/healthz` |
| Control protocol | `ws://localhost:8080/ws` |

## Common tasks

| Task | Command |
|------|---------|
| Backend (`cargo watch`) | `mise run backend` |
| Frontend | `mise run frontend` |
| Export specta → TS | `mise run export-types` |
| Unit tests (once) | `mise run test-unit` |
| Integration (Playwright) | `mise run test` |
| Release single binary | `mise run build` |
| Format | `mise run format` |

## Tech stack

| Layer | Technology |
|-------|------------|
| Frontend | SvelteKit (static SPA), Skeleton v4, Tailwind v4 |
| Backend | Rust, Axum, Tokio |
| Protocol | WebSocket `Command` / `Event`; specta → `frontend/src/lib/types/bindings.ts` |
| Domain | Hand-rolled FSMs in `backend/src/fsm/` |
| Sites | Files on disk + `sites.json` manifest (not Git yet) |
| Task runner | mise |

## Layout

```
backend/            # Axum binary, FSM, Astro process control, WS
frontend/           # SvelteKit admin UI
integration-tests/  # Playwright against backend :8080
.claude/specs/      # Design history (not live runbooks)
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the system map.

## License

MIT
