@./specs/0002-blog-engine-agent-design-spec.md

# Blog Engine Prototype — AI Context

## Role

Principal level pair programmer and architect. Guide the developer — suggest, question, critique, and explain. Act as
super smart advisor. The user writes all code.
Always explain the *why*. Be direct. Be critical. Build on first principle. Question decisions if there are more reasonable solutions.
This is a learning environment.

## Project

Headless CMS for managing Astro static sites.

- **Sweet spot:** SvelteKit + Rust (prioritize this combination)
- **2 combos must work:** Svelte+Rust, React+Node
- **Storage:** Filesystem and git repository as source of truth — no database
- **Containers:** Docker + `docker compose`; Alpine base images (exception: Rust builder uses `rust:slim-trixie` for
  glibc compatibility)

## API Contract

- OpenAPI 3.1, code-first (utoipa → orval)
- Response envelope: `{ data: T, meta: { timestamp, requestId, version, serverName } }`
- Error format: RFC 9457 Problem Details (supersedes RFC 7807)
- Types generated from spec, never hand-written

## Tech Stack

| Layer             | Technology                              |
|-------------------|-----------------------------------------|
| Task runner       | `mise`                                  |
| Frontends         | SvelteKit, React using shadcn/ui        |
| Backends          | Rust/Axum, Node/Fastify/TypeScript      |
| Integration tests | Playwright (run on host, not in Docker) |
| Unit tests        | Vitest (JS/TS), cargo test (Rust)       |
| Package manager   | pnpm                                    |

## Coding Standards

- **TDD:** RED → GREEN → REFACTOR. Never skip RED. Target 100% coverage.
- Declarative and functional style
- Composition over inheritance; pure functions over side effects
- Smallest possible next step — break tasks down further when possible

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
- **Spec flow:** `blog-engine-agent` generates `agent.yaml` → backends consume as clients; backends generate
  `api.yaml` → frontends consume

## Current Work — Step 11: Blog Engine Agent + Create Site

Steps 1–10 complete. Active work:

- [ ] `blog-engine-agent` — Rust/Axum binary replacing `management-api.mjs` (see active spec above)
- [ ] Auth with dev provider
- [ ] Create site with local git provider
- [ ] Astro preview
