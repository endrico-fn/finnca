<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
</script>

<div class="flex min-h-0 flex-1 animate-pulse flex-col gap-2 overflow-hidden" aria-hidden="true">
  <div class="grid shrink-0 grid-cols-12 gap-2">
    {#each [0, 1, 2, 3] as i (i)}
      <div class="border-line bg-bg-card col-span-12 h-20 border sm:col-span-6 lg:col-span-3"></div>
    {/each}
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-12 gap-2 overflow-hidden">
    <div class="border-line bg-bg-card col-span-12 h-full lg:col-span-8"></div>
    <div class="border-line bg-bg-card col-span-12 h-full lg:col-span-4"></div>
  </div>

  <div class="border-line bg-bg-card h-24 shrink-0 border"></div>

  <span class="font-proto text-text-dim text-smaller tracking-wider uppercase">
    {i18n.t.loadingFinnca}
  </span>
</div>
