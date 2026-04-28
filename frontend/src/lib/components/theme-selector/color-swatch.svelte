<script lang="ts">
  export let themeName: string;
</script>

<span
  data-theme={themeName}
  class="grid shrink-0 grid-cols-2 gap-px rounded p-1"
>
  <span class="size-2 rounded-full bg-primary-500"></span>
  <span class="size-2 rounded-full bg-secondary-500"></span>
  <span class="size-2 rounded-full bg-tertiary-500"></span>
  <span class="size-2 rounded-full bg-surface-400"></span>
</span>
