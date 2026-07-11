# Blog Engine — Architecture (as built)

## Overview

- **One process:** Rust/Axum binary (`blog-engine-api`).
- **Three HTTP surfaces:** `GET /healthz`, `GET /ws` (WebSocket), static SPA.
- **No database.** Site list lives in `{SITES_DIR}/sites.json`; site files live under `{SITES_DIR}/{slug}/`.
- **No OpenAPI / REST resource API.** Application ops go over the typed WebSocket protocol.
- **Git is not wired yet.** Scaffold runs with `--no-git`; `git_url` on the manifest is currently empty. Treat “Git source of truth” as a future goal, not current behaviour.

## Runtime diagram

```
Browser
  │  Vite :5173 (dev) or SPA from :8080 (prod/embed)
  │
  ▼
Axum :8080
  ├── GET /healthz
  ├── GET /ws  ──► dispatch Command → FSM + astro/* → broadcast Event
  └── static SPA (ServeDir in dev; rust-embed with --features embed)
          │
          ├── sites.json + site folders under SITES_DIR
          └── spawn: pnpm create astro / pnpm install / pnpm dev / build
                      │
                      └── Astro preview :PREVIEW_PORT (browser hits this directly)
```

## Frontend state

- Server state is **push-only** via WebSocket into `frontend/src/lib/state/socket.svelte.ts`.
- Full `Snapshot` on every (re)connect; then `SiteChanged` / `SiteRemoved` / `PreviewChanged` / `BuildLog` / `Error`.
- No TanStack Query for server state.

## Domain FSMs

Pure `transition(state, event)` in `backend/src/fsm/`:

- `SiteState`: Creating → Ready → Building → Ready | BuildFailed; Deleting → removed
- `PreviewState`: Stopped → Starting → Running → Stopping → Stopped; Failed + retry paths

Illegal transitions become typed `Event::Error` (`ErrorCode::InvalidTransition`, etc.).

## Wire types

- Single vocabulary in `backend/src/types.rs` (`Command`, `Event`, `WsEnvelope`, FSM enums).
- Export: `mise run export-types` → `frontend/src/lib/types/bindings.ts` (never hand-edit).

## Config (env)

| Var | Default | Purpose |
|-----|---------|---------|
| `SITES_DIR` | `/tmp/astro-sites` | Astro projects + manifest |
| `PREVIEW_PORT` | `4321` | Astro dev server port |
| `FRONTEND_DIR` | `../frontend/build` | SPA directory for non-embed builds |

Loaded with `dotenvy` + `envy` in `backend/src/config.rs`.

## Deployment shape

- Dev: backend without `embed`; SPA from disk (`FRONTEND_DIR`) or Vite separately.
- Release: `mise run build` → frontend build, then `cargo build --release --features embed`.

## Key modules

| Path | Role |
|------|------|
| `backend/src/ws/` | Upgrade, dispatch, fan-out |
| `backend/src/fsm/` | Pure site/preview transitions |
| `backend/src/astro/` | Manifest, scaffold, preview child, build |
| `backend/src/types.rs` | Specta wire types |
| `frontend/src/lib/state/socket.svelte.ts` | Client store + reconnect |

## Specs

Design history lives under `.claude/specs/`. Prefer this file and the code over draft narrative inside old specs.
