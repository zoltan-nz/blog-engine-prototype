import { describe, expect, it } from "vitest";
import { resolveSiteFileUrl } from "./site-file-url.js";

const HTTP_BASE = "http://localhost:8080/site-files";

describe("resolveSiteFileUrl", () => {
  it("resolves a path relative to the post file into the site's src/", () => {
    expect(
      resolveSiteFileUrl(
        HTTP_BASE,
        "my-blog",
        "first-post",
        "../../assets/photo.jpg",
      ),
    ).toBe("http://localhost:8080/site-files/my-blog/assets/photo.jpg");
  });

  it("resolves against the desktop site-files:// protocol", () => {
    expect(
      resolveSiteFileUrl(
        "site-files://localhost",
        "my-blog",
        "first-post",
        "../../assets/photo.jpg",
      ),
    ).toBe("site-files://localhost/my-blog/assets/photo.jpg");
  });

  it("leaves absolute URLs unchanged", () => {
    expect(
      resolveSiteFileUrl(
        HTTP_BASE,
        "my-blog",
        "first-post",
        "https://example.com/a.png",
      ),
    ).toBe("https://example.com/a.png");
  });
});
