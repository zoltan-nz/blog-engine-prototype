# Blog Engine Agent — Design Spec

**Date:** 2026-03-16
**Status:** Approved

## Overview

Replace `astro-server/management-api.mjs` (interpreted Node.js) with a compiled Rust binary (`blog-engine-agent`)
injected into the astro-server container at Docker build time. The agent is an HTTP server (Axum) that controls the
Astro infrastructure — scaffolding sites, managing git repos, and running the preview dev server.

Architecturally this belongs to the blog controller layer, not the CMS admin app. It lives at the repo root as a
standalone crate.

## Milestones

### Milestone 1 — REST API + Manifest (current)

The HTTP server manages `sites.json` only. No git, no pnpm, no blocking operations. `POST /sites` registers a new site
with status `Pending` and returns immediately. The `.mjs` is deleted after this milestone.

### Milestone 2 — Background Worker

A `tokio::spawn` worker communicates with the HTTP layer via `tokio::mpsc` channel. Picks up status transitions and
executes the real work: Astro scaffolding, git init/commit/push, preview server lifecycle. Status flows:
`Pending → Building → Ready → Previewing → Error`.

## Location

```
blog-engine-agent/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── handlers.rs
│   ├── handlers/
│   │   ├── healthz.rs
│   │   ├── sites.rs
│   │   └── preview.rs
│   ├── state.rs
│   └── error.rs
└── src/bin/
    └── gen_open_api_yaml.rs
```

Not part of a Cargo workspace — independent release cycle from `admin-cms-app/backend-rust`.

## Data Files

Two files manage site state — each with a distinct owner and purpose:

**`/app/astro-sites/sites.json`** — agent manifest, never committed to any site repo:

```json
{
  "sites": [
    {
      "folder": "my-blog",
      "name": "My Blog",
      "git_url": "/app/git-repos/my-blog.git",
      "status": "ready",
      "error": null
    }
  ]
}
```

- Created on first `POST /sites` if missing; treated as empty if absent
- `GET /sites` reads this file — no directory walking
- `POST /sites` appends an entry with `status: "pending"` and returns immediately
- Background worker (Milestone 2) updates status as work progresses

**`SiteStatus` enum:**

```rust
pub enum SiteStatus {
    Pending,    // registered, work not started
    Ready,      // site built and usable
    Previewing, // preview server is active
    Error,      // something went wrong (see error field)
}
```

**`/app/astro-sites/<slug>/.blog-engine-config.json`** — site self-description, committed to the site's git repo:

```json
{
  "name": "My Blog",
  "slug": "my-blog"
}
```

- Written by `POST /sites` during scaffolding, before the initial git commit
- Committed to the site repo so it travels with the site when cloned
- Enables future git-import flow: clone repo → read config → register in `sites.json`

## API Surface

Same contract as the current `.mjs`. Adds `/healthz` for consistency with the other backends.

| Method | Path                   | Description                      |
|--------|------------------------|----------------------------------|
| GET    | `/healthz`             | Health check                     |
| GET    | `/sites`               | List all sites                   |
| POST   | `/sites`               | Create a new site                |
| POST   | `/sites/:slug/preview` | Start preview dev server         |
| DELETE | `/preview`             | Stop active preview (idempotent) |

**Response envelope:** All responses use the same envelope as the CMS backends:
`{ data: T, meta: { timestamp, requestId, version, serverName } }`. The `serverName` value is `blog-engine-agent`. This
is a deliberate deviation from the current `.mjs` (which returns raw types) — consistency across all services and
visibility into which service/version responded outweighs the migration cost. The backends currently deserialise raw
JSON from the agent; migration step 4 updates them to unwrap the `data` field.

## OpenAPI Contract

- `utoipa` generates the spec from handler annotations
- `src/bin/gen_open_api_yaml.rs` exports to `open-api-contracts/agent.yaml` — same naming as
  `backend-rust/src/bin/gen_open_api_yaml.rs` for consistency
- New `mise` task `agent-spec-gen` runs `cargo run --bin gen_open_api_yaml` (separate from `spec-gen` which covers the
  CMS API)
- `agent.yaml` documents the internal API for reference and future typed client generation; client generation tooling is
  deferred to a follow-up step

## State Management

