import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import PostEditor from "./post-editor.svelte";
import { fakeSocket } from "$lib/test/mocks/socket.js";
import type { PostMeta } from "$lib/types/bindings.js";

vi.mock("$lib/state/socket.svelte", async () => {
  const { fakeSocket } = await import("$lib/test/mocks/socket.js");
  return {
    backendUrl: "http://localhost:8080",
    getSocket: () => fakeSocket,
  };
});

const post = (overrides: Partial<PostMeta> = {}): PostMeta => ({
  id: "hello-world",
  title: "Hello World",
  description: "A first post",
  pub_date: "2026-01-01",
  updated_date: null,
  ...overrides,
});

describe("PostEditor", () => {
  beforeEach(() => {
    fakeSocket.requestPost = vi.fn().mockResolvedValue("Body text here.");
    fakeSocket.updatePost = vi.fn().mockResolvedValue(undefined);
  });

  it("loads the post body via requestPost and mounts the editor with it", async () => {
    await render(PostEditor, {
      props: { siteSlug: "my-blog", post: post() },
    });

    expect(fakeSocket.requestPost).toHaveBeenCalledWith(
      "my-blog",
      "hello-world",
    );
    await expect.element(page.getByText("Body text here.")).toBeInTheDocument();
  });

  it("prefills title and description inputs from the post prop", async () => {
    await render(PostEditor, {
      props: { siteSlug: "my-blog", post: post() },
    });

    await expect
      .element(page.getByLabelText("Title"))
      .toHaveValue("Hello World");
    await expect
      .element(page.getByLabelText("Description"))
      .toHaveValue("A first post");
  });

  it("does not re-fetch the body when only metadata changes on the same post id", async () => {
    const { rerender } = await render(PostEditor, {
      props: { siteSlug: "my-blog", post: post() },
    });
    await expect.element(page.getByText("Body text here.")).toBeInTheDocument();

    await rerender({
      siteSlug: "my-blog",
      post: post({ title: "Edited Elsewhere" }),
    });

    expect(fakeSocket.requestPost).toHaveBeenCalledTimes(1);
  });
});
