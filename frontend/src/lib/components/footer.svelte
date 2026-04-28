<script lang="ts">
  import { createHealthz } from "../../generated-api.js";
  import { backendURL } from "$lib/api/fetch-with-base-url.js";
  import ThemeSelector from "./theme-selector/theme-selector.svelte";
  import FontSelector from "./font-selector/font-selector.svelte";

  const checkConnection = createHealthz();
</script>

<footer
  class="border-t border-surface-200-800 bg-surface-50-950 px-8 py-4"
  data-testid="footer"
>
  <div
    class="mx-auto flex max-w-4xl items-center justify-between gap-4 text-sm"
  >
    <ThemeSelector />
    <FontSelector />
    <span class="text-surface-600-400">Backend: {backendURL}</span>
    <span class="text-surface-600-400">
      Server Name: {checkConnection.data?.data.meta.serverName}
    </span>
    <span class="text-surface-600-400">
      Version: {checkConnection.data?.data.meta.version}
    </span>
    <button
      class="flex items-center gap-2 transition-opacity hover:opacity-80"
      onclick={() => checkConnection.refetch()}
      disabled={checkConnection.isLoading}
      title={checkConnection.isSuccess ? "Connected" : "Disconnected"}
    >
      {#if checkConnection.isLoading}
        <svg
          class="size-3 animate-spin"
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
      {:else}
        <span
          class="size-3 rounded-full {checkConnection.isSuccess
            ? 'bg-success-500'
            : 'bg-error-500'}"
        ></span>
      {/if}
      <span class="text-surface-600-400">
        {checkConnection.isLoading
          ? "Checking..."
          : checkConnection.isSuccess
            ? "Connected"
            : "Disconnected"}
      </span>
    </button>
  </div>
</footer>
