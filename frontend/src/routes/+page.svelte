<script lang="ts">
  import { Dialog, Portal } from "@skeletonlabs/skeleton-svelte";
  import { getSocket } from "$lib/state/socket.svelte";
  import type { SiteView } from "$lib/types/bindings.js";
  import {
    Hammer,
    Loader,
    LoaderCircle,
    Play,
    Square,
    Trash2,
    X,
  } from "@lucide/svelte";

  const socket = getSocket();

  // Converts a blog name into a URL-safe slug (directory name + git repo name).
  // Rules: GitHub repo names allow [a-z0-9._-], max 100 chars.
  function nameToSlug(name: string): string {
    return name
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 100);
  }

  let dialogOpen = $state(false);
  let blogName = $state("");
  let slug = $derived(nameToSlug(blogName));

  let previewBusy = $derived(
    socket.preview.state.type === "Starting" ||
      socket.preview.state.type === "Stopping",
  );

  function openModal() {
    blogName = "";
    dialogOpen = true;
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!blogName.trim() || !slug) return;
    socket.createSite(blogName.trim(), slug);
    dialogOpen = false;
  }

  function handleDelete(site: SiteView) {
    if (
      !confirm(
        "Are you sure you want to destroy this blog? This action cannot be undone.",
      )
    ) {
      return;
    }
    socket.deleteSite(site.slug);
  }

  function isPreviewedSite(site: SiteView): boolean {
    return socket.preview.slug === site.slug;
  }
</script>

<div class="mx-auto max-w-4xl">
  <div class="mb-8 flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold">Blog Engine Admin</h1>
      <p class="mt-1 text-surface-600-400">Manage your Astro blog sites.</p>
    </div>
    <button class="btn preset-filled-primary-500" onclick={openModal}>
      Create a new blog
    </button>
  </div>

  {#if socket.lastError}
    <div
      class="mb-4 flex items-center justify-between rounded-container preset-tonal-error p-4"
    >
      <span>
        <strong>{socket.lastError.code}</strong>: {socket.lastError.message}
      </span>
      <button
        class="btn-icon btn-sm"
        onclick={() => socket.dismissError()}
        aria-label="Dismiss error"
      >
        <X size={16} />
      </button>
    </div>
  {/if}

  {#if socket.status !== "open"}
    <div class="mb-4 rounded-container preset-tonal-warning p-4">
      <span>
        {socket.status === "connecting"
          ? "Connecting to backend…"
          : "Connection lost — reconnecting…"}
      </span>
    </div>
  {/if}

  <!-- Site list -->
  {#if socket.sites.length === 0}
    <div class="card rounded-container preset-tonal-surface py-16 text-center">
      <p class="text-surface-600-400">No blogs yet. Create your first one!</p>
    </div>
  {:else}
    <ul class="grid gap-4 sm:grid-cols-2">
      {#each socket.sites as site (site.slug)}
        {@const busy =
          site.state.type === "Creating" || site.state.type === "Deleting"}
        {@const previewed = isPreviewedSite(site)}
        {@const isLive = previewed && socket.preview.state.type === "Running"}
        {@const isStarting =
          previewed && socket.preview.state.type === "Starting"}
        <li
          class="card rounded-container border border-surface-200-800 preset-filled-surface-100-900 shadow-sm"
        >
          <div class="p-4">
            <div class="flex items-start justify-between">
              <div>
                <h2 class="flex items-center gap-2 text-xl font-bold">
                  {site.name}
                  {#if site.state.type === "Creating"}
                    <span class="badge preset-tonal-surface text-xs">
                      <LoaderCircle size={12} class="animate-spin" />
                      Scaffolding…
                    </span>
                  {:else if site.state.type === "Building"}
                    <span class="badge preset-tonal-primary text-xs">
                      <LoaderCircle size={12} class="animate-spin" />
                      Building…
                    </span>
                  {:else if site.state.type === "BuildFailed"}
                    <span
                      class="badge preset-filled-error-500 text-xs"
                      title={site.state.payload.reason}
                    >
                      Build failed
                    </span>
                  {:else if site.state.type === "Deleting"}
                    <span class="badge preset-tonal-surface text-xs">
                      <LoaderCircle size={12} class="animate-spin" />
                      Deleting…
                    </span>
                  {/if}
                  {#if isLive && socket.preview.url}
                    <a
                      href={socket.preview.url}
                      target="_blank"
                      class="badge preset-filled-success-500 text-xs"
                    >
                      Live
                    </a>
                  {/if}
                </h2>
                <p class="font-mono text-sm text-surface-600-400">
                  {site.slug}
                </p>
              </div>
            </div>
            <div class="mt-4 flex gap-2">
              {#if isLive}
                <button
                  class="btn preset-outlined-surface-300-700 btn-sm"
                  disabled={previewBusy && !isLive}
                  onclick={() => socket.stopPreview()}
                >
                  <Square size={16} />Stop Preview
                </button>
              {:else}
                <button
                  class="btn preset-outlined-surface-300-700 btn-sm"
                  disabled={busy ||
                    previewBusy ||
                    site.state.type === "Building"}
                  onclick={() => socket.startPreview(site.slug)}
                >
                  {#if isStarting}
                    <LoaderCircle size={16} class="animate-spin" />
                    Starting preview…
                  {:else}
                    <Play size={16} />Start Preview
                  {/if}
                </button>
              {/if}
              <button
                class="btn preset-outlined-surface-300-700 btn-sm"
                disabled={busy ||
                  site.state.type === "Building" ||
                  site.state.type === "Creating"}
                onclick={() => socket.buildSite(site.slug)}
              >
                <Hammer size={16} />Build
              </button>
              <div class="ml-auto">
                <button
                  onclick={() => handleDelete(site)}
                  disabled={busy || site.state.type === "Building"}
                  class="btn items-center preset-outlined-error-300-700 btn-sm text-error-50"
                >
                  {#if site.state.type === "Deleting"}
                    <Loader size={16} class="animate-spin" />
                  {:else}
                    <Trash2 size={16} />
                  {/if}
                  Destroy
                </button>
              </div>
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<!-- Create blog dialog -->
<Dialog open={dialogOpen} onOpenChange={({ open: o }) => (dialogOpen = o)}>
  <Portal>
    <Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-950/50" />
    <Dialog.Positioner
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
    >
      <Dialog.Content
        class="w-full max-w-md space-y-4 card rounded-container preset-filled-surface-100-900 p-6 shadow-xl"
      >
        <Dialog.Title class="text-lg font-bold">Create a new blog</Dialog.Title>

        <form onsubmit={handleSubmit} class="space-y-4">
          <label class="label" for="blog-name">
            <span class="label-text">Blog name</span>
            <input
              id="blog-name"
              type="text"
              placeholder="My Awesome Blog"
              class="input w-full"
              bind:value={blogName}
              required
            />
          </label>

          <label class="label" for="blog-slug">
            <span class="label-text">Slug (auto-generated)</span>
            <input
              id="blog-slug"
              type="text"
              class="input w-full font-mono"
              value={slug}
              readonly
            />
            <p class="mt-1 text-sm text-surface-600-400">
              Used as directory and git repo name
            </p>
          </label>

          <footer class="flex justify-end gap-2">
            <Dialog.CloseTrigger class="btn preset-tonal-surface">
              Cancel
            </Dialog.CloseTrigger>
            <button
              type="submit"
              class="btn preset-filled-primary-500"
              disabled={!blogName.trim()}
            >
              Create
            </button>
          </footer>
        </form>
      </Dialog.Content>
    </Dialog.Positioner>
  </Portal>
</Dialog>
