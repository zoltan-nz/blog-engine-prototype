import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import Footer from "./footer.svelte";

const tauri = vi.hoisted(() => ({ isTauri: false }));
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => tauri.isTauri }));

vi.mock("#lib/state/socket.svelte.js", async () => {
  const { fakeSocket } = await import("#lib/test/mocks/socket.js");
  return {
    backendUrl: "http://test-backend.test:8080",
    getSocket: () => fakeSocket,
  };
});

describe("Footer", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    tauri.isTauri = false;
  });

  it("shows Connected when the socket is open", async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        data: { status: "healthy", version: "1.0.0" },
        meta: {
          requestId: "123e4567-e89b-12d3-a456-426614174000",
          serverName: "backend",
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      }),
    } as Response);

    await render(Footer);

    await expect.element(page.getByText("Connected")).toBeInTheDocument();
    await expect.element(page.getByText("Version: 1.0.0")).toBeInTheDocument();
  });

  it("shows the configured backend URL", async () => {
    vi.mocked(fetch).mockRejectedValue(new Error("Network error"));

    await render(Footer);

    await expect
      .element(page.getByText("Backend: http://test-backend.test:8080"))
      .toBeInTheDocument();
  });

  it("shows an in-app backend on desktop without fetching healthz", async () => {
    tauri.isTauri = true;

    await render(Footer);

    await expect.element(page.getByText("Backend: in-app")).toBeInTheDocument();
    expect(fetch).not.toHaveBeenCalled();
  });
});
