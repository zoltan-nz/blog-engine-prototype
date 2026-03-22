# Blog Engine — Architecture

## System Overview

Headless CMS (Admin App) controls Astro-based static sites. No database — Git is the source of truth. Every site is a
Git repo, every content edit is a commit. Single stack: Rust/Axum backend + SvelteKit frontend.

```
┌─────────────────────────────────────────────────────┐
│              Admin App                               │
│  Frontend (SvelteKit :3000)                         │
│      ↕  OpenAPI 3.1 + generated query clients       │
│  Backend (Rust/Axum :8080)                          │
└───────────────────┬─────────────────────────────────┘
                    │ WebSocket (ws://astro-server/api/agent/ws)
                    ▼
┌─────────────────────────────────────────────────────┐
│              Astro Server                            │
│  astro-supervisor  (Rust binary, connects outbound) │
│  Astro dev server  :4321  (preview, external)        │
│  Content: Markdown files, assets, astro-sites vol   │
└─────────────────────────────────────────────────────┘
```

## Deployment Models

### A. Same machine (current — Docker compose)

CMS backend and supervisor run on the same host (Docker network). Supervisor initiates an outbound WebSocket connection
to the backend. Handler logic is identical across all deployment models — only the connection URL differs.

### B. Remote (future — cloud CMS, local Astro)

CMS runs in the cloud. Supervisor runs on the user's machine (where Node.js and the Astro project live). Supervisor
initiates an outbound WebSocket connection to the cloud CMS — works through NAT/firewall. Git is the content transport.

```
CMS Binary (cloud) ◄──WebSocket── astro-supervisor (user's machine)
                                        │ filesystem
                                   ~/my-astro-project/
```

## Port Scheme

| Service            | Port | Exposed |
|--------------------|------|---------|
| Rust backend       | 8080 | Yes     |
| SvelteKit frontend | 3000 | Yes     |
| Astro preview      | 4321 | Yes     |

## Provider Abstraction

One code path. Local/prod differ only by env var config — not code branches.

| Var | `dev` / `local` | `github` |
|-----|-----------------|----------|
| `AUTH_PROVIDER` | Auto-login, hardcoded dev user | Real GitHub OAuth |
| `GIT_PROVIDER` | Bare repos on Docker volume | GitHub API + push |

`compose.yaml` defaults to `dev`/`local`. `compose.prod.yaml` uses `github`/`github`.

## API Spec Flow

```
backend  →  open-api-contracts/api.yaml  →  frontend (generated clients)
```

Supervisor communicates via WebSocket protocol (spec 0004). Shared types live in the `admin-protocol` crate.
`mise spec-gen` covers the CMS HTTP API pipeline.

## What's Built (Steps 1–10 Complete)

- Svelte + Rust stack running in Docker with hot reload
- `/healthz` endpoint with `{ data, meta }` envelope
- Swagger UI + raw spec at `/api-docs/openapi.json` (Rust)
- Generated query clients in frontend (svelte-query via orval)
- `mise spec-gen` pipeline: Rust → api.yaml → frontend
- Playwright integration tests
- React + Node combo also built (archived) — proved the OpenAPI contract works across stacks

## Step 11: Create Site (In Progress)

Full flow (same in every environment):
1. User logs in → `GET /auth/login` → provider handles → session cookie set
2. User creates site → `POST /sites { name, slug }`
3. Backend sends `CreateSite` command to supervisor via WebSocket; supervisor scaffolds Astro project, `git init`, commit, push
4. Site appears in list with git URL; preview available via supervisor

### Implementation Order
- **A** — `admin-protocol` crate (shared `Command`/`Event`/`Envelope` types)
- **B** — `astro-supervisor` Rust binary (WebSocket client, connects outbound to backend)
- **C** — Auth with dev provider (`AUTH_PROVIDER=dev`)
- **D** — Create site with local git provider (`GIT_PROVIDER=local`)
- **E** — Astro preview
- **F** — GitHub provider (post-MVP)

## Decisions Log

| Date | Decision | Why |
|------|----------|-----|
| 2025-12-19 | OpenAPI 3.1 | Shared types TS ↔ Rust |
| 2025-12-19 | RFC 9457 errors | Industry standard |
| 2025-12-19 | `{ data, meta }` envelope | Simple, room for metadata |
| 2025-12-19 | Filesystem source of truth | No database lock-in |
| 2025-12-19 | Alpine Linux base | Minimal resources |
| 2025-12-25 | Debian slim for Rust builder | Faster glibc builds; musl is slow |
| 2025-12-29 | SPA mode for frontends | No SSR needed for admin UI |
| 2026-01-06 | `compose.prod.yaml` separate | Uses Dockerfile.prod, no volume mounts |
| 2026-01-18 | One GitHub repo per site | Needed for GitHub Pages custom domains |
| 2026-01-18 | Signed cookie for session | Stateless; no server-side store |
| 2026-03-14 | Environment parity via providers | One code path; config-only differences |
| 2026-03-16 | Supervisor as standalone crate | Independent release cycle from CMS backend |
| 2026-03-21 | Drop React + Node — single stack | Proved API contract works; 4× feature cost not justified |
| 2026-03-21 | Supervisor scope: process management only | Content CRUD goes direct to filesystem, not through supervisor |
| 2026-03-22 | WebSocket now (not future) | Supervisor connects outbound; works through NAT; same handler logic across all deployment models |
| 2026-03-22 | `admin-protocol` crate for shared types | New command variant = compile error in every consumer |
| 2026-03-21 | Portable CMS binary (long-term) | `rust-embed` SPA + Axum API = single downloadable binary |

## Resource Benchmarks (Phase 0)

| Container | Image Size | Memory |
|-----------|------------|--------|
| backend-rust-prod | 122 MB | 0.75 MB |
| backend-node-prod | 255 MB | 26.5 MB |

Rust: ~2× smaller image, ~35× less RAM.

## Future

- **Portable CMS binary** — `rust-embed` to serve SPA from backend binary; single download
- **Content volumes** — mount `astro-sites` to CMS backend for direct content CRUD
- **GitHub provider** — auth + git; API shape already exists from dev/local providers
- **Content editing** — Markdown posts via admin UI, each edit = git commit
- **Astro builds** — trigger `astro build` from admin; CI via `repository_dispatch`
- **Gitea** — drop-in upgrade for local git provider with web UI + OAuth
- **Quadlet** — migrate prod from `docker compose` to systemd-native Podman
