<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.svg";
    import Footer from "$lib/components/footer.svelte";

    const {children} = $props();
</script>

<svelte:head>
    <link rel="icon" href={favicon}/>
</svelte:head>

<div class="flex min-h-screen flex-col">
    <main class="container mx-auto flex-grow p-4">
        {@render children()}
    </main>


    <Footer/>
</div>
