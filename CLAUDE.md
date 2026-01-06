@AGENTS.md
@README.md
@ARCHITECTURE.md

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

### Step 7: Frontend - SvelteKit ✅
- [x] Initialize SvelteKit + DaisyUI in `admin-cms-app/frontend-svelte/`
- [x] Create basic page with health status display
- [x] Create Dockerfile (Alpine + Node)
- [x] Add service to compose.yaml with profile
- [x] Test: page loads (CORS enabled, frontend ↔ backend communication works)

### Step 8: Frontend - React ✅
- [x] Initialize React Router v7 (SPA mode) + DaisyUI in `admin-cms-app/frontend-react/`
- [x] Create basic page with health status display
- [x] Create Dockerfile (Alpine + Node)
- [x] Add service to compose.yaml with profile
- [x] Test: page loads

### Step 9: Full Stack Smoke Test ✅
- [x] Run all 4 dev combinations (Svelte+Rust, Svelte+Node, React+Rust, React+Node)
- [x] All Playwright tests pass (dev)
- [x] Add mise tasks for combination testing (`test-svelte-rust`, `test-all-combos`, etc.)
- [x] Fix compose.prod.yaml to use Dockerfile.prod
- [x] Add build args for VITE_API_BACKEND_URL in frontend Dockerfile.prod files
- [x] Add mise tasks for production (`up-prod`, `up-prod-svelte`, etc.)
- [x] Production Svelte+Rust passes all smoke tests
- [x] Production React+Node passes all smoke tests
- [x] Documented: Prod frontends have backend URLs baked in at build time (Vite limitation)
  - Svelte → Rust (8080) is the intended prod pairing
  - React → Node (8081) is the intended prod pairing
  - Cross-pairing (e.g. Svelte+Node) requires rebuilding the frontend image

### Step 10: First Feature — Create Astro Site ⬜
- [ ] Set up OpenAPI → TypeScript type generation
- [ ] Write integration test: click button → site created
- [ ] Add API endpoint: `POST /api/sites`
- [ ] Add UI button: "Create Astro Blog"
- [ ] Backend executes `npm create astro`
- [ ] Test passes

---

## Decisions Log

| Date       | Decision                             | Context                                                                                                      |
|------------|--------------------------------------|--------------------------------------------------------------------------------------------------------------|
| 2025-12-19 | OpenAPI 3.1 for contract             | Shared types between TS and Rust                                                                             |
| 2025-12-19 | RFC 7807 for errors                  | Industry standard                                                                                            |
| 2025-12-19 | Simple envelope `{ data, meta }`     | Easy serialization                                                                                           |
| 2025-12-19 | Filesystem as source of truth        | No database needed                                                                                           |
| 2025-12-19 | Podman + `podman compose`            | Rootless containers (not podman-compose)                                                                     |
| 2025-12-19 | Alpine Linux base                    | Minimal resources                                                                                            |
| 2025-12-19 | SvelteKit + Rust sweet spot          | Preferred combination                                                                                        |
| 2025-12-21 | `/healthz` endpoint                  | K8s compatible health checks                                                                                 |
| 2025-12-23 | No default Astro project             | Sites created dynamically by admin                                                                           |
| 2025-12-23 | Single active site model             | One dev server at a time, switch on demand                                                                   |
| 2025-12-23 | Container-level healthcheck          | Simple node command, no HTTP server needed                                                                   |
| 2025-12-25 | Debian slim for Rust, not Alpine     | Faster glibc builds; Alpine musl is slow for Rust                                                            |
| 2025-12-29 | SPA mode for frontends               | No SSR needed; admin UI doesn't need SEO; simpler architecture                                               |
| 2025-12-29 | adapter-static for SvelteKit         | SPA with client-side routing via fallback: 'index.html'                                                      |
| 2025-12-31 | mise for task running                | Unified commands via `mise run <task>`; see `mise tasks` for list                                            |
| 2025-12-31 | ~~compose.all.yaml for comparison~~  | ~~Run all backends simultaneously~~ (superseded)                                                             |
| 2026-01-01 | Compose profiles, not multiple files | Single compose.yaml with `--profile` flag instead of `-f file1 -f file2`                                     |
| 2025-12-31 | inlineSources in tsconfig            | Source maps embed TypeScript for debugging without shipping src/                                             |
| 2025-12-31 | Multi-stage Dockerfiles              | Prod images only contain compiled artifacts, not dev dependencies                                            |
| 2025-12-31 | Tests run outside compose            | Playwright + unit tests run on host for fast TDD feedback                                                    |
| 2025-12-31 | vitest for JS/TS unit tests          | Fast, Vite-native, watch mode; cargo-watch for Rust                                                          |
| 2026-01-03 | CORS permissive in dev               | `CorsLayer::new().allow_origin(Any)` for local development                                                   |
| 2026-01-03 | DaisyUI handles form styling         | Removed `@tailwindcss/forms` — DaisyUI already provides form components                                      |
| 2026-01-03 | Consolidated compose files           | Deleted compose.all.yaml, compose.backend-*.yaml, compose.frontend-*.yaml; single compose.yaml with profiles |
| 2026-01-03 | React Router v7 for React Frontend   | Matches SvelteKit framework scope; configured as SPA (Client Data)                                           |
| 2026-01-03 | Nginx for SvelteKit Prod             | Using `nginx:alpine` to serve static files in production for better performance                              |
| 2026-01-05 | VITE_API_BACKEND_URL for frontends   | Unified env var across React and SvelteKit; Vite requires VITE_ prefix by default                           |
| 2026-01-06 | Build-time env vars for Vite prod    | VITE_* must be passed as ARG in Dockerfile.prod; Vite embeds values at build time                           |
| 2026-01-06 | Separate compose.prod.yaml           | Uses Dockerfile.prod, no volume mounts; port mapping 3000:80 keeps same external ports as dev               |
| 2026-01-06 | Smoke test verifies "Connected"      | Proves frontend actually reaches backend; not just HTTP 200, but actual API communication                   |
| 2026-01-06 | Fixed prod pairings                  | Svelte↔Rust (8080), React↔Node (8081); cross-pairing in prod requires frontend rebuild                     |

