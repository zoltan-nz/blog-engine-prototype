<script lang="ts">
  import { Dialog, Portal } from "@skeletonlabs/skeleton-svelte";
  import { getSocket } from "$lib/state/socket.svelte";
  import type { PostMeta } from "$lib/types/bindings.js";
  import { Plus, Trash2 } from "@lucide/svelte";
  import { slugify } from "./slugify.js";

  let {
    siteSlug,
    selectedId = null,
    onSelect,
  }: {
    siteSlug: string;
    selectedId?: string | null;
    onSelect: (post: PostMeta) => void;
  } = $props();

  const socket = getSocket();
  const posts = $derived(socket.posts[siteSlug] ?? []);

  let dialogOpen = $state(false);
  let newTitle = $state("");
  let newId = $derived(slugify(newTitle));

  function openNewPostDialog(): void {
    newTitle = "";
    dialogOpen = true;
  }

  function handleSubmit(e: SubmitEvent): void {
    e.preventDefault();
    if (!newTitle.trim() || !newId) return;
    socket.createPost(siteSlug, newId, newTitle.trim(), "", "");
    dialogOpen = false;
  }

  function handleDelete(post: PostMeta, event: MouseEvent): void {
    event.stopPropagation();
    if (!confirm(`Delete "${post.title}"? This cannot be undone.`)) return;
    socket.deletePost(siteSlug, post.id);
  }
</script>

<div class="flex items-center justify-between">
  <h2 class="text-lg font-bold">Posts</h2>
  <button
    class="btn preset-outlined-surface-300-700 btn-sm"
    onclick={openNewPostDialog}
  >
    <Plus size={16} />New post
  </button>
</div>

{#if posts.length === 0}
  <p class="mt-4 text-surface-600-400">No posts yet. Create your first one!</p>
{:else}
  <ul class="mt-4 space-y-2">
    {#each posts as post (post.id)}
      <li
        class="flex items-center justify-between rounded-container preset-tonal-surface transition-colors hover:preset-tonal-primary"
        class:preset-filled-primary-500={post.id === selectedId}
      >
        <button
          type="button"
          class="flex-1 p-3 text-left"
          onclick={() => onSelect(post)}
        >
          <span class="block font-medium">{post.title}</span>
          <span class="block text-sm text-surface-600-400">{post.pub_date}</span
          >
        </button>
        <button
          type="button"
          aria-label={`Delete ${post.title}`}
          class="mr-2 btn-icon preset-tonal-error btn-icon-sm"
          onclick={(e) => handleDelete(post, e)}
        >
          <Trash2 size={14} />
        </button>
      </li>
    {/each}
  </ul>
{/if}

<Dialog open={dialogOpen} onOpenChange={({ open: o }) => (dialogOpen = o)}>
  <Portal>
    <Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-950/50" />
    <Dialog.Positioner
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
    >
      <Dialog.Content
        class="w-full max-w-md space-y-4 card rounded-container preset-filled-surface-100-900 p-6 shadow-xl"
      >
        <Dialog.Title class="text-lg font-bold">New post</Dialog.Title>

        <form onsubmit={handleSubmit} class="space-y-4">
          <label class="label" for="post-new-title">
            <span class="label-text">Title</span>
            <input
              id="post-new-title"
              type="text"
              placeholder="My First Post"
              class="input w-full"
              bind:value={newTitle}
              required
            />
          </label>

          <label class="label" for="post-new-id">
            <span class="label-text">Id (auto-generated)</span>
            <input
              id="post-new-id"
              type="text"
              class="input w-full font-mono"
              value={newId}
              readonly
            />
          </label>

          <footer class="flex justify-end gap-2">
            <Dialog.CloseTrigger class="btn preset-tonal-surface">
              Cancel
            </Dialog.CloseTrigger>
            <button
              type="submit"
              class="btn preset-filled-primary-500"
              disabled={!newTitle.trim()}
            >
              Create
            </button>
          </footer>
        </form>
      </Dialog.Content>
    </Dialog.Positioner>
  </Portal>
</Dialog>
