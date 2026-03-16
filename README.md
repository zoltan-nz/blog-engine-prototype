# Blog Engine Prototype

A headless CMS for managing Astro-based static sites.

## Quick Start

```bash
# Prerequisites: Docker + mise

# Run all the servers together
mise run up

# List all the tasks
mise tasks

# Run with SvelteKit + Rust (sweet spot) in watch mode
mise run up-svelte-rust-watch

# Access
# Admin UI:  http://localhost:3000
# API:       http://localhost:8080
# Preview:   http://localhost:4321
```

## Features

An **Admin App** (frontend + backend) that controls an **Astro static site generator**.

- Users can use a UI to create static sites and manage content.
- It is also possible to manage content directly editing the Markdown files as normally we do in Astro projects.
- Content stored as Markdown files (no database)
- Deploy to GitHub Pages / Cloudflare Pages

## Tech Stack

- **Frontends:** SvelteKit, React (DaisyUI)
- **Backends:** Node.js/Fastify, Rust/Axum
- **Static Site:** Astro
- **Containers:** Docker + Compose (Alpine Linux)
- **API Contract:** OpenAPI 3.1

## License

MIT
