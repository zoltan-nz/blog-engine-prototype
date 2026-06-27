# Blog Engine — Architecture

## System Overview

- Headless CMS (Admin App) controls Astro-based static sites. 
- No database. Git is the source of truth. Every site is a Git repo.
- Edit and snapshots saved as a commit. 
- Single stack: Rust backend + SvelteKit frontend. 
- Pure WebSocket protocol between frontend, backend with supervisor process that manages Astro projects and previews.

Simple architecture:
 - Frontend: SvelteKit SPA, TanStack Query
 - Backend: Rust, Axum
 - Communication: pure WebSocket protocol via HTTP2
 - Frontend is served by the Rust backend

## Deployment Models

- On binary using `rust-embed-for-web`.

## Strict types between frontend and backend

- Using `specta` to generate bindings for the frontend.
