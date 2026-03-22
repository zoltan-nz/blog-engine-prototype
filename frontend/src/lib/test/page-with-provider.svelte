<script lang="ts">
  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import Page from "../../routes/+page.svelte";

  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, enabled: false } },
  });
</script>

<QueryClientProvider client={queryClient}>
  <Page />
</QueryClientProvider>
