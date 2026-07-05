import { describe, expect, it } from "vitest";
import { removeSite, upsertSite } from "./socket.svelte";
import type { SiteView } from "$lib/types/bindings.js";

const ready = (slug: string): SiteView => ({
  slug,
  name: slug.toUpperCase(),
  state: { type: "Ready" },
});

describe("upsertSite", () => {
  it("inserts a new site sorted by slug", () => {
    const result = upsertSite([ready("b-blog")], ready("a-blog"));
    expect(result.map((s) => s.slug)).toEqual(["a-blog", "b-blog"]);
  });

  it("replaces an existing site by slug", () => {
    const updated: SiteView = {
      slug: "a-blog",
      name: "A BLOG",
      state: { type: "Building" },
    };
    const result = upsertSite([ready("a-blog"), ready("b-blog")], updated);
    expect(result).toHaveLength(2);
    expect(result[0].state.type).toBe("Building");
  });
});

describe("removeSite", () => {
  it("removes by slug and keeps the rest", () => {
    const result = removeSite([ready("a-blog"), ready("b-blog")], "a-blog");
    expect(result.map((s) => s.slug)).toEqual(["b-blog"]);
  });

  it("is a no-op for unknown slugs", () => {
    const result = removeSite([ready("a-blog")], "ghost");
    expect(result).toHaveLength(1);
  });
});
