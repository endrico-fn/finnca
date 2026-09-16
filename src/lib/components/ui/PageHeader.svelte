<script lang="ts">
  import type { Snippet } from 'svelte';
  import { resolve } from '$app/paths';

  let {
    crumb,
    crumbHref = '/app',
    title,
    actions,
  }: {
    crumb?: string;
    crumbHref?: string;
    title: string;
    actions?: Snippet;
  } = $props();
</script>

<div class="app-header relative z-30 gap-4">
  <nav
    class="font-proto text-medium flex shrink-0 items-center gap-2 tracking-[0.08em] uppercase"
  >
    {#if crumb}
      <a
        href={resolve(crumbHref as '/app')}
        class="text-text-muted hover:text-text-strong transition-colors"
      >
        {crumb}
      </a>
      <span class="text-text-muted/60 font-bold">/</span>
    {/if}
    <span class="text-text-strong font-semibold">{title}</span>
  </nav>
  {#if actions}
    <div class="relative z-30 ml-auto flex shrink-0 items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</div>
