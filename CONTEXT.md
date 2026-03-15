# Blog Engine Prototype — Project Context

## What This Project Is

A headless CMS that controls Astro-based static sites. An **Admin App** (web UI + API backend) manages content stored as Markdown files inside an **Astro Server**. The generated static output deploys to GitHub Pages or Cloudflare Pages. There is no database — Git is the source of truth. Every site is a Git repository, every content edit is a commit.

The project doubles as a **framework comparison sandbox**: two frontends (SvelteKit, React) and two backends (Rust/Axum, Node/Fastify) all implement the same API contract. The "sweet spot" pairing is **SvelteKit + Rust**.

---

## Current Architecture

```
┌─────────────────────────────────────────────────────┐
│              Admin App (Controller)                  │
│                                                     │
│  Frontend (SvelteKit :3000 or React :3001)          │
│      ↕  OpenAPI 3.1 + generated query clients       │
│  Backend (Rust/Axum :8080 or Node/Fastify :8081)    │
└───────────────────┬─────────────────────────────────┘
                    │ shared volume (file ops)
                    ▼
┌─────────────────────────────────────────────────────┐
│              Astro Server (:4321)                    │
│  Content: Markdown files, assets, config            │
│  Output: Static HTML → GitHub Pages / CF Pages      │
└─────────────────────────────────────────────────────┘
```

All services run in Docker containers orchestrated by `compose.yaml` with profiles. The `astro-sites` volume is shared between the backend and the Astro server. Tests run on the host (not inside containers) for fast TDD feedback.

### Port Scheme

| Service            | Port | Notes               |
|--------------------|------|---------------------|
| Rust backend       | 8080 | Sweet spot backend  |
| Node backend       | 8081 | Alternative backend |
| SvelteKit frontend | 3000 | Sweet spot frontend |
| React frontend     | 3001 | Alternative frontend|
| Astro server       | 4321 | Static site preview |

### Key Tech Choices

| Component       | Choice                     | Why                                          |
|-----------------|----------------------------|----------------------------------------------|
| API spec        | OpenAPI 3.1 (code-first)   | Rust generates spec → frontends consume types|
| Error format    | RFC 9457 Problem Details   | Industry standard                            |
| Response format | `{ data, meta }` envelope  | Simple, room for metadata                    |
| Database        | None (Git + filesystem)    | Git-native, no lock-in                       |
| Containers      | Docker + Compose           | Alpine Linux base, profiles for variants     |
| Code generation | utoipa → orval             | Rust generates OpenAPI YAML, orval generates svelte-query and react-query clients |

---

## What Has Been Built (Phase 0 — Complete)

Phase 0's goal was: all containers start, communicate, and Playwright validates. Every step below is done.

Steps 1–9 are fully complete. Step 1 created the folder structure. Step 2 set up Playwright integration tests with the initial failing smoke test (RED phase). Step 3 defined the OpenAPI contract with the `/healthz` endpoint, envelope schema, and RFC 9457 error schema. Step 4 built the base `compose.yaml` with the Astro server container (Alpine + Node + pnpm). Steps 5 and 6 implemented the health endpoint in both backends — Node/Fastify and Rust/Axum respectively — each with its own Dockerfile and compose service. Steps 7 and 8 created both frontends (SvelteKit and React Router v7, both with DaisyUI) that display connection status by calling the backend health endpoint. Step 9 ran all four frontend/backend combinations through smoke tests in both dev and production modes, with mise tasks for each combo.

**What's implemented in each component right now:**

The **Rust backend** has a single `GET /healthz` endpoint returning a `{ data, meta }` envelope. It uses `utoipa-axum` for code-first OpenAPI generation and exposes Swagger UI at `/swagger-ui`. A separate binary (`gen_open_api_yaml`) exports the spec to `open-api-contracts/api.yaml`. The meta object includes `timestamp`, `requestId`, `version`, and `serverName`.

The **Node backend** mirrors the same health endpoint using Fastify with `fastify-openapi-glue` for contract-driven routing and `openapi-typescript` for generated types.

Both **frontends** display a "Blog Engine Admin" heading and a footer that calls the health endpoint, shows the backend URL, server name, version, and a Connected/Disconnected indicator. They use generated query clients (svelte-query and react-query) produced by orval from the shared OpenAPI spec.

The **Astro server** is a minimal container with Node 25 Alpine, pnpm installed, and a `/app/sites` directory ready. It currently runs `tail -f /dev/null` (idle) — no Astro project has been scaffolded yet.

