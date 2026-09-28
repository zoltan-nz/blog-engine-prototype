import { describe, expect, it } from "vitest";
import { resolveSiteFileUrl } from "./site-file-url.js";

const BACKEND = "http://localhost:8080";

describe("resolveSiteFileUrl", () => {
  it("resolves a path relative to the post file into the site's src/", () => {
    expect(
      resolveSiteFileUrl(
        BACKEND,
        "my-blog",
        "first-post",
        "../../assets/photo.jpg",
      ),
    ).toBe("http://localhost:8080/site-files/my-blog/assets/photo.jpg");
  });

  it("leaves absolute URLs unchanged", () => {
    expect(
      resolveSiteFileUrl(
        BACKEND,
        "my-blog",
        "first-post",
        "https://example.com/a.png",
      ),
    ).toBe("https://example.com/a.png");
  });
});
