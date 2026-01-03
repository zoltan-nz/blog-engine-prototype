<script lang="ts">
  import { onMount } from "svelte";
  import { env } from "$env/dynamic/public";

  let connected = $state(false);
  let loading = $state(true);

  const backendUrl = env.PUBLIC_BACKEND_URL || "http://localhost:8080";

  async function checkConnection() {
    loading = true;
    try {
      const res = await fetch(`${backendUrl}/healthz`);
      const data = await res.json();
      connected = res.ok && data.data?.status === "healthy";
    } catch {
      connected = false;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    checkConnection();
  });
</script>

<footer class="border-t border-base-300 bg-base-100 px-8 py-4">
  <div class="mx-auto flex max-w-4xl items-center justify-between text-sm">
    <span class="text-base-content/60">Backend: {backendUrl}</span>
    <button
      class="flex items-center gap-2 transition-opacity hover:opacity-80"
      onclick={checkConnection}
      disabled={loading}
      title={connected ? "Connected" : "Disconnected"}
    >
      {#if loading}
        <span class="loading loading-xs loading-spinner"></span>
      {:else}
        <span
          class="h-3 w-3 rounded-full {connected ? 'bg-success' : 'bg-error'}"
        ></span>
      {/if}
      <span class="text-base-content/60">
        {loading ? "Checking..." : connected ? "Connected" : "Disconnected"}
      </span>
    </button>
  </div>
</footer>