The **integration tests** verify that the frontend loads (HTTP 200), the backend health endpoint returns the correct envelope, the frontend footer shows the backend URL, and the frontend displays "Connected" status.

---

## What Comes Next

### Step 10: Add Server Info to Meta (Small)

Add the `serverName` field to the meta response. The OpenAPI spec and generated types already include it (added during the utoipa migration), but integration tests need updating to verify it. Tasks: update OpenAPI spec if needed, update unit tests, update integration tests, confirm health response includes backend name.

### Step 11: Create Astro Site (The First Real Feature)

This is the first feature that makes the CMS actually do something. The full plan follows below.

---

## Step 11 — Detailed Plan

### Core Principle: Environment Parity

The local development environment should behave the same as production. There is **one code path**, not two. The system always authenticates, always creates an Astro project, always initializes a Git repo, always commits, and always pushes to a remote. The only things that change between environments are configuration values: which auth provider to use and where the git remote points.

This means the difference between "local" and "production" is a small config change, not a code branch. Integration tests exercise the same flow that runs in production. CI/CD runs the same flow. A demo on localhost looks and behaves like the real thing.

### Provider Abstraction

Instead of conditional logic like "if production, do GitHub stuff," the system uses swappable providers configured by environment variables.

**Auth Provider** controls how users authenticate.

In the MVP (`AUTH_PROVIDER=dev`), a dev auth provider automatically logs you in as a local dev user. No OAuth redirect, no external service — the backend returns a hardcoded dev user session when you hit the auth endpoints. The frontend sees the exact same API shape it would see with real OAuth: `GET /auth/login` redirects to the provider, `GET /auth/callback` exchanges a code for a session, `GET /auth/me` returns the current user. With the dev provider, the "redirect" just bounces back immediately and the "exchange" always succeeds.

In production (`AUTH_PROVIDER=github`), the same endpoints perform real GitHub OAuth. The frontend code is identical — it doesn't know or care which provider is behind the API.

**Git Provider** controls where repositories are created and pushed.

In the MVP (`GIT_PROVIDER=local`), a local bare Git repository on a Docker volume acts as the remote. When a site is created, the system runs `git init`, commits the scaffolded Astro project, creates a bare repo at a known path, and pushes to it. This exercises the complete git workflow — init, add, commit, push — without any network dependency. The bare repos live on a volume, so you can inspect them, and integration tests can verify that commits and pushes actually happened.

In production (`GIT_PROVIDER=github`), the same operations target GitHub instead. The system uses the authenticated user's token to create a repo via the GitHub API, then pushes to `github.com`. The git commands are identical; only the remote URL and the repo creation API call change.

```
┌──────────────┐       ┌──────────────────┐       ┌──────────────────────┐
│   Frontend   │──────▶│     Backend      │──────▶│   Astro Server       │
│              │       │                  │       │   Management API     │
│  Login btn   │       │  Auth Provider:  │       │                      │
│  Create btn  │       │   dev | github   │       │  Scaffold Astro proj │
│  Site list   │       │                  │       │  git init + commit   │
│  Preview     │       │  Git Provider:   │       │  git push to remote  │
│              │       │   local | github │       │                      │
└──────────────┘       └──────────────────┘       └──────────────────────┘
                                                           │
                                                           ▼
                                              ┌────────────────────────┐
                                              │   Git Remote           │
                                              │                        │
                                              │   local: bare repo     │
                                              │          on volume     │
                                              │                        │
                                              │   prod:  github.com    │
                                              └────────────────────────┘
```

### Configuration

```bash
# Environment variables (set in compose.yaml or .env)

# Auth: "dev" (auto-login, no external service) or "github" (real OAuth)
AUTH_PROVIDER=dev

# Git: "local" (bare repos on volume) or "github" (GitHub API + push)
GIT_PROVIDER=local

# Only needed when AUTH_PROVIDER=github and GIT_PROVIDER=github
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...

# Internal: how the backend reaches the astro-server management API
ASTRO_MANAGEMENT_URL=http://astro-server:4320
```

The `compose.yaml` (dev) defaults to `AUTH_PROVIDER=dev` and `GIT_PROVIDER=local`. The `compose.prod.yaml` sets `AUTH_PROVIDER=github` and `GIT_PROVIDER=github` with the real credentials.

### The Create Site Flow (Same in Every Environment)

