<script lang="ts">
  import { Popover, Portal } from "@skeletonlabs/skeleton-svelte";
  import { fonts } from "./font-names.ts";
  import { fontStore } from "$lib/state/font.svelte";
  import { Check, Type, ChevronDown } from "@lucide/svelte";

  const currentFont = $derived(
    fonts.find((f) => f.family === fontStore.family),
  );
</script>

<Popover>
  <Popover.Trigger
    class="btn flex items-center gap-2 preset-tonal-surface btn-sm"
  >
    <Type class="size-4" />
    <span class="hidden text-xs sm:inline"
      >{currentFont?.name ?? fontStore.family}</span
    >
    <ChevronDown class="size-4 opacity-60" />
  </Popover.Trigger>
  <Portal>
    <Popover.Positioner>
      <Popover.Content
        class="z-50 w-56 overflow-hidden card rounded-container border border-surface-200-800 preset-filled-surface-100-900 shadow-xl"
      >
        <ul class="max-h-72 overflow-y-auto p-1">
          {#each fonts as font}
            <li>
              <button
                class="flex w-full items-center gap-3 rounded px-2 py-1.5 text-xs capitalize transition-colors hover:preset-tonal-surface"
                class:preset-tonal-primary={fontStore.family === font.family}
                onclick={() => {
                  fontStore.apply(font);
                }}
                role="option"
                aria-selected={fontStore.family === font.family}
              >
                <span
                  class="flex-1 text-left"
                  style="font-family: {font.cssStack}"
                >
                  {font.name}
                </span>
                {#if fontStore.family === font.family}
                  <Check class="size-4" />
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      </Popover.Content>
    </Popover.Positioner>
  </Portal>
</Popover>
