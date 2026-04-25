<script lang="ts">
  import { themeNames } from './theme-names.ts';

  let selectedTheme = $state(localStorage.getItem('theme') ?? '');

  const changeTheme = (themeName: string): void => {
    document.documentElement.setAttribute('data-theme', themeName);
    localStorage.setItem('theme', themeName);
    selectedTheme = themeName;
  };
</script>

<div title="Change Theme" class="dropdown dropdown-top">
  <div tabindex="0" role="button" class="btn group btn-sm gap-1.5 px-1.5 btn-ghost" aria-label="Change Theme">
    <div
      class="bg-base-100 group-hover:border-base-content/20 border-base-content/10 grid shrink-0 grid-cols-2 gap-0.5 rounded-md border p-1 transition-colors">
      <div class="bg-base-content size-1 rounded-full"></div>
      <div class="bg-primary size-1 rounded-full"></div>
      <div class="bg-secondary size-1 rounded-full"></div>
      <div class="bg-accent size-1 rounded-full"></div>
    </div>
    <span class="hidden text-xs sm:inline">Theme</span>
    <svg width="12px" height="12px" class="mt-px hidden size-2 fill-current opacity-60 sm:inline-block"
         xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2048 2048">
      <path d="M1799 349l242 241-1017 1017L7 590l242-241 775 775 775-775z"></path>
    </svg>
  </div>
  <div tabindex="-1"
       class="dropdown-content bg-base-200 text-base-content rounded-box h-[30.5rem] max-h-[calc(100vh-8.6rem)] overflow-y-auto border-[length:var(--border)] border-white/5 shadow-2xl outline-[length:var(--border)] outline-black/5">
    <ul tabindex="-1" class="menu w-56">
      <li tabindex="-1" class="menu-title text-xs">Theme</li>
      {#each themeNames as theme}
        <li>
          <button class="gap-3 px-2" onclick={() => changeTheme(theme)}>
            <div data-theme={theme} class="bg-base-100 grid shrink-0 grid-cols-2 gap-0.5 rounded-md p-1 shadow-sm">
              <div class="bg-base-content size-1 rounded-full"></div>
              <div class="bg-primary size-1 rounded-full"></div>
              <div class="bg-secondary size-1 rounded-full"></div>
              <div class="bg-accent size-1 rounded-full"></div>
            </div>
            <div class="w-32 truncate">{theme}</div>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"
                 class="h-3 w-3 shrink-0 {selectedTheme === theme ? 'visible' : 'invisible'}">
              <path d="M20.285 2l-11.285 11.567-5.286-5.011-3.714 3.716 9 8.728 15-15.285z"></path>
            </svg>
          </button>
        </li>
      {/each}
    </ul>
  </div>
</div>
