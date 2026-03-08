import { defineConfig } from "orval";

export default defineConfig({
  api: {
    output: {
      target: "./src/generated-api.ts",
      client: "react-query",
      override: {
        mutator: {
          path: "./src/lib/api/fetch-with-base-url.ts",
          name: "fetchWithBaseUrl",
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
