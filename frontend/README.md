# Frontend Svelte

Generated with Svelte Kit generator `sv`. Static SPA (adapter-static), served
by the Rust backend.

All server state arrives over the typed WebSocket protocol
(`src/lib/state/socket.svelte.ts`); the types in `src/lib/types/bindings.ts`
are generated from the Rust wire types — never edit them by hand:

```sh
mise run export-types
```

Run the dev server (expects the backend on :8080):

```sh
pnpm run dev

# or start the server and open the app in a new browser tab
pnpm run dev -- --open
```

## Building

To create a production version of your app:

```sh
pnpm run build
```

You can preview the production build with `pnpm run preview`.

## Testing

```sh
pnpm exec vitest run    # component + socket-reducer unit tests
pnpm run check          # svelte-check
```
