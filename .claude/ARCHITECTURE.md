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
                    │ HTTP (internal network)
                    ▼
┌─────────────────────────────────────────────────────┐
│              Astro Server                            │
│  blog-engine-agent :4320  (Rust binary, internal)   │
│  Astro dev server  :4321  (preview, external)        │
│  Content: Markdown files, assets, astro-sites vol   │
└─────────────────────────────────────────────────────┘
```

## Deployment Models

### A. Same machine (current — Docker compose)

CMS backend and agent run on the same host (or Docker network). Agent is an HTTP server; backend calls it via
`ASTRO_MANAGEMENT_URL`. This is internal IPC, not a public API.

### B. Remote (future — cloud CMS, local Astro)

CMS runs in the cloud. Agent runs on the user's machine (where Node.js and the Astro project live). Agent initiates an
outbound WebSocket connection to the CMS — works through NAT/firewall. Git is the content transport.

```
CMS Binary (cloud) ◄──WebSocket── Agent (user's machine)
                                        │ filesystem
                                   ~/my-astro-project/
```

## Port Scheme

| Service            | Port | Exposed |
|--------------------|------|---------|
| Rust backend       | 8080 | Yes     |
| SvelteKit frontend | 3000 | Yes     |
| Astro preview      | 4321 | Yes     |
| Agent (internal)   | 4320 | No      |

## Provider Abstraction

One code path. Local/prod differ only by env var config — not code branches.

| Var | `dev` / `local` | `github` |
|-----|-----------------|----------|
| `AUTH_PROVIDER` | Auto-login, hardcoded dev user | Real GitHub OAuth |
| `GIT_PROVIDER` | Bare repos on Docker volume | GitHub API + push |

`compose.yaml` defaults to `dev`/`local`. `compose.prod.yaml` uses `github`/`github`.

## API Spec Flow

```
blog-engine-agent  →  open-api-contracts/agent.yaml  →  backends (clients)
backend-rust       →  open-api-contracts/api.yaml    →  frontends (clients)
```

`mise agent-spec-gen` and `mise spec-gen` wire each pipeline.

## What's Built (Steps 1–10 Complete)

- Svelte + Rust stack running in Docker with hot reload
- `/healthz` endpoint with `{ data, meta }` envelope
- Swagger UI + raw spec at `/api-docs/openapi.json` (Rust)
- Generated query clients in frontend (svelte-query via orval)
- `management-api.mjs` — site listing, creation, preview start/stop (to be replaced by agent)
- `mise spec-gen` pipeline: Rust → api.yaml → frontend
- Playwright integration tests
- React + Node combo also built (archived) — proved the OpenAPI contract works across stacks

## Step 11: Create Site (In Progress)

Full flow (same in every environment):
1. User logs in → `GET /auth/login` → provider handles → session cookie set
2. User creates site → `POST /sites { name, slug }`
3. Backend calls agent: scaffold Astro project, `git init`, commit, push to remote
4. Site appears in list with git URL; preview available via agent

### Implementation Order
- **A** — `blog-engine-agent` Rust binary (replaces `management-api.mjs`)
- **B** — Auth with dev provider (`AUTH_PROVIDER=dev`)
- **C** — Create site with local git provider (`GIT_PROVIDER=local`)
- **D** — Astro preview
- **E** — GitHub provider (post-MVP)

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
| 2026-01-01 | Compose profiles, not multiple files | Single `compose.yaml` with `--profile` |
| 2026-01-06 | `compose.prod.yaml` separate | Uses Dockerfile.prod, no volume mounts |
| 2026-01-18 | One GitHub repo per site | Needed for GitHub Pages custom domains |
| 2026-01-18 | Signed cookie for session | Stateless; no server-side store |
| 2026-03-14 | Environment parity via providers | One code path; config-only differences |
| 2026-03-16 | Agent as internal service | No envelope on internal APIs; raw domain types |
| 2026-03-16 | `blog-engine-agent` standalone crate | Independent release cycle from CMS backend |
| 2026-03-21 | Drop React + Node — single stack | Proved API contract works; 4× feature cost not justified |
| 2026-03-21 | Agent scope: process management only | Content CRUD goes direct to filesystem, not through agent |
| 2026-03-21 | Agent HTTP now, WebSocket future | HTTP routes map to WS commands; handler logic unchanged |
| 2026-03-21 | Portable CMS binary (long-term) | `rust-embed` SPA + Axum API = single downloadable binary |

## Resource Benchmarks (Phase 0)

| Container | Image Size | Memory |
|-----------|------------|--------|
| backend-rust-prod | 122 MB | 0.75 MB |
| backend-node-prod | 255 MB | 26.5 MB |

Rust: ~2× smaller image, ~35× less RAM.

## Future

- **Portable CMS binary** — `rust-embed` to serve SPA from backend binary; single download
- **WebSocket agent** — agent connects outbound to cloud CMS for remote deployment
- **Content volumes** — mount `astro-sites` to CMS backend for direct content CRUD
- **GitHub provider** — auth + git; API shape already exists from dev/local providers
- **Content editing** — Markdown posts via admin UI, each edit = git commit
- **Astro builds** — trigger `astro build` from admin; CI via `repository_dispatch`
- **Gitea** — drop-in upgrade for local git provider with web UI + OAuth
- **Quadlet** — migrate prod from `docker compose` to systemd-native Podman
