<script lang="ts">
  import { Dialog, Portal } from "@skeletonlabs/skeleton-svelte";
  import {
    createListSites,
    createCreateSite,
    createPreviewSite,
  } from "../generated-api.js";
  import type { CreateSiteRequest } from "../generated-api.js";

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

  const sitesQuery = createListSites();
  const createSiteMutation = createCreateSite();
  const previewMutation = createPreviewSite();
  let previewingSlug = $state<string | null>(null);

  let dialogOpen = $state(false);
  let blogName = $state("");
  let slug = $derived(nameToSlug(blogName));

  function openModal() {
    blogName = "";
    dialogOpen = true;
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!blogName.trim() || !slug) return;

    const body: CreateSiteRequest = { name: blogName.trim(), slug };
    await createSiteMutation.mutateAsync({ data: body });

    dialogOpen = false;
    await sitesQuery.refetch();
  }

  async function handlePreview(slug: string) {
    previewingSlug = slug;
    try {
      const result = await previewMutation.mutateAsync({ slug });
      if (result.status === 200 && result.data.data.previewUrl) {
        window.open(result.data.data.previewUrl, "_blank");
        sitesQuery.refetch();
      }
    } finally {
      previewingSlug = null;
    }
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

  <!-- Site list -->
  {#if sitesQuery.isLoading}
    <div class="flex justify-center py-12">
      <svg
        class="size-12 animate-spin text-primary-500"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
      >
        <circle
          class="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          stroke-width="4"
        ></circle>
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
        ></path>
      </svg>
    </div>
  {:else if sitesQuery.isError}
    <div class="rounded-container preset-tonal-error p-4">
      <span>Failed to load sites. Is the backend running?</span>
    </div>
  {:else}
    {@const sites = sitesQuery.data?.data.data ?? []}
    {#if sites.length === 0}
      <div
        class="card rounded-container preset-tonal-surface py-16 text-center"
      >
        <p class="text-surface-600-400">No blogs yet. Create your first one!</p>
      </div>
    {:else}
      <ul class="grid gap-4 sm:grid-cols-2">
        {#each sites as site (site.slug)}
          {@const isPreviewing = previewingSlug === site.slug}
          <li
            class="card rounded-container border border-surface-200-800 preset-filled-surface-100-900 shadow-sm"
          >
            <div class="p-4">
              <div class="flex items-start justify-between">
                <div>
                  <h2 class="flex items-center gap-2 text-xl font-bold">
                    {site.name}
                    {#if site.previewUrl}
                      <span class="badge preset-filled-success-500 text-xs"
                        >▶ Live</span
                      >
                    {/if}
                  </h2>
                  <p class="font-mono text-sm text-surface-600-400">
                    {site.slug}
                  </p>
                  <p class="text-surface-500-400 truncate text-xs">
                    {site.gitUrl}
                  </p>
                </div>
              </div>
              <div class="mt-4 flex gap-2">
                <button
                  class="btn preset-outlined-surface-300-700 btn-sm"
                  disabled={isPreviewing}
                  onclick={() => handlePreview(site.slug)}
                >
                  {#if isPreviewing}
                    <svg
                      class="size-4 animate-spin"
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      ></circle>
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                      ></path>
                    </svg>
                  {/if}
                  Preview
                </button>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
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

          {#if createSiteMutation.isError}
            <div class="rounded-container preset-tonal-error p-3 text-sm">
              Failed to create blog. Please try again.
            </div>
          {/if}

          <footer class="flex justify-end gap-2">
            <Dialog.CloseTrigger class="btn preset-tonal-surface">
              Cancel
            </Dialog.CloseTrigger>
            <button
              type="submit"
              class="btn preset-filled-primary-500"
              disabled={!blogName.trim() || createSiteMutation.isPending}
            >
              {#if createSiteMutation.isPending}
                <svg
                  class="size-4 animate-spin"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                  ></circle>
                  <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                  ></path>
                </svg>
                Creating…
              {:else}
                Create
              {/if}
            </button>
          </footer>
        </form>
      </Dialog.Content>
    </Dialog.Positioner>
  </Portal>
</Dialog>
