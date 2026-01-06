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

## What Is This?

An **Admin App** (frontend + backend) that controls an **Astro static site generator**.

- Non-techy users get a UI to manage content
- Techy users can still edit Astro directly
- Content stored as Markdown files (no database)
- Deploy to GitHub Pages / Cloudflare Pages

## Stack Variants

| Frontend  | Backend      | Command                 |
|-----------|--------------|-------------------------|
| SvelteKit | Rust/Axum    | `./run.sh svelte rust`  |
| SvelteKit | Node/Fastify | `./run.sh svelte node`  |
| React     | Rust/Axum    | `./run.sh react rust`   |
| React     | Node/Fastify | `./run.sh react node`   |

## Documentation

- [Architecture](./ARCHITECTURE.md) — System design, decisions, API format, findings, benchmarks, comparisons
- [Project Plan](./CLAUDE.md) — Current phase, TODOs, progress
- [Agent Guidelines](./AGENTS.md) — AI assistant instructions and coding standards

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