```
User → clicks "Login" (or auto-redirected)
  → GET /auth/login
  → Provider handles auth (dev: instant, github: OAuth redirect)
  → GET /auth/callback → session cookie set
  → Frontend shows user info + "Create a new blog" button

User → clicks "Create a new blog"
  → Modal/form: enter blog name → slug auto-generated
  → POST /sites { name, slug }
  → Backend validates, forwards to Astro Server management API:
      1. npm create astro@latest /app/astro-sites/{slug} --template minimal --no-git --skip-houston
      2. pnpm install
      3. git init && git add . && git commit -m "Initial Astro site: {name}"
      4. Git Provider creates remote repo (local: mkdir bare repo, github: API call)
      5. git remote add origin {remote_url} && git push -u origin main
  → 201 { data: { id, name, slug, gitUrl }, meta }

User → sees site in list
  → Can click to preview (Astro dev server at :4321)
  → Can see git history (commits visible in bare repo or GitHub)
```

Every step in this flow executes in every environment. Integration tests verify the entire chain including the git operations.

### New API Endpoints

**Auth (always present, provider determines behavior):**

| Method | Path             | Purpose                              |
|--------|------------------|--------------------------------------|
| GET    | `/auth/login`    | Redirect to auth provider            |
| GET    | `/auth/callback` | Exchange code for session            |
| GET    | `/auth/me`       | Return current user (or 401)         |
| POST   | `/auth/logout`   | Clear session                        |

**Sites:**

| Method | Path             | Purpose                              |
|--------|------------------|--------------------------------------|
| POST   | `/sites`         | Create a new blog site               |
| GET    | `/sites`         | List all sites                       |
| GET    | `/sites/:slug`   | Get site details (incl. git info)    |

### Astro Server Management API

A new lightweight HTTP API running inside the Astro server container on an internal port (4320, not exposed to host). This is the service that actually scaffolds projects and runs git commands, since it has Node, pnpm, and git available.

**File:** `astro-server/management-api.mjs`

`POST /sites { name, slug, git_remote_url }` scaffolds a new Astro project at `/app/astro-sites/{slug}`, installs dependencies, initializes a git repo, commits the initial files, adds the provided remote URL, and pushes. The `git_remote_url` is determined by the backend based on the active Git Provider — the management API doesn't know or care whether it's a local path or a GitHub URL.

`GET /sites` reads the `/app/astro-sites/` directory and returns the list of sites with basic info (name from package.json, slug from directory name, git remote URL from git config).

`POST /sites/:slug/build` triggers an Astro build for a specific site (future, but the endpoint shape is planned now).

`POST /sites/:slug/preview` starts the Astro dev server for a specific site on port 4321 (future).

### Docker Compose Changes

The Astro server Dockerfile needs `git` installed (`apk add git`). A `git-repos` volume holds the local bare repositories when using the local Git Provider. The management API runs on port 4320 (internal network only). The `compose.yaml` defaults both providers to their local/dev variants.

```yaml
# Additions to compose.yaml
volumes:
  astro-sites:      # existing
  git-repos:        # bare repos for local git provider

services:
  astro-server:
    # ... existing config ...
    volumes:
      - astro-sites:/app/astro-sites
      - git-repos:/app/git-repos    # local bare repo storage
    environment:
      - MANAGEMENT_PORT=4320

  backend-rust:
    # ... existing config ...
    environment:
      - AUTH_PROVIDER=dev
      - GIT_PROVIDER=local
      - ASTRO_MANAGEMENT_URL=http://astro-server:4320
```

### Session Management

Signed cookie (`tower-cookies` in Rust) storing `{ user_id, user_name, provider }`. Stateless — no server-side store. The dev auth provider sets this cookie with hardcoded dev user values. The GitHub auth provider sets it with the real GitHub user info after OAuth.

### Frontend Changes

The frontend always shows the same UI flow regardless of environment. On load, it checks `GET /auth/me`. If not authenticated, it shows "Login" (which with the dev provider resolves instantly). Once authenticated, it shows the user name and a "Create a new blog" button. The create flow opens a form, submits to `POST /sites`, and on success displays the site with its git URL. The git URL will be a local path in dev and a GitHub URL in production, but the UI just displays whatever the backend returns.

### New Dependencies

**Rust backend:** `reqwest` (with json feature) for calling astro-server management API and GitHub API. `tower-cookies` for session cookies.

**Astro server Dockerfile:** `git` (`apk add git`). Optionally `express` or plain Node `http` module for the management API.

---

## Implementation Order (TDD)

The development follows strict TDD: write the failing test first (RED), then implement just enough to pass (GREEN), then refactor. Since the system uses providers, we build everything with the local/dev providers first — this is the full feature, not a subset.

### Phase A — Auth with Dev Provider

