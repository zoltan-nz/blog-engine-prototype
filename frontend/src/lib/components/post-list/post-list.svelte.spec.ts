import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import PostList from "./post-list.svelte";
import { fakeSocket } from "#lib/test/mocks/socket.js";
import type { PostMeta } from "#lib/types/bindings.js";

vi.mock("#lib/state/socket.svelte.js", async () => {
  const { fakeSocket } = await import("#lib/test/mocks/socket.js");
  return {
    backendUrl: "http://localhost:8080",
    getSocket: () => fakeSocket,
  };
});

const post = (id: string, title: string, pubDate: string): PostMeta => ({
  id,
  title,
  description: "desc",
  pub_date: pubDate,
  updated_date: null,
});

describe("PostList", () => {
  beforeEach(() => {
    fakeSocket.posts = {};
    fakeSocket.createPost = vi.fn();
    fakeSocket.deletePost = vi.fn();
    fakeSocket.draftPost = vi.fn(() => new Promise<void>(() => {}));
    vi.stubGlobal(
      "confirm",
      vi.fn(() => true),
    );
  });

  it("renders posts for the site in the order the store provides", async () => {
    fakeSocket.posts = {
      "my-blog": [
        post("newer", "Newer Post", "2026-06-01"),
        post("older", "Older Post", "2026-01-01"),
      ],
    };
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    const rows = page.getByRole("listitem");
    await expect.element(rows.nth(0)).toMatchTextContent("Newer Post");
    await expect.element(rows.nth(1)).toMatchTextContent("Older Post");
  });

  it("shows an empty state when the site has no posts", async () => {
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    await expect.element(page.getByText(/no posts yet/i)).toBeInTheDocument();
  });

  it("opens a dialog and creates a post from the entered title, slugified into an id", async () => {
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    await page.getByRole("button", { name: "New post" }).click();
    await expect.element(page.getByRole("dialog")).toBeInTheDocument();
    await page.getByLabelText("Title").fill("New Post Title");
    await page.getByRole("button", { name: "Create", exact: true }).click();

    expect(fakeSocket.createPost).toHaveBeenCalledWith(
      "my-blog",
      "new-post-title",
      "New Post Title",
      "",
      "",
    );
  });

  it("does not create a post if the dialog is cancelled", async () => {
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    await page.getByRole("button", { name: "New post" }).click();
    await page.getByLabelText("Title").fill("Should not save");
    await page.getByRole("button", { name: "Cancel" }).click();

    expect(fakeSocket.createPost).not.toHaveBeenCalled();
  });

  it("deletes a post after confirmation", async () => {
    fakeSocket.posts = {
      "my-blog": [post("a-post", "A Post", "2026-01-01")],
    };
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    await page.getByRole("button", { name: /delete/i }).click();

    expect(fakeSocket.deletePost).toHaveBeenCalledWith("my-blog", "a-post");
  });

  it("calls onSelect when a row is clicked", async () => {
    const onSelect = vi.fn();
    fakeSocket.posts = {
      "my-blog": [post("a-post", "A Post", "2026-01-01")],
    };
    await render(PostList, { props: { siteSlug: "my-blog", onSelect } });

    await page.getByText("A Post").click();

    expect(onSelect).toHaveBeenCalledWith(
      post("a-post", "A Post", "2026-01-01"),
    );
  });

  it("asks the AI agent to draft a post from a topic and shows it as drafting", async () => {
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    await page.getByRole("button", { name: "Draft with AI" }).click();
    await page.getByLabelText("Topic").fill("Why static sites are cheap");
    await page.getByRole("button", { name: "Draft", exact: true }).click();

    expect(fakeSocket.draftPost).toHaveBeenCalledWith(
      "my-blog",
      "why-static-sites-are-cheap",
      "Why static sites are cheap",
    );
    await expect
      .element(page.getByText(/drafting.*why static sites are cheap/i))
      .toBeInTheDocument();
  });

  it("shows why a draft failed", async () => {
    fakeSocket.draftPost = vi.fn(() =>
      Promise.reject({
        code: "Internal",
        message: "agent failed: not signed in",
        correlationId: "c-1",
      }),
    );
    await render(PostList, {
      props: { siteSlug: "my-blog", onSelect: vi.fn() },
    });

    await page.getByRole("button", { name: "Draft with AI" }).click();
    await page.getByLabelText("Topic").fill("Anything");
    await page.getByRole("button", { name: "Draft", exact: true }).click();

    await expect
      .element(page.getByText("agent failed: not signed in"))
      .toBeInTheDocument();
  });
});
