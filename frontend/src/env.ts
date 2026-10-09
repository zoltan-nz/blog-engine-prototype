import { defineEnvVars } from "@sveltejs/kit/env";

const DEFAULT_BACKEND_URL = "http://localhost:8080";

export const variables = defineEnvVars({
  PUBLIC_API_BACKEND_URL: {
    public: true,
    // Static SPA: there is no server at runtime, so inline the value at build time.
    static: true,
    schema: (value) => value || DEFAULT_BACKEND_URL,
  },
});
