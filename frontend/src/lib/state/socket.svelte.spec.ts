import { describe, expect, it } from "vitest";
import {
  PendingRequests,
  PendingUpdates,
  removeSite,
  removePost,
  upsertPost,
  upsertSite,
} from "./socket.svelte";
import type { PostMeta, SiteView } from "$lib/types/bindings.js";

const ready = (slug: string): SiteView => ({
  slug,
  name: slug.toUpperCase(),
  state: { type: "Ready" },
});

const post = (id: string, pubDate: string): PostMeta => ({
  id,
  title: id.toUpperCase(),
  description: "desc",
  pub_date: pubDate,
  updated_date: null,
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

describe("upsertPost", () => {
  it("inserts a new post sorted by pub_date descending", () => {
    const posts = { "my-blog": [post("older", "2026-01-01")] };
    const result = upsertPost(posts, "my-blog", post("newer", "2026-06-01"));
    expect(result["my-blog"].map((p) => p.id)).toEqual(["newer", "older"]);
  });

  it("replaces an existing post by id, keeping sort order", () => {
    const posts = {
      "my-blog": [post("newer", "2026-06-01"), post("older", "2026-01-01")],
    };
    const updated: PostMeta = {
      ...post("older", "2026-01-01"),
      title: "Edited",
    };
    const result = upsertPost(posts, "my-blog", updated);
    expect(result["my-blog"]).toHaveLength(2);
    expect(result["my-blog"][1].title).toBe("Edited");
  });

  it("creates the site's post list when it doesn't exist yet", () => {
    const result = upsertPost({}, "my-blog", post("first", "2026-01-01"));
    expect(result["my-blog"].map((p) => p.id)).toEqual(["first"]);
  });
});

describe("removePost", () => {
  it("removes by id and keeps the rest", () => {
    const posts = {
      "my-blog": [post("a", "2026-01-01"), post("b", "2026-02-01")],
    };
    const result = removePost(posts, "my-blog", "a");
    expect(result["my-blog"].map((p) => p.id)).toEqual(["b"]);
  });

  it("is a no-op for an unknown site slug", () => {
    const posts = { "my-blog": [post("a", "2026-01-01")] };
    const result = removePost(posts, "ghost", "a");
    expect(result).toEqual(posts);
  });
});

describe("PendingRequests", () => {
  it("resolves the matching correlation id with the body", async () => {
    const pending = new PendingRequests();
    const promise = pending.register("c-1");

    const settled = pending.settleBody("c-1", "the body");

    expect(settled).toBe(true);
    await expect(promise).resolves.toBe("the body");
  });

  it("rejects the matching correlation id with the error", async () => {
    const pending = new PendingRequests();
    const promise = pending.register("c-2");

    const settled = pending.settleError("c-2", {
      code: "PostNotFound",
      message: "not found",
      correlationId: "c-2",
    });

    expect(settled).toBe(true);
    await expect(promise).rejects.toMatchObject({ code: "PostNotFound" });
  });

  it("does not settle on an unrelated correlation id", () => {
    const pending = new PendingRequests();
    pending.register("c-3");

    const settled = pending.settleBody("unrelated", "body");

    expect(settled).toBe(false);
  });
});

describe("PendingUpdates", () => {
  it("resolves when the matching site/id PostChanged arrives", async () => {
    const pending = new PendingUpdates();
    const promise = pending.register("c-1", "my-blog", "hello-world");

    const settled = pending.settleChanged("my-blog", "hello-world");

    expect(settled).toBe(true);
    await expect(promise).resolves.toBeUndefined();
  });

  it("rejects when the correlated Error arrives", async () => {
    const pending = new PendingUpdates();
    const promise = pending.register("c-1", "my-blog", "hello-world");

    const settled = pending.settleError("c-1", {
      code: "PostNotFound",
      message: "not found",
      correlationId: "c-1",
    });

    expect(settled).toBe(true);
    await expect(promise).rejects.toMatchObject({ code: "PostNotFound" });
  });

  it("does not settle on an unrelated site/id or correlation id", () => {
    const pending = new PendingUpdates();
    pending.register("c-1", "my-blog", "hello-world");

    expect(pending.settleChanged("other-blog", "hello-world")).toBe(false);
    expect(
      pending.settleError("unrelated", {
        code: "Internal",
        message: "x",
        correlationId: "unrelated",
      }),
    ).toBe(false);
  });
});
