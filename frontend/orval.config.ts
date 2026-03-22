import { defineConfig } from "orval";

export default defineConfig({
  api: {
    output: {
      target: "./src/generated-api.ts",
      client: "svelte-query",
      override: {
        mutator: {
          path: "./src/lib/api/fetch-with-base-url.ts",
          name: "fetchWithServerUrl",
          extension: ".js",
        },
      },
      prettier: true,
      tsconfig: "./tsconfig.json",
    },
    input: {
      target: "../../open-api-contracts/api.yaml",
    },
  },
});
