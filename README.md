# Blog Engine Prototype

A headless CMS for managing Astro-based static sites. Built as a framework comparison sandbox.

## Quick Start

```bash
# Prerequisites: Podman + podman compose + mise

# Run all the servers together
mise run up

# List all the tasks
mise tasks

# Run with SvelteKit + Rust (sweet spot)
podman compose --profile rust --profile svelte

# Access
# Admin UI:  http://localhost:3000
# API:       http://localhost:8000
# Preview:   http://localhost:4321
```

## Features

An **Admin App** (frontend + backend) that controls an **Astro static site generator**.

- Users can use a UI to create static sites and manage content.
- It is also possible to manage content directly editing the Markdown files as normally we do in Astro projects.
- Content stored as Markdown files (no database)
- Deploy to GitHub Pages / Cloudflare Pages

## Documentation

- [Architecture](./ARCHITECTURE.md) — System design, decisions, API format, findings, benchmarks, comparisons
- [Notes](NOTES.md) — Current phase, TODOs, progress

## Development

```bash
# Run integration tests
cd tests/integration && pnpm exec playwright test

# Generate types from OpenAPI contract
npm run generate:types
```

## Tech Stack

- **Frontends:** SvelteKit, React (DaisyUI)
- **Backends:** Node.js/Fastify, Rust/Axum
- **Static Site:** Astro
- **Containers:** Podman + Compose (Alpine Linux)
- **API Contract:** OpenAPI 3.1

## Setup Tools with `mise`

We use [mise](https://mise.jdx.dev/) for tool version management and task running.

```bash
# Install required tools (Node, pnpm, Rust)
mise install

# List available tasks
mise tasks

# Common tasks
mise run up          # Start all services (both backends)
mise run up-rust     # Start with Rust backend only
mise run up-node     # Start with Node backend only
mise run down        # Stop all services
mise run test        # Run integration tests
mise run health      # Check all health endpoints
mise run stats       # Show container resource usage
```

## License

MIT