1. Write integration test: page shows "Login" button when not authenticated.
2. Write integration test: after login flow, page shows user name.
3. Implement dev auth provider in Rust backend (`GET /auth/login`, `/auth/callback`, `/auth/me`).
4. Add `tower-cookies` session middleware.
5. Update OpenAPI spec, run `mise dev-gen-api-clients`.
6. Implement dev auth in Node backend.
7. Build Svelte frontend: login button, auth check on load, user display.
8. Build React frontend: same.
9. Integration tests pass: user can log in and see their name.

### Phase B — Create Site with Local Git

1. Write integration test: authenticated user sees "Create a new blog" button.
2. Write integration test: submitting the form creates a site (verify via `GET /sites`).
3. Write integration test: created site has a git repository with at least one commit.
4. Build the Astro server management API (`POST /sites`, `GET /sites`).
5. Test management API directly with curl (scaffold + git init + commit + push to local bare repo).
6. Add `POST /sites` and `GET /sites` to Rust backend (calls astro-server management API, passes local git remote path).
7. Update OpenAPI spec, run `mise dev-gen-api-clients`.
8. Implement Node backend site handlers.
9. Build Svelte frontend: create button, modal form, site list, git info display.
10. Build React frontend: same.
11. All integration tests pass — full flow works with local git.

### Phase C — Astro Preview

1. Write integration test: clicking "Preview" on a site shows the Astro dev server output.
2. Extend management API with `POST /sites/:slug/preview` (starts Astro dev server).
3. Wire up the frontend preview button.
4. Integration test verifies the preview page loads.

### Phase D — GitHub Provider (Post-MVP)

When ready to add real GitHub support, the work is mostly implementing the second provider variant — the API shape, frontend, and tests already exist from the dev/local providers.

1. Implement GitHub auth provider (real OAuth flow) alongside the existing dev provider.
2. Implement GitHub git provider (create repo via API, push to github.com) alongside the existing local provider.
3. Switch `compose.prod.yaml` to use `AUTH_PROVIDER=github` and `GIT_PROVIDER=github`.
4. Add integration tests that mock GitHub API responses to verify the provider works (no real GitHub account needed in CI).
5. Manual testing with real GitHub account in a staging environment.

---

## Coding Conventions

**Test-driven development** with 100% coverage. Break everything into small steps and implement test-first. Unit tests use each language's native testing (Rust's built-in tests, vitest for JS/TS). Integration tests use Playwright and run outside Docker on the host for fast feedback.

**Declarative and functional style preferred.** Composition over inheritance. Pure functions over side effects.

**Rust conventions:** No `get_` prefix on getters (`fn name(&self)` not `fn get_name(&self)`). Mutable getter: `fn name_mut(&mut self)`. Enums over booleans. Newtypes over primitives. `Result<T, E>` for errors, `?` operator, custom error types with `thiserror`. `#[warn(clippy::all, clippy::pedantic, clippy::nursery)]`. Implement `IntoResponse` for custom response types. Use extractors (State, Json, Path) idiomatically. Tower middleware for cross-cutting concerns.

**API format:** All responses use the `{ data, meta }` envelope. Errors follow RFC 9457. The OpenAPI spec is generated from Rust code (utoipa) and exported to `open-api-contracts/api.yaml`. Frontend types are generated from this spec using orval.

**TDD workflow across four terminals:** Terminal 1 runs `mise run up` (services with hot reload). Terminal 2 runs Playwright integration tests in watch mode. Terminal 3 runs `cargo watch -x test` for Rust unit tests. Terminal 4 runs `vitest --watch` for JS/TS unit tests.

---

## Project Structure

```
/blog-engine-prototype
├── open-api-contracts/
│   └── api.yaml                 # OpenAPI 3.1 (generated from Rust)
│
├── admin-cms-app/
│   ├── frontend-svelte/         # SvelteKit + DaisyUI + svelte-query
│   ├── frontend-react/          # React Router v7 + DaisyUI + react-query
│   ├── backend-node/            # Fastify + TypeScript + openapi-glue
│   └── backend-rust/            # Axum + utoipa + swagger-ui
│
├── astro-server/                # Astro runtime container (Alpine + Node + pnpm + git)
│   ├── Dockerfile
│   └── management-api.mjs       # Internal API for site operations (NEW)
│
├── integration-tests/           # Playwright tests
│   └── tests/
│       ├── smoke.spec.ts        # Existing: services respond
│       ├── auth.spec.ts         # NEW: login/logout flow
│       └── sites.spec.ts        # NEW: create site, verify git
│
├── compose.yaml                 # Dev compose (dev auth + local git)
├── compose.prod.yaml            # Prod compose (github auth + github git)
├── mise.toml                    # Task runner (mise run <task>)
├── ARCHITECTURE.md              # Design decisions, benchmarks
├── CONTEXT.md                   # This file
└── README.md                    # Quick start
```

