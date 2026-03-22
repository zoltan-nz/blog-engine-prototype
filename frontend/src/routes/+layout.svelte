<script lang="ts">
  import "./layout.css";
  import favicon from "$lib/assets/favicon.svg";
  import Footer from "$lib/components/footer.svelte";
  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import { browser } from "$app/environment";

  const { children } = $props();

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        enabled: browser,
      },
    },
  });
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<QueryClientProvider client={queryClient}>
  <div class="flex min-h-screen flex-col">
    <main class="container mx-auto flex-grow p-4">
      {@render children()}
    </main>

    <Footer />
  </div>
</QueryClientProvider>
