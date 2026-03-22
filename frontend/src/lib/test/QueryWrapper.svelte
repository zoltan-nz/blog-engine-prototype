<script lang="ts">
  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";

  const { children } = $props();

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, enabled: false },
    },
  });
</script>

<QueryClientProvider client={queryClient}>
  {@render children()}
</QueryClientProvider>
