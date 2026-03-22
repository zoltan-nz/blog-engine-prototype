import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import PageWithProvider from "$lib/test/page-with-provider.svelte";

describe("/+page.svelte", () => {
  it("should render h1", async () => {
    render(PageWithProvider);

    const heading = page.getByRole("heading", { level: 1 });
    await expect.element(heading).toBeInTheDocument();
  });

  it('should render the "Create a new blog" button', async () => {
    render(PageWithProvider);

    const button = page.getByRole("button", { name: "Create a new blog" });
    await expect.element(button).toBeInTheDocument();
  });
});
