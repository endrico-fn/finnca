<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    layout = 'grid',
    children,
  }: {
    title: string;
    layout?: 'grid' | 'list';
    children: Snippet;
  } = $props();
</script>

<div class="px-2.5 pt-2 pb-2">
  <p class="font-proto text-text-muted text-smaller tracking-widest uppercase">{title}</p>
  {#if layout === 'grid'}
    <div class="mt-1.5 grid grid-cols-2 gap-1">
      {@render children()}
    </div>
  {:else}
    <div class="mt-1.5 flex max-h-60 flex-col gap-1 overflow-y-auto">
      {@render children()}
    </div>
  {/if}
</div>
