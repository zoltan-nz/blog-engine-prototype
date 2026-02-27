import { defineConfig } from "orval";

export default defineConfig({
  api: {
    output: {
      target: "./bin/generated-api.ts",
      client: "svelte-query",
      override: {
        mutator: {
          path: "./bin/lib/api/fetch-with-base-url.ts",
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