```rust
// state.rs
struct ActivePreview {
    slug: String,
    child: tokio::process::Child,
}

pub struct AppState {
    preview: tokio::sync::Mutex<Option<ActivePreview>>,
}

impl AppState {
    pub fn new() -> Self { ... }
    pub async fn lock_preview(&self) -> tokio::sync::MutexGuard<'_, Option<ActivePreview>> {
        self.preview.lock().await
    }
}
```

Handlers access the preview state exclusively through `lock_preview()` — the field itself is private.

- `tokio::sync::Mutex` — held across `.await` points during port polling
- Concurrency semantics: a second concurrent `POST /sites/:slug/preview` **blocks** until the first completes (same
  queue behaviour as the `.mjs` `previewLock` chain). No timeout on the caller — the existing `.mjs` has the same
  behaviour.
- Stop: `SIGTERM` + `child.wait()` to reap the process and prevent port conflicts on restart

Port polling: `tokio::time::timeout` (10s) wrapping a loop of `tokio::net::TcpStream::connect` probes every 200ms
against `127.0.0.1:4321`. TCP connect is sufficient — the Astro dev server won't open the port until its HTTP stack is
ready, and avoids adding `reqwest` as a dependency to the agent.

## Child Process Management

The agent shells out to the same tools as the `.mjs`, with exact flags preserved:

| Operation            | Command                                                                                      |
|----------------------|----------------------------------------------------------------------------------------------|
| Scaffold site        | `pnpm create astro@latest <siteDir> --template minimal --no-git --skip-houston --no-install` |
| Install deps         | `pnpm install` (in site dir)                                                                 |
| Git config           | `git config --global init.defaultBranch main`                                                |
| Git identity         | `git config user.email cms@blog-engine.local` + `git config user.name "Blog Engine CMS"`     |
| Write config         | write `.blog-engine-config.json` (`{ name, slug }`) into site dir before git commit          |
| Git init/commit/push | `git init`, `git add .`, `git commit`, `git init --bare`, `git remote add`, `git push`       |
| Preview dev server   | `pnpm dev --host 0.0.0.0` (in site dir)                                                      |

`tokio::process::Command` for async (preview server), `std::process::Command` for sync (scaffold, git).

## Error Handling

- `thiserror` for domain errors
- RFC 9457 Problem Details responses — this is a breaking change from the current `.mjs` which returns
  `{ error: "..." }` shapes. The backends currently pass the raw error body through to the frontend verbatim. After
  migration they must parse RFC 9457 and pass it through unchanged. Frontend error-handling paths will also need
  updating to handle RFC 9457 structure — this is a known downstream impact and is in scope for the migration.
- No `.unwrap()` in production code; `.expect("reason")` for invariants only

## Docker Build

Multi-stage build. Builder uses `rust:slim-trixie` (Debian slim) rather than `rust:alpine` due to better C library
compatibility for the `openssl`/`pkg-config` deps. Final image stays `node:25-alpine`.

Dependency caching via dummy `main.rs` pattern (mirrors `backend-rust/Dockerfile.dev`):

```dockerfile
FROM rust:slim-trixie AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY src ./src
RUN cargo build --release --locked

FROM node:25-alpine
RUN apk add --no-cache git && npm install -g pnpm
WORKDIR /app
RUN mkdir -p /app/astro-sites /app/git-repos
COPY --from=builder /build/target/release/blog-engine-agent ./
EXPOSE 4320 4321
CMD ["./blog-engine-agent"]
```

## Testing

- **Unit tests:** Pure functions — path construction, site listing, JSON serialisation (`cargo test`)
- **Integration tests:** A `cargo test` integration test that spawns the agent binary directly (no Docker — binary runs
  on the host during testing) and exercises its HTTP interface (list sites, create site, healthz). This covers the agent
  in isolation before the Playwright tests exercise it end-to-end through the backends.
- **TDD:** RED → GREEN → REFACTOR, target 100% coverage on business logic

## Migration Steps

1. Create `blog-engine-agent/` crate and implement the agent with full test coverage
2. Update `astro-server/Dockerfile` to the multi-stage build above
3. Update `compose.yaml` healthcheck for `astro-server`: replace `node -e "require('node:http')..."` with
   `wget -q --spider http://127.0.0.1:4320/healthz`
4. Update `backend-rust` and `backend-node` error handling to parse RFC 9457 from the agent (was `{ error: "..." }`)
5. Add `agent-spec-gen` mise task
6. Update existing `mise` tasks to cover the new crate: `test-unit` (or new `test-unit-agent`), `format`
7. Delete `astro-server/management-api.mjs`
8. Verify all existing integration tests pass
