// let's create a component test

import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import Footer from "./footer.svelte";

vi.mock("$env/dynamic/public", () => ({
  env: {
    PUBLIC_BACKEND_URL: "http://test-backend.test:8080",
  },
}));

describe("Footer", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
  });

  it("shows Connected when health check succeeds", async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => ({ data: { status: "healthy" } }),
    } as Response);

    render(Footer);

    await expect.element(page.getByText("Connected")).toBeInTheDocument();
  });

  it("shows Disconnected when health check fails", async () => {
    vi.mocked(fetch).mockRejectedValue(new Error("Network error"));

    render(Footer);

    await expect.element(page.getByText("Disconnected")).toBeInTheDocument();
  });
});
