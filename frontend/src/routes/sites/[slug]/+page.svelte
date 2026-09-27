<script lang="ts">
  import { page } from "$app/state";
  import { getSocket } from "$lib/state/socket.svelte";
  import { decidePreviewAction } from "$lib/state/preview-switch.js";
  import PostList from "$lib/components/post-list/post-list.svelte";
  import PostEditor from "$lib/components/post-editor/post-editor.svelte";
  import { ArrowLeft, LoaderCircle } from "@lucide/svelte";
  import type { PostMeta } from "$lib/types/bindings.js";

  const socket = getSocket();
  let selectedPost = $state<PostMeta | null>(null);
  // `page.params` is typed across the whole route tree, so `slug` reads as
  // optional even though this route guarantees it — this component only
  // ever renders under `/sites/[slug]`.
  const siteSlug = $derived(page.params.slug ?? "");
  const site = $derived(socket.sites.find((s) => s.slug === siteSlug));

  // Re-decided on every preview change: 'none' while already running/starting
  // for this site (or stopping) avoids re-issuing commands mid-transition;
  // 'stop-then-start' naturally chains once the resulting PreviewChanged
  // arrives and the effect re-runs.
  $effect(() => {
    const action = decidePreviewAction(socket.preview, siteSlug);
    if (action === "start") {
      socket.startPreview(siteSlug);
    } else if (action === "stop-then-start") {
      socket.stopPreview();
    }
  });

  const previewIsForThisSite = $derived(socket.preview.slug === siteSlug);

  const iframeSrc = $derived(
    previewIsForThisSite &&
      socket.preview.state.type === "Running" &&
      socket.preview.url
      ? selectedPost
        ? `${socket.preview.url}/blog/${selectedPost.id}`
        : socket.preview.url
      : null,
  );
</script>

<div class="flex h-[calc(100vh-2rem)] flex-col">
  <header class="mb-4 flex items-center gap-3">
    <a
      href="/"
      class="btn-icon preset-outlined-surface-300-700 btn-sm"
      aria-label="Back to dashboard"
    >
      <ArrowLeft size={16} />
    </a>
    <div>
      <h1 class="text-xl font-bold">{site?.name ?? siteSlug}</h1>
      <p class="font-mono text-sm text-surface-600-400">{siteSlug}</p>
    </div>
  </header>

  <div class="grid flex-1 grid-cols-2 gap-4 overflow-hidden">
    <div
      class="flex flex-col gap-4 overflow-y-auto rounded-container border border-surface-200-800 p-4"
    >
      <PostList
        {siteSlug}
        selectedId={selectedPost?.id ?? null}
        onSelect={(post) => (selectedPost = post)}
      />
      {#if selectedPost}
        {#key selectedPost.id}
          <PostEditor {siteSlug} post={selectedPost} />
        {/key}
      {/if}
    </div>

    <div
      class="flex items-center justify-center overflow-hidden rounded-container preset-tonal-surface"
    >
      {#if previewIsForThisSite && socket.preview.state.type === "Running" && socket.preview.url}
        <iframe
          src={iframeSrc ?? ""}
          title="Live preview"
          class="h-full w-full border-0"
        ></iframe>
      {:else if previewIsForThisSite && socket.preview.state.type === "Failed"}
        <div
          class="flex flex-col items-center gap-2 p-4 text-center text-error-500"
        >
          <p>Preview failed to start: {socket.preview.state.payload.reason}</p>
          <button
            class="btn preset-outlined-surface-300-700 btn-sm"
            onclick={() => socket.startPreview(siteSlug)}
          >
            Retry
          </button>
        </div>
      {:else}
        <div class="flex flex-col items-center gap-2 text-surface-600-400">
          <LoaderCircle size={32} class="animate-spin" />
          <p>Starting preview…</p>
        </div>
      {/if}
    </div>
  </div>
</div>
