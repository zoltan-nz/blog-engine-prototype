# Blog Engine — Architecture

## System Overview

- Headless CMS (Admin App) controls Astro-based static sites. 
- No database. Git is the source of truth. Every site is a Git repo.
- Edit and snapshots saved as a commit. 
- Single stack: Rust backend + SvelteKit frontend. 
- Pure WebSocket protocol between frontend, backend with supervisor process that manages Astro projects and previews.

Simple architecture:
 - Frontend: SvelteKit SPA; server state arrives by WebSocket push into a `$state` store (no TanStack Query)
 - Backend: Rust, Axum
 - Communication: pure WebSocket protocol (`/ws`); only `/healthz` and static assets stay HTTP
 - Frontend is served by the Rust backend

## State machines

- Domain state is governed by hand-rolled finite state machines (`backend/src/fsm/`):
  `SiteState` and `PreviewState` are pure `transition(state, event)` functions whose
  state enums double as specta wire types. Illegal client commands are rejected by
  the FSM and surfaced as typed `Error` events.

## Deployment Models

- One binary using `rust-embed` behind the `embed` cargo feature
  (`mise run build`). Dev builds serve the SPA from disk via `ServeDir`.

## Strict types between frontend and backend

- Using `specta` (1.x) to generate bindings for the frontend:
  `mise run export-types` → `frontend/src/lib/types/bindings.ts`.
