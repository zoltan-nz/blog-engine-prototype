<script lang="ts">
  import { backendUrl, getSocket } from "$lib/state/socket.svelte";
  import ThemeSelector from "./theme-selector/theme-selector.svelte";
  import FontSelector from "./font-selector/font-selector.svelte";

  const socket = getSocket();

  interface HealthMeta {
    serverName: string;
    version: string;
  }

  // One-shot identity fetch; liveness comes from the WS status, which is
  // reactive and reconnect-aware.
  async function fetchHealthMeta(): Promise<HealthMeta | null> {
    try {
      const response = await fetch(`${backendUrl}/healthz`);
      if (!response.ok) return null;
      const body = await response.json();
      return { serverName: body.meta.serverName, version: body.meta.version };
    } catch {
      return null;
    }
  }

  let health = $state<HealthMeta | null>(null);
  fetchHealthMeta().then((meta) => (health = meta));

  let connected = $derived(socket.status === "open");
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
    <span class="text-surface-600-400">Backend: {backendUrl}</span>
    <span class="text-surface-600-400">
      Server Name: {health?.serverName ?? "—"}
    </span>
    <span class="text-surface-600-400">
      Version: {health?.version ?? "—"}
    </span>
    <span class="flex items-center gap-2" title={socket.status}>
      <span
        class="size-3 rounded-full {connected
          ? 'bg-success-500'
          : 'bg-error-500'}"
      ></span>
      <span class="text-surface-600-400">
        {connected ? "Connected" : "Disconnected"}
      </span>
    </span>
  </div>
</footer>
