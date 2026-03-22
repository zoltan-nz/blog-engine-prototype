<script lang="ts">
  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import Footer from "$lib/components/footer.svelte";

  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
</script>

<QueryClientProvider client={queryClient}>
  <Footer />
</QueryClientProvider>
