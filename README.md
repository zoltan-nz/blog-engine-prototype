# Blog Engine Prototype

A headless CMS for managing Astro-based static sites.

## Quick Start

```bash
# Prerequisites: Docker + mise

# Start all services
mise run up

# List all tasks
mise tasks

# Start with file watching
mise run up-watch

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

- **Frontend:** SvelteKit (shadcn/ui)
- **Backend:** Rust/Axum
- **Supervisor:** `astro-supervisor` Rust binary (WebSocket client, manages Astro lifecycle)
- **Static Site:** Astro
- **Containers:** Docker + Compose (Alpine Linux)
- **API Contract:** OpenAPI 3.1

## License

MIT
