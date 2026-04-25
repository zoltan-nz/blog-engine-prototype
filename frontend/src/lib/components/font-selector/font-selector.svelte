<script lang="ts">
  import { fonts } from './font-names.ts';

  let selectedFamily = $state(localStorage.getItem('font-family') ?? 'Inter');

  const loadFont = (url: string): void => {
    if (document.querySelector(`link[href="${url}"]`)) return;
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = url;
    document.head.appendChild(link);
  };

  const changeFont = (family: string, googleParam: string): void => {
    const url = `https://fonts.googleapis.com/css2?family=${googleParam}&display=swap`;
    loadFont(url);
    document.documentElement.style.setProperty('--font-family', `"${family}", sans-serif`);
    localStorage.setItem('font-family', family);
    localStorage.setItem('font-url', url);
    selectedFamily = family;
  };
</script>

<div title="Change Font" class="dropdown dropdown-top">
  <div tabindex="0" role="button" class="btn group btn-sm gap-1.5 px-1.5 btn-ghost" aria-label="Change Font">
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none"
         stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
         class="opacity-60">
      <polyline points="4 7 4 4 20 4 20 7"></polyline>
      <line x1="9" y1="20" x2="15" y2="20"></line>
      <line x1="12" y1="4" x2="12" y2="20"></line>
    </svg>
    <span class="hidden text-xs sm:inline">Font</span>
    <svg width="12px" height="12px" class="mt-px hidden size-2 fill-current opacity-60 sm:inline-block"
         xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2048 2048">
      <path d="M1799 349l242 241-1017 1017L7 590l242-241 775 775 775-775z"></path>
    </svg>
  </div>
  <div tabindex="-1"
       class="dropdown-content bg-base-200 text-base-content rounded-box overflow-y-auto border-[length:var(--border)] border-white/5 shadow-2xl outline-[length:var(--border)] outline-black/5">
    <ul class="menu w-52">
      <li class="menu-title text-xs">Font</li>
      {#each fonts as font}
        <li>
          <button class="gap-3 px-2" onclick={() => changeFont(font.family, font.googleParam)}>
            <span class="w-36 truncate text-sm" style="font-family: '{font.family}', sans-serif">{font.name}</span>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"
                 class="h-3 w-3 shrink-0 {selectedFamily === font.family ? 'visible' : 'invisible'}">
              <path d="M20.285 2l-11.285 11.567-5.286-5.011-3.714 3.716 9 8.728 15-15.285z"></path>
            </svg>
          </button>
        </li>
      {/each}
    </ul>
  </div>
</div>
