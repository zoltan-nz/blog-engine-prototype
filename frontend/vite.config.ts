import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";
import devtoolsJson from "vite-plugin-devtools-json";
import { playwright } from "@vitest/browser-playwright";

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit({
      preprocess: vitePreprocess(),
      adapter: adapter({ fallback: "index.html" }),
    }),
    devtoolsJson(),
  ],
  resolve: { tsconfigPaths: true },

  test: {
    expect: { requireAssertions: true },

    projects: [
      {
        extends: "./vite.config.ts",

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
