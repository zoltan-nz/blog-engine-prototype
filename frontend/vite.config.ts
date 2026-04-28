import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";
import devtoolsJson from "vite-plugin-devtools-json";
import { playwright } from "@vitest/browser-playwright";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit(), devtoolsJson()],
  resolve: { tsconfigPaths: true },

  test: {
    expect: { requireAssertions: true },

    projects: [
      {
        extends: "./vite.config.ts",

        resolve: {
          alias: {
            // Mock SvelteKit's public env module for isolated component tests.
            // In production, SvelteKit injects this via SSR; tests need a stub.
            "$env/dynamic/public": resolve("./src/lib/test/mocks/env.ts"),
          },
        },

        test: {
          name: "client",

          browser: {
            enabled: true,
            provider: playwright(),
            instances: [{ browser: "chromium", headless: true }],
          },

          include: ["src/**/*.svelte.{test,spec}.{js,ts}"],
          exclude: ["src/lib/server/**"],
        },
      },
    ],
  },
});
