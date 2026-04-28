<script lang="ts">
  import { Popover, Portal } from "@skeletonlabs/skeleton-svelte";
  import {
    themeFontFamilyMap,
    type ThemeName,
    themeNames,
  } from "./theme-names.ts";
  import { fontStore } from "$lib/state/font.svelte";
  import ColorSwatch from "./color-swatch.svelte";
  import { ChevronDownIcon, Check } from "@lucide/svelte";

  type Mode = "system" | "light" | "dark";

  let selectedTheme = $state<ThemeName>(
    (localStorage.getItem("theme") as ThemeName) ?? "cerberus",
  );
  let mode = $state<Mode>((localStorage.getItem("mode") as Mode) ?? "system");

  // Keep data-mode in sync when OS preference changes (system mode only)
  $effect(() => {
    if (mode !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const sync = (e: MediaQueryListEvent) => {
      document.documentElement.setAttribute(
        "data-mode",
        e.matches ? "dark" : "light",
      );
    };
    mq.addEventListener("change", sync);
    return () => mq.removeEventListener("change", sync);
  });

  function changeTheme(theme: ThemeName): void {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
    selectedTheme = theme;
    fontStore.applyByFamily(themeFontFamilyMap[theme]);
  }

  function setMode(m: Mode): void {
    mode = m;
    localStorage.setItem("mode", m);

    switch (m) {
      case "dark":
        document.documentElement.setAttribute("data-mode", "dark");
        break;
      case "light":
        document.documentElement.setAttribute("data-mode", "light");
        break;
      case "system":
        const prefersDark = window.matchMedia(
          "(prefers-color-scheme: dark)",
        ).matches;
        document.documentElement.setAttribute(
          "data-mode",
          prefersDark ? "dark" : "light",
        );
        break;
    }
  }
</script>

<Popover>
  <Popover.Trigger
    class="btn flex items-center gap-2 preset-tonal-surface btn-sm"
  >
    <ColorSwatch themeName={selectedTheme} />
    <span class="hidden text-xs capitalize sm:inline">{selectedTheme}</span>
    <ChevronDownIcon class="size-4 opacity-60" />
  </Popover.Trigger>
  <Portal>
    <Popover.Positioner>
      <Popover.Content
        class="z-50 w-56 overflow-hidden card rounded-container border border-surface-200-800 preset-filled-surface-100-900 shadow-xl"
      >
        <!-- Theme list -->
        <ul class="max-h-64 overflow-y-auto p-1">
          {#each themeNames as theme}
            <li>
              <button
                class="flex w-full items-center gap-3 rounded px-2 py-1.5 text-xs capitalize transition-colors hover:preset-tonal-surface"
                class:preset-tonal-primary={selectedTheme === theme}
                onclick={() => changeTheme(theme)}
                role="option"
                aria-selected={selectedTheme === theme}
              >
                <ColorSwatch themeName={theme} />
                <span class="flex-1 text-left">{theme}</span>
                {#if selectedTheme === theme}
                  <Check class="size-4 shrink-0" />
                {/if}
              </button>
            </li>
          {/each}
        </ul>

        <!-- Mode toggle -->
        <div class="border-t border-surface-200-800 p-2">
          <p class="mb-1.5 px-1 text-xs text-surface-600-400">Mode</p>
          <div class="grid grid-cols-3 gap-1">
            {#each ["system", "light", "dark"] as const as m}
              <button
                class="btn btn-sm text-xs capitalize transition-colors"
                class:preset-filled-primary-500={mode === m}
                class:preset-tonal-surface={mode !== m}
                onclick={() => setMode(m)}
              >
                {m}
              </button>
            {/each}
          </div>
        </div>
      </Popover.Content>
    </Popover.Positioner>
  </Portal>
</Popover>
