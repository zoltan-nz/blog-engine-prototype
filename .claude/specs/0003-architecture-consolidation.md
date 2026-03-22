# Architecture Consolidation

**Date:** 2026-03-21
**Status:** Approved

## Overview

During the blog-engine-agent design work (Step 11), two architectural questions surfaced that required explicit
decisions: whether to continue maintaining two frontend/backend stacks, and how the CMS should communicate with the
Astro site infrastructure long-term. This spec documents both decisions and establishes the system's target architecture.

## Decision 1: Single Stack — Rust + Svelte

**Drop `backend-node` (Fastify/TypeScript) and `frontend-react` from active development.** Keep only `backend-rust`
(Axum) + `frontend-svelte` (SvelteKit).

### Rationale

The "2 combos must work" requirement (Svelte+Rust, React+Node) served its purpose: it validated that the OpenAPI
contract and code-generation pipeline (`utoipa → api.yaml → orval → generated clients`) work correctly across stacks.
Steps 1–10 proved this — both combos pass identical Playwright integration tests against the same API surface.

Continuing to maintain four services (2 backends × 2 frontends) quadruples the implementation cost of every feature. The
resource benchmarks already show the Rust advantage:

| Container | Image Size | Memory |
|-----------|------------|--------|
| backend-rust-prod | 122 MB | 0.75 MB |
| backend-node-prod | 255 MB | 26.5 MB |

Rust is ~2× smaller image, ~35× less RAM. The system is heading toward a single portable binary — maintaining a second
stack delays that goal without adding value.

### Impact

- Remove from `compose.yaml`: `backend-node`, `frontend-react` services and their profiles
- Remove from `mise.toml`: `test-react-node`, `test-unit-node`, `test-unit-react` and cross-combo tasks
- Integration tests run only against Svelte + Rust
- Keep the React/Node code in the repo (archived, not deleted) — it proves the API contract is stack-agnostic
- The OpenAPI contract remains the same — if someone wanted to build a React frontend later, the generated client would
  still work

## Decision 2: Agent Communication — HTTP Now, WebSocket Future

**The blog-engine-agent keeps its HTTP REST interface for same-machine / Docker compose deployment.** The evolution path
to WebSocket for remote deployment is documented here but not implemented until needed.

### Rationale

The agent's operations are request/response by nature: "scaffold this site" → "done or failed", "start preview" →
"here is the URL". HTTP over Docker's internal network is internal IPC — the transport being HTTP doesn't make it a
public API.

A CQRS event bus (NATS, Redis Streams, filesystem watchers) would add infrastructure complexity without solving a
problem that exists at the current scale. The concurrency model (`tokio::sync::Mutex` serialising preview operations)
already matches the `.mjs` behaviour.

The HTTP routes map directly to future WebSocket commands:

| HTTP Route | WebSocket Command |
|---|---|
| `GET /healthz` | `HEALTHZ` |
| `GET /sites` | `LIST_SITES` |
| `POST /sites` | `CREATE_SITE` |
| `POST /sites/:slug/preview` | `START_PREVIEW` |
| `DELETE /preview` | `STOP_PREVIEW` |

When the transport changes, the handler logic stays the same — only the dispatch layer changes.

## Decision 3: Agent Scope — Process Management Only

**The agent handles scaffolding, manifest management, and preview server lifecycle. Content management (posts, images,
templates) never goes through the agent.**

### Rationale

Content operations are filesystem reads and writes against Astro site directories — markdown files, JSON configs, image
assets. The CMS backend can perform these directly when it has volume access. Routing every file read/write through the
agent would create an unnecessarily large API surface and make the agent a bottleneck.

The separation:

| Concern | Owner | How |
|---|---|---|
| Site scaffolding (`pnpm create astro`) | Agent | HTTP API (internal) |
| Git operations (`git init`, `commit`, `push`) | Agent | HTTP API (internal) |
| Preview server (`pnpm dev`) | Agent | HTTP API (internal) |
| Site manifest (`sites.json`) | Agent | Filesystem |
| Content CRUD (posts, images, templates) | CMS backend | Direct filesystem access |

For the Docker compose setup, this means the `astro-sites` and `git-repos` volumes will eventually be mounted to the CMS
backend container as well (read/write for content, read-only is sufficient for listing).

## Decision 4: Portable CMS Binary — Long-Term Vision

**`backend-rust` + embedded SPA (via `rust-embed`) becomes a single downloadable binary.** Users download it, point it
at their Astro project directory, and manage content through the browser.

### Two Deployment Models

```
A. Same machine (local dev, Docker compose — current):

   CMS Binary ──HTTP──► Agent (same host / Docker network)
                              │ filesystem
                         astro-sites/  git-repos/

B. Remote (cloud CMS, local Astro — future):

   CMS Binary (cloud) ◄──WebSocket── Agent (user's machine, initiates outbound)
                                           │ filesystem
                                      ~/my-astro-project/
```

### Rationale

- **Single binary** = easy distribution, no Docker required for end users
- **Agent connects outbound** = works through NAT and firewalls (the user's local machine can't accept inbound
  connections from the cloud)
- **Git is the content transport** for the remote case — CMS commits content changes, the Astro project pulls from git.
  This is the standard Jamstack pattern (CMS → git → CI/CD → static site)
- The Docker compose setup is the **development environment** for building toward this target. It simulates the
  same-machine deployment model using containers

### What This Means for Current Work

Nothing changes for Step 11. The current architecture (agent as HTTP server in `astro-server` container, backend calling
it via `ASTRO_MANAGEMENT_URL`) is the correct implementation of deployment model A. The WebSocket evolution (model B) is
a future milestone, not current work.

## Migration Steps

1. Create this spec (0003) — documenting the decisions
2. Update `CLAUDE.md` — single stack references, remove React/Node from tech table
3. Update `ARCHITECTURE.md` — remove React/Node from diagrams, add deployment models
4. Continue Step 11 (blog-engine-agent Milestone 1) — unchanged by these decisions
5. Future: remove React/Node services from `compose.yaml` and `mise.toml`
6. Future: mount content volumes to CMS backend for direct content access
7. Future: `rust-embed` for SPA embedding in the CMS binary
8. Future: WebSocket agent for remote deployment model
