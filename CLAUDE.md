# Project Plan

## Current Phase: Phase 0 — Scaffold & Smoke Test

**Goal:** All containers start and can communicate. Playwright validates.

---

## TODO

### Step 1: Project Structure ✅
- [x] Create folder structure
- [x] Initialize git tracking for new folders

### Step 2: First Integration Test (RED) ✅
- [x] Set up Playwright in `integration-tests/`
- [x] Write smoke test: all services respond
- [x] Test FAILS (nothing exists yet) — RED confirmed

### Step 3: API Contract ✅
- [x] Create `open-api-contracts/api.yaml`
- [x] Define `/health` endpoint
- [x] Define envelope schema `{ data, meta }`
- [x] Define RFC 7807 error schema

### Step 4: Compose & Astro Server ✅
- [x] Create base `compose.yaml` with shared network
- [x] Create Astro runtime in `astro-server/` (no default project)
- [x] Create Dockerfile (Alpine + Node + pnpm)
- [x] Add astro-server service to compose
- [x] Test: astro server container is healthy

### Step 5: Backend - Node ✅
- [x] Initialize Fastify + TypeScript in `admin-cms-app/backend-node/`
- [x] Implement `GET /healthz`
- [x] Create Dockerfile (Alpine + Node)
- [x] Create `compose.backend-node.yaml`
- [x] Test: health endpoint returns envelope

### Step 6: Backend - Rust ✅
- [x] Initialize Axum project in `admin-cms-app/backend-rust/`
- [x] Implement `GET /healthz`
- [x] Create Dockerfile (Debian slim + Rust)
- [x] Create `compose.backend-rust.yaml`
- [x] Test: health endpoint returns envelope

### Step 7: Frontend - SvelteKit ⬜
- [ ] Initialize SvelteKit + DaisyUI in `admin-cms-app/frontend-svelte/`
- [ ] Create basic page with health status display
- [ ] Create Dockerfile (Alpine + Node)
- [ ] Create `compose.frontend-svelte.yaml`
- [ ] Test: page loads

### Step 8: Frontend - React ⬜
- [ ] Initialize React + DaisyUI in `admin-cms-app/frontend-react/`
- [ ] Create basic page with health status display
- [ ] Create Dockerfile (Alpine + Node)
- [ ] Create `compose.frontend-react.yaml`
- [ ] Test: page loads

### Step 9: Full Stack Smoke Test ⬜
- [ ] Run all 4 combinations
- [ ] All Playwright tests pass
- [ ] Document any issues

### Step 10: First Feature — Create Astro Site ⬜
- [ ] Set up OpenAPI → TypeScript type generation
- [ ] Write integration test: click button → site created
- [ ] Add API endpoint: `POST /api/sites`
- [ ] Add UI button: "Create Astro Blog"
- [ ] Backend executes `npm create astro`
- [ ] Test passes

---

## Decisions Log

| Date | Decision | Context |
|------|----------|---------|
| 2025-12-19 | OpenAPI 3.1 for contract | Shared types between TS and Rust |
| 2025-12-19 | RFC 7807 for errors | Industry standard |
| 2025-12-19 | Simple envelope `{ data, meta }` | Easy serialization |
| 2025-12-19 | Filesystem as source of truth | No database needed |
| 2025-12-19 | Podman + `podman compose` | Rootless containers (not podman-compose) |
| 2025-12-19 | Alpine Linux base | Minimal resources |
| 2025-12-19 | SvelteKit + Rust sweet spot | Preferred combination |
| 2025-12-21 | `/healthz` endpoint | K8s compatible health checks |
| 2025-12-23 | No default Astro project | Sites created dynamically by admin |
| 2025-12-23 | Single active site model | One dev server at a time, switch on demand |
| 2025-12-23 | Container-level healthcheck | Simple node command, no HTTP server needed |
| 2025-12-25 | Debian slim for Rust, not Alpine | Faster glibc builds; Alpine musl is slow for Rust |
| 2025-12-29 | SPA mode for frontends | No SSR needed; admin UI doesn't need SEO; simpler architecture |
| 2025-12-29 | adapter-static for SvelteKit | SPA with client-side routing via fallback: 'index.html' |
| 2025-12-31 | mise for task running | Unified commands via `mise run <task>`; see `mise tasks` for list |
| 2025-12-31 | compose.all.yaml for comparison | Run all backends simultaneously (Rust:8080, Node:8081) |
| 2025-12-31 | inlineSources in tsconfig | Source maps embed TypeScript for debugging without shipping src/ |
| 2025-12-31 | Multi-stage Dockerfiles | Prod images only contain compiled artifacts, not dev dependencies |
| 2025-12-31 | Tests run outside compose | Playwright + unit tests run on host for fast TDD feedback |
| 2025-12-31 | vitest for JS/TS unit tests | Fast, Vite-native, watch mode; cargo-watch for Rust |

