# Project Plan

## Current Phase: Phase 0 — Scaffold & Smoke Test

**Goal:** All containers start and can communicate. Playwright validates.

---

## TODO

### Step 1: Project Structure ⬜
- [ ] Create folder structure
- [ ] Initialize git tracking for new folders

### Step 2: First Integration Test (RED) ⬜
- [ ] Set up Playwright in `tests/integration/`
- [ ] Write smoke test: all services respond
- [ ] Test should FAIL (nothing exists yet)

### Step 3: API Contract ⬜
- [ ] Create `contracts/api.yaml`
- [ ] Define `/health` endpoint
- [ ] Define envelope schema
- [ ] Define RFC 7807 error schema

### Step 4: Astro Server ⬜
- [ ] Initialize Astro project in `astro-server/`
- [ ] Create Dockerfile (Alpine + Node)
- [ ] Add to base `compose.yaml`
- [ ] Test: astro server responds

### Step 5: Backend - Node ⬜
- [ ] Initialize Fastify + TypeScript in `backend-node/`
- [ ] Implement `GET /health`
- [ ] Create Dockerfile (Alpine + Node)
- [ ] Create `compose.backend-node.yaml`
- [ ] Test: health endpoint returns envelope

### Step 6: Backend - Rust ⬜
- [ ] Initialize Axum project in `backend-rust/`
- [ ] Implement `GET /health`
- [ ] Create Dockerfile (Alpine + Rust)
- [ ] Create `compose.backend-rust.yaml`
- [ ] Test: health endpoint returns envelope

### Step 7: Frontend - SvelteKit ⬜
- [ ] Initialize SvelteKit + DaisyUI in `frontend-svelte/`
- [ ] Create basic page with health status display
- [ ] Create Dockerfile (Alpine + Node)
- [ ] Create `compose.frontend-svelte.yaml`
- [ ] Test: page loads

### Step 8: Frontend - React ⬜
- [ ] Initialize React + DaisyUI in `frontend-react/`
- [ ] Create basic page with health status display
- [ ] Create Dockerfile (Alpine + Node)
- [ ] Create `compose.frontend-react.yaml`
- [ ] Test: page loads

### Step 9: Full Stack Smoke Test ⬜
- [ ] Run all 4 combinations
- [ ] All Playwright tests pass
- [ ] Document any issues

### Step 10: First Feature — Create Astro Site ⬜
- [ ] Write integration test: click button → site created
- [ ] Add API endpoint: `POST /api/sites`
- [ ] Add UI button: "Create Astro Blog"
- [ ] Backend executes `npm create astro`
- [ ] Test passes

---

## Decisions Log

| Date | Decision | Context |
|------|----------|---------|
| 2024-12-19 | OpenAPI 3.1 for contract | Shared types between TS and Rust |
| 2024-12-19 | RFC 7807 for errors | Industry standard |
| 2024-12-19 | Simple envelope `{ data, meta }` | Easy serialization |
| 2024-12-19 | Filesystem as source of truth | No database needed |
| 2024-12-19 | Podman + compose | Rootless containers |
| 2024-12-19 | Alpine Linux base | Minimal resources |
| 2024-12-19 | SvelteKit + Rust sweet spot | Preferred combination |

---

## Notes

- **TDD approach:** Write test first (RED), then implement (GREEN)
- **Tiny steps:** Each task should be small and testable
- **All variants:** Build all 4 frontend/backend combinations
- **Sweet spot:** SvelteKit + Rust is the preferred combo

---

## Commands Reference

```bash
# Run sweet spot (SvelteKit + Rust)
podman-compose -f compose.yaml \
  -f compose.frontend-svelte.yaml \
  -f compose.backend-rust.yaml up

# Run Playwright tests
cd tests/integration && npx playwright test

# Generate types from OpenAPI
npm run generate:types
```
