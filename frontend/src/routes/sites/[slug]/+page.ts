// Dynamic route, not known at build time; adapter-static serves it via the
// index.html fallback and SvelteKit takes over client-side (see ssr = false
// in the root layout).
export const prerender = false;
