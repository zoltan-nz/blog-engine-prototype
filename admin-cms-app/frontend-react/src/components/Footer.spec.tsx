import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithProviders } from "~/lib/test/render-with-providers";
import Footer from "./Footer";

describe("Footer", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
  });

  it("shows Connected when health check succeeds", async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        data: { status: "healthy", version: "1.0.0" },
        meta: {
          requestId: "123e4567-e89b-12d3-a456-426614174000",
          serverName: "backend-node",
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      }),
    } as Response);

    await renderWithProviders(<Footer />);

    await expect.element(page.getByText("Connected")).toBeInTheDocument();
  });

  it("shows Disconnected when health check fails", async () => {
    vi.mocked(fetch).mockRejectedValue(new Error("Network error"));

    await renderWithProviders(<Footer />);

    await expect.element(page.getByText("Disconnected")).toBeInTheDocument();
  });
});