---

## Notes

- **Always read first:** At the start of each session, read `AGENTS.md`, `README.md`, and `NOTES.md` for context.
- **Task runner:** Use `mise tasks` to list available commands, `mise run <task>` to execute.
- **TDD approach:** Write test first (RED), then implement (GREEN)
- **Tiny steps:** Each task should be small and testable
- **All variants:** Build all 4 frontend/backend combinations
- **Sweet spot:** SvelteKit + Rust is the preferred combo
- **Folder structure:** Admin app variants live under `admin-cms-app/` (backend-node, backend-rust, frontend-svelte, frontend-react)
- **Node.js tsconfig:** Use `tsc --init` as baseline, then add `allowSyntheticDefaultImports: true` for CommonJS default imports
- **Alpine/musl DNS:** Alpine uses musl libc, which handles DNS differently than glibc. Use `127.0.0.1` instead of `localhost` inside containers. Services must bind to `0.0.0.0` to be reachable.
- **TDD workflow:** Run integration tests in watch mode during development. Keep the red/green feedback visible for motivation and gamification.
- **Rust Best Practices:** Added to `agents.md` for reuse. Key patterns: enums over strings, `impl IntoResponse`, `#[derive(Default)]`, structured logging with `tracing`.
- **Feature-driven development:** Implement to satisfy tests, not to mirror other implementations. Each language has its own idioms.

---

## Commands Reference

```bash
# List all available tasks
mise tasks

# Start all services (both backends for comparison)
mise run up              # Rust:8080, Node:8081, Astro:4321

# Start single backend stack
mise run up-rust         # Rust backend only
mise run up-node         # Node backend only

# Stop services
mise run down

# Health checks
mise run health          # Check all endpoints
mise run health-rust     # Check Rust only
mise run health-node     # Check Node only

# Integration tests (Playwright - runs outside compose)
mise run test            # Run once
mise run test-ui         # TDD mode with UI (watch)

# Unit tests (watch mode - run in separate terminals)
mise run test-unit-rust    # Rust backend (cargo watch)
mise run test-unit-node    # Node backend (vitest)
mise run test-unit-svelte  # SvelteKit frontend (vitest)
mise run test-unit-react   # React frontend (vitest)
mise run test-unit         # Run all unit tests once

# Container stats
mise run stats           # Image sizes and memory usage

# Standalone prod containers (quick testing)
mise run prod-rust       # Build & run Rust prod (8080)
mise run prod-node       # Build & run Node prod (8081)
mise run prod-both       # Both prod containers
mise run prod-stop       # Stop prod containers
```

## TDD Workflow (Multiple Terminals)

```
Terminal 1: mise run up              # Services with hot reload
Terminal 2: mise run test-ui         # Playwright watch (integration)
Terminal 3: mise run test-unit-rust  # Rust unit tests watch
Terminal 4: mise run test-unit-node  # Node unit tests watch
```
