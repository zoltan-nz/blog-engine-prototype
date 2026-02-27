<script lang="ts">
  import { createGetHealthz } from "../../generated-api.js";
  import { backendURL } from "$lib/api/fetch-with-base-url.js";

  const checkConnection = createGetHealthz();
</script>

<footer
  class="border-t border-base-300 bg-base-100 px-8 py-4"
  data-testid="footer"
>
  <div class="mx-auto flex max-w-4xl items-center justify-between text-sm">
    <span class="text-base-content/60">Backend: {backendURL}</span>
    <span class="text-base-content/60"
      >Server Name: {checkConnection.data?.data.meta.serverName}</span
    >
    <span class="text-base-content/60"
      >Version: {checkConnection.data?.data.meta.version}</span
    >
    <button
      class="flex items-center gap-2 transition-opacity hover:opacity-80"
      onclick={() => checkConnection.refetch()}
      disabled={checkConnection.isLoading}
      title={checkConnection.isSuccess ? "Connected" : "Disconnected"}
    >
      {#if checkConnection.isLoading}
        <span class="loading loading-xs loading-spinner"></span>
      {:else}
        <span
          class="h-3 w-3 rounded-full {checkConnection.isSuccess
            ? 'bg-success'
            : 'bg-error'}"
        ></span>
      {/if}
      <span class="text-base-content/60">
        {checkConnection.isLoading
          ? "Checking..."
          : checkConnection.isSuccess
            ? "Connected"
            : "Disconnected"}
      </span>
    </button>
  </div>
</footer>
