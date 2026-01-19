# Notes And Plans

## Phase 0 — Scaffold & Smoke Test

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

### Step 10: Add Server Info To Meta

Add `serverName` field to the `meta` object.

- [ ] Update OpenAPI specification with `serverName` field
- [ ] Update unit tests
- [ ] Update the integration tests
- [ ] Update health response with the name of the backend

### Step 11: First Feature — Create Astro Site ⬜

- [ ] Set up OpenAPI → TypeScript type generation
- [ ] Write integration test: click button → site created
- [ ] Add API endpoint: `POST /api/sites`
- [ ] Add UI button: "Create Astro Blog"
- [ ] Backend executes `npm create astro`
- [ ] Test passes

## Future: Quadlet Production Deployment

**Summary:** Migrate from `podman compose` (dev-oriented) to Quadlet (production-native) for deployment:

- Quadlet = systemd generator that converts `.container` files → systemd services
- Native boot startup, auto-restart, journald logging, auto-updates
- Images pushed to ghcr.io, pulled on production server
- Files stored in `quadlet/` folder (tracked in git, symlinked to `~/.config/containers/systemd/`)

**Key benefits over compose:**

| Feature             | Compose          | Quadlet                          |
|---------------------|------------------|----------------------------------|
| Boot startup        | Extra config     | Native `WantedBy=default.target` |
| Auto-updates        | Needs watchtower | Built-in `AutoUpdate=registry`   |
| Process supervision | Limited          | Full systemd                     |
| Logs                | `podman logs`    | `journalctl`                     |

**Files to create when implementing:**

- `scripts/build-push-images.sh` — Build & push to ghcr.io
- `quadlet/*.container` — One per service (5 total)
- `quadlet/blog.network` — Shared network
- `quadlet/astro-sites.volume` — Shared volume
- `quadlet/README.md` — Deployment instructions
- mise tasks for `build-push`, `quadlet-up`, `quadlet-down`, etc.