---

## Notes

- **Always read first:** At the start of each session, read `AGENTS.md`, `README.md`, and `ARCHITECTURE.md` for context.
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
- **Svelte component testing:** Mock `$env/dynamic/public` with `vi.mock()` before importing components. Use `it()` for test cases (not `describe()` — its body runs immediately before `beforeEach`).

---

## Commands Reference

```bash
# List all available tasks
mise tasks

# Start services (using compose profiles)
mise run up              # All: Rust:8080, Node:8081, Svelte:3000, Astro:4321
mise run up-svelte       # Sweet spot: Rust + Svelte (recommended)
mise run up-rust         # Rust backend only
mise run up-node         # Node backend only

# Stop services
mise run down

# Health checks
mise run health          # Check all endpoints
mise run health-rust     # Check Rust only
mise run health-node     # Check Node only
mise run health-svelte   # Check Svelte frontend

# Integration tests (Playwright - runs outside compose)
mise run test            # Run once
mise run test-ui         # TDD mode with UI (watch)
mise run test-svelte-rust  # E2E: Svelte + Rust
mise run test-react-node   # E2E: React + Node
mise run test-all-combos   # E2E: All 4 combinations

# Unit tests (watch mode - run in separate terminals)
mise run test-unit-rust    # Rust backend (cargo watch)
mise run test-unit-node    # Node backend (vitest)
mise run test-unit-svelte  # SvelteKit frontend (vitest)
mise run test-unit-react   # React frontend (vitest)
mise run test-unit         # Run all unit tests once

# Container stats
mise run stats           # Image sizes and memory usage

# Production compose stack
mise run prod-up         # All prod services (builds images)
mise run prod-down       # Stop prod stack
```

## TDD Workflow (Multiple Terminals)

```
Terminal 1: mise run up              # Services with hot reload
Terminal 2: mise run test-ui         # Playwright watch (integration)
Terminal 3: mise run test-unit-rust  # Rust unit tests watch
Terminal 4: mise run test-unit-node  # Node unit tests watch
```

## Available MCP Tools for Svelte:

You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

### 1. list-sections

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

### 2. get-documentation

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

### 3. svelte-autofixer

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

### 4. playground-link

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.