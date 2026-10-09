<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { Crepe, CrepeFeature } from "@milkdown/crepe";
  import "@milkdown/crepe/theme/common/style.css";
  import "@milkdown/crepe/theme/classic.css";
  import { backendUrl, getSocket } from "#lib/state/socket.svelte.js";
  import { resolveSiteFileUrl } from "./site-file-url.js";
  import { createAutosave } from "#lib/state/autosave.svelte.js";
  import type { PostMeta } from "#lib/types/bindings.js";

  let { siteSlug, post }: { siteSlug: string; post: PostMeta } = $props();

  const socket = getSocket();

  // Seeded once from the prop, then locally owned by the inputs below — not
  // a $derived, since the parent's `{#key post.id}` guarantees a fresh
  // instance (and fresh seed) per distinct post; re-syncing from `post` on
  // every prop update would clobber in-progress typing. untrack() marks the
  // one-time read as intended.
  let title = $state(untrack(() => post.title));
  let description = $state(untrack(() => post.description));
  let editorRoot: HTMLDivElement | undefined = $state();
  let crepe: Crepe | null = null;

  const autosave = createAutosave((draft) =>
    socket.updatePost(
      siteSlug,
      post.id,
      draft.title,
      draft.description,
      draft.body,
    ),
  );

  // Fetch-and-mount runs exactly once per component instance: the parent
  // wraps usage in `{#key post.id}` so a genuinely different post gets a
  // fresh instance (fresh onMount), while a metadata-only prop change on
  // the same post never refetches the body or remounts Crepe (the spec's
  // echo invariant). Title/description keep their local values too; see
  // the seeding comment above.
  let destroyed = false;

  onMount(() => {
    void (async () => {
      const body = await socket.requestPost(siteSlug, post.id);
      if (destroyed || !editorRoot) return;
      // Show images from the site's own files; the saved markdown keeps
      // its original URLs.
      crepe = new Crepe({
        root: editorRoot,
        defaultValue: body,
        featureConfigs: {
          [CrepeFeature.ImageBlock]: {
            proxyDomURL: (url: string) =>
              resolveSiteFileUrl(backendUrl, siteSlug, post.id, url),
          },
        },
      });
      await crepe.create();
      crepe.on((listener) => {
        listener.markdownUpdated((_ctx, markdown) => {
          autosave.update({ title, description, body: markdown });
        });
      });
    })();
  });

  function handleMetaInput(): void {
    autosave.update({
      title,
      description,
      body: crepe?.getMarkdown() ?? "",
    });
  }

  onDestroy(() => {
    destroyed = true;
    void crepe?.destroy();
  });
</script>

<div class="flex h-full flex-col gap-3">
  <div class="flex items-center justify-between">
    <span class="text-sm text-surface-600-400">
      {#if autosave.status === "saving"}
        Saving…
      {:else if autosave.status === "saved"}
        Saved
      {:else if autosave.status === "error"}
        Save failed — retrying on next edit
      {/if}
    </span>
  </div>

  <label class="label" for="post-title">
    <span class="label-text">Title</span>
    <input
      id="post-title"
      type="text"
      class="input"
      bind:value={title}
      oninput={handleMetaInput}
    />
  </label>

  <label class="label" for="post-description">
    <span class="label-text">Description</span>
    <input
      id="post-description"
      type="text"
      class="input"
      bind:value={description}
      oninput={handleMetaInput}
    />
  </label>

  <div
    class="flex-1 overflow-y-auto rounded-container border border-surface-200-800 p-2"
    bind:this={editorRoot}
  ></div>
</div>
