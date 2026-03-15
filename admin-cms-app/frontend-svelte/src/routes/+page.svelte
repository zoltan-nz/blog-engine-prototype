<script lang="ts">
  import { createListSites, createCreateSite } from '../generated-api.js';
  import type { CreateSiteRequest } from '../generated-api.js';

  // Converts a blog name into a URL-safe slug (directory name + git repo name).
  // Rules: GitHub repo names allow [a-z0-9._-], max 100 chars.
  function nameToSlug(name: string): string {
    return name
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 100);
  }

  const sitesQuery = createListSites();
  const createSiteMutation = createCreateSite();

  let dialogEl = $state<HTMLDialogElement | null>(null);
  let blogName = $state('');
  let slug = $derived(nameToSlug(blogName));

  function openModal() {
    blogName = '';
    dialogEl?.showModal();
  }

  function closeModal() {
    dialogEl?.close();
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!blogName.trim() || !slug) return;

    const body: CreateSiteRequest = { name: blogName.trim(), slug };
    await createSiteMutation.mutateAsync({ data: body });

    closeModal();
    sitesQuery.refetch();
  }
</script>

<div class="mx-auto max-w-4xl">
  <div class="mb-8 flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold">Blog Engine Admin</h1>
      <p class="text-base-content/70 mt-1">Manage your Astro blog sites.</p>
    </div>
    <button class="btn btn-primary" onclick={openModal}>Create a new blog</button>
  </div>

  <!-- Site list -->
  {#if sitesQuery.isLoading}
    <div class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
  {:else if sitesQuery.isError}
    <div class="alert alert-error">
      <span>Failed to load sites. Is the backend running?</span>
    </div>
  {:else}
    {@const sites = sitesQuery.data?.data.data ?? []}
    {#if sites.length === 0}
      <div class="card bg-base-200 py-16 text-center">
        <p class="text-base-content/60">No blogs yet. Create your first one!</p>
      </div>
    {:else}
      <ul class="grid gap-4 sm:grid-cols-2">
        {#each sites as site (site.slug)}
          <li class="card bg-base-100 border-base-300 border shadow-sm">
            <div class="card-body">
              <h2 class="card-title">{site.name}</h2>
              <p class="text-base-content/60 font-mono text-sm">{site.slug}</p>
              <p class="text-base-content/50 truncate text-xs">{site.gitUrl}</p>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<!-- Create blog modal -->
<dialog bind:this={dialogEl} class="modal">
  <div class="modal-box">
    <h3 class="mb-4 text-lg font-bold">Create a new blog</h3>

    <form onsubmit={handleSubmit}>
      <div class="form-control mb-4">
        <label class="label" for="blog-name">
          <span class="label-text">Blog name</span>
        </label>
        <input
          id="blog-name"
          type="text"
          placeholder="My Awesome Blog"
          class="input input-bordered w-full"
          bind:value={blogName}
          required
        />
      </div>

      <div class="form-control mb-6">
        <label class="label" for="blog-slug">
          <span class="label-text">Slug (auto-generated)</span>
        </label>
        <input
          id="blog-slug"
          type="text"
          class="input input-bordered w-full font-mono"
          value={slug}
          readonly
        />
        <p class="label label-text-alt text-base-content/50">Used as directory and git repo name</p>
      </div>

      {#if createSiteMutation.isError}
        <div class="alert alert-error mb-4">
          <span>Failed to create blog. Please try again.</span>
        </div>
      {/if}

      <div class="modal-action">
        <button type="button" class="btn btn-ghost" onclick={closeModal}>Cancel</button>
        <button
          type="submit"
          class="btn btn-primary"
          disabled={!blogName.trim() || createSiteMutation.isPending}
        >
          {#if createSiteMutation.isPending}
            <span class="loading loading-spinner loading-sm"></span>
            Creating…
          {:else}
            Create
          {/if}
        </button>
      </div>
    </form>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>
