import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-react";
import Footer from "./Footer";

describe("Footer", () => {
  beforeEach(() => {
    vi.stubEnv("VITE_API_BACKEND_URL", "http://test-backend.test:8080");
    vi.stubGlobal("fetch", vi.fn());
  });

  it("shows Connected when health check succeeds", async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => ({ data: { status: "healthy" } }),
    } as Response);

    const screen = await render(<Footer />);

    await expect.element(screen.getByText("Connected")).toBeInTheDocument();
  });

  it("shows Disconnected when health check fails", async () => {
    vi.mocked(fetch).mockRejectedValue(new Error("Network error"));

    const screen = await render(<Footer />);

    await expect.element(screen.getByText("Disconnected")).toBeInTheDocument();
  });
});