---

## Decisions Log

| Date       | Decision                                    | Context                                                              |
|------------|---------------------------------------------|----------------------------------------------------------------------|
| 2025-12-19 | OpenAPI 3.1 for contract                    | Shared types between TS and Rust                                     |
| 2025-12-19 | RFC 9457 for errors                         | Industry standard                                                    |
| 2025-12-19 | Simple envelope `{ data, meta }`            | Easy serialization                                                   |
| 2025-12-19 | Filesystem as source of truth               | No database needed                                                   |
| 2025-12-19 | Docker + `docker compose`                   | Containerized services                                               |
| 2025-12-19 | Alpine Linux base                           | Minimal resources                                                    |
| 2025-12-19 | SvelteKit + Rust sweet spot                 | Preferred combination                                                |
| 2025-12-21 | `/healthz` endpoint                         | K8s compatible health checks                                         |
| 2025-12-23 | No default Astro project                    | Sites created dynamically by admin                                   |
| 2025-12-23 | Single active site model                    | One dev server at a time                                             |
| 2025-12-25 | Debian slim for Rust, not Alpine            | Faster glibc builds; musl is slow for Rust                           |
| 2025-12-29 | SPA mode for frontends                      | No SSR; admin UI doesn't need SEO                                    |
| 2026-01-01 | Compose profiles, not multiple files        | Single `compose.yaml` with `--profile` flag                          |
| 2026-01-03 | CORS permissive in dev                      | `CorsLayer::new().allow_origin(Any)` for local dev                   |
| 2026-01-03 | React Router v7 for React frontend          | Matches SvelteKit scope; configured as SPA                           |
| 2026-01-05 | VITE_API_BACKEND_URL for frontends          | Unified env var; Vite requires VITE_ prefix                          |
| 2026-01-06 | Build-time env vars for Vite prod           | VITE_* must be ARG in Dockerfile.prod                                |
| 2026-01-06 | Separate compose.prod.yaml                  | Uses Dockerfile.prod, no volume mounts                               |
| 2026-01-06 | Fixed prod pairings                         | Svelte↔Rust (8080), React↔Node (8081)                               |
| 2026-01-18 | One GitHub repo per site                    | Needed for GitHub Pages custom domains                               |
| 2026-01-18 | Signed cookie for session (MVP)             | Stateless; no server-side store; `tower-cookies` in Rust             |
| 2026-03-14 | Environment parity via providers            | One code path; local/prod differ only by config, not code branches   |
| 2026-03-14 | Dev auth provider for local/test            | Same API shape as GitHub OAuth; auto-login, no external dependency   |
| 2026-03-14 | Local git provider with bare repos          | All git ops (init, commit, push) run locally; same flow as prod      |
| 2026-03-14 | MVP includes full git, not full GitHub      | Git operations always execute; only the remote target is deferred    |

---

## Resource Benchmarks

From Phase 0 testing with minimal `/healthz` endpoints:

| Container         | Image Size | Memory Usage |
|-------------------|------------|--------------|
| backend-rust-prod | 122 MB     | 0.75 MB      |
| backend-node-prod | 255 MB     | 26.5 MB      |

Rust is roughly 2x smaller in image size and uses 35x less RAM.

---

## Future Considerations (Post-MVP)

**GitHub provider implementation:** Add `AUTH_PROVIDER=github` and `GIT_PROVIDER=github` variants. The API shape, frontend, and integration test structure already exist from the dev/local providers — this is primarily implementing the second variant of each trait/interface.

**Content editing:** Once a site exists, the CMS should allow creating/editing Markdown posts through the admin UI. Each edit becomes a git commit. Users can also edit files directly in the repo — both workflows coexist.

**Astro builds:** Trigger Astro builds from the admin UI. Locally, this runs `astro build` inside the container. In production, this could trigger GitHub Actions via `repository_dispatch`.

**Quadlet production deployment:** Migrate from `docker compose` to Quadlet (systemd-native, Podman-specific) for production with auto-updates, journald logging, and native boot startup. Images pushed to ghcr.io.

**Gitea as local provider upgrade:** For a richer local dev experience, replace bare repos with a Gitea container. This gives a GitHub-like web UI for browsing repos locally, and Gitea supports OAuth2 — making the local environment even closer to production. The provider abstraction means this is a drop-in replacement.
