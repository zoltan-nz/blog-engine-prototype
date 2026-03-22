// Mock for $env/dynamic/public used in browser unit tests.
// SvelteKit injects this at runtime via SSR; isolated component tests need this stub.
export const env = {
  PUBLIC_API_BACKEND_URL: "http://localhost:8080",
};
