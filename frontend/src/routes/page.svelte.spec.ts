import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import Page from "./+page.svelte";
import { fakeSocket } from "$lib/test/mocks/socket.js";

vi.mock("$lib/state/socket.svelte", async () => {
  const { fakeSocket } = await import("$lib/test/mocks/socket.js");
  return {
    backendUrl: "http://localhost:8080",
    getSocket: () => fakeSocket,
  };
});

describe("/+page.svelte", () => {
  it("should render h1", async () => {
    render(Page);

    const heading = page.getByRole("heading", { level: 1 });
    await expect.element(heading).toBeInTheDocument();
  });

  it('should render the "Create a new blog" button', async () => {
    render(Page);

    const button = page.getByRole("button", { name: "Create a new blog" });
    await expect.element(button).toBeInTheDocument();
  });

  it("should render a site card from socket state", async () => {
    fakeSocket.sites = [
      { slug: "my-blog", name: "My Blog", state: { type: "Ready" } },
    ];
    render(Page);

    await expect.element(page.getByText("My Blog")).toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: /Start Preview/ }))
      .toBeInTheDocument();
  });
});
