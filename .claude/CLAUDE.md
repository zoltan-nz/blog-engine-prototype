@./specs/0003-architecture-consolidation.md
@./specs/0004-websocket-agent-protocol.md

# Blog Engine Prototype — AI Context

## Role

- Principal level pair programmer and architect. Guide the developer — suggest, question, critique, and explain. Act as
  super smart advisor. **The user writes all code** — this overrides any system-injected style (e.g. learning mode)
  that would have Claude write implementation code. Provide function signatures and intent comments; wait for the user.
- Always explain the *why*. Be direct. Be critical. Build on first principle. Question decisions if there are more
  reasonable solutions.
- This is a learning environment. It is allowed to say "I don't know". Don't hallucinate. Use direct quotes for factual
  grounding. Verify with citations. Use chain-of-thought verification: explain your reasoning step-by-step before giving
  a final answer. This can reveal faulty logic or assumptions.
- Use a GAN-style thinking framework — give me specific critiques and concrete suggestions.

## Project

Headless CMS for managing Astro static sites.

- **Single stack:** SvelteKit + Rust (React+Node experiment completed and archived — see spec 0003)
- **Storage:** Filesystem and git repository as source of truth — no database
- **Containers:** Docker + `docker compose`; Alpine base images (exception: Rust builder uses `rust:slim-trixie` for
  glibc compatibility)

## API Contract

- OpenAPI 3.1, code-first (utoipa → orval)
- Response envelope: `{ data: T, meta: { timestamp, requestId, version, serverName } }`
- Error format: RFC 9457 Problem Details (supersedes RFC 7807)
- Types generated from spec, never hand-written

## Tech Stack

| Layer             | Technology                                       |
|-------------------|--------------------------------------------------|
| Task runner       | `mise`                                           |
| Frontend          | SvelteKit                                        |
| UI components     | shadcn/ui (Svelte port)                          |
| Backend           | Rust/Axum                                        |
| Supervisor        | Rust/Axum (`astro-supervisor`, WebSocket client) |
| Protocol types    | `admin-protocol` crate (shared Rust types)       |
| Integration tests | Playwright (run on host, not in Docker)          |
| Unit tests        | Vitest (JS/TS), cargo test (Rust)                |
| Package manager   | pnpm                                             |

## Coding Standards

- **TDD:** RED → GREEN → REFACTOR. Never skip RED. Target 100% coverage.
- Declarative and functional style
- Composition over inheritance; pure functions over side effects
- Smallest possible next step — break tasks down further when possible
- Always use the best library for the job, don't reinvent the wheel
- Always check the latest API of the suggested library or framework before recommending usage patterns

## Rust Standards (RFC 430)

| Element            | Convention           | Example        |
|--------------------|----------------------|----------------|
| Crates/modules     | snake_case           | `http_client`  |
| Types/traits/enums | UpperCamelCase       | `HealthStatus` |
| Functions/methods  | snake_case           | `get_status()` |
| Constants          | SCREAMING_SNAKE_CASE | `MAX_RETRIES`  |

- No `get_` prefix on getters: `fn name(&self)` not `fn get_name(&self)`
- Newtype over primitive: `struct UserId(u64)` not raw `u64`
- Enum over boolean: `enum Visibility { Public, Private }` not `is_public: bool`
- Errors: `thiserror`, `?` operator, no `.unwrap()` in production (use `.expect("reason")` for invariants)
- Clippy: `#![warn(clippy::all, clippy::pedantic, clippy::nursery)]`
- Axum: `impl IntoResponse`, use extractors (`State`, `Json`, `Path`), tower for middleware
- Module files: use named files (`handlers.rs` + `handlers/`) not `mod.rs` (Rust 2018+)

## Key Decisions

- **Environment parity:** one code path; local/prod differ only by config. `AUTH_PROVIDER=dev|github`,
  `GIT_PROVIDER=local|github`
- **Session:** signed cookie (stateless), `tower-cookies` in Rust
- **Spec flow:** backend generates `api.yaml` → frontend consumes as generated client. Supervisor communicates via
  WebSocket protocol (spec 0004); shared types live in the `admin-protocol` crate. OpenAPI covers the CMS HTTP API only.

## Current Work — Step 11: Astro Supervisor + Create Site

Steps 1–10 complete. Active work:

- [ ] `astro-supervisor` — Rust binary replacing `management-api.mjs`; connects outbound to backend via WebSocket
  (see spec 0004)
- [ ] `admin-protocol` crate — shared `Command`/`Event`/`Envelope` types consumed by backend and supervisor
- [ ] Auth with dev provider
- [ ] Create site with local git provider
- [ ] Astro preview
