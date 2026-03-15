import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { renderWithProviders } from "~/lib/test/render-with-providers";
import Home from "./home";

describe("Home", () => {
  it("should render h1", async () => {
    await renderWithProviders(<Home />);

    const heading = page.getByRole("heading", { level: 1 });
    await expect.element(heading).toBeInTheDocument();
  });

  it('should render the "Create a new blog" button', async () => {
    await renderWithProviders(<Home />);

    const button = page.getByRole("button", { name: "Create a new blog" });
    await expect.element(button).toBeInTheDocument();
  });
});
