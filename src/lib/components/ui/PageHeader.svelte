<script lang="ts">
  import type { Snippet } from 'svelte';

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
    class="font-proto flex shrink-0 items-center gap-1.5 text-[14px] tracking-[0.08em] uppercase"
  >
    {#if crumb}
      <a href={crumbHref} class="text-text-muted hover:text-text-strong transition-colors">
        {crumb}
      </a>
      <span class="text-text-muted font-bold">/</span>
      <span class="text-text-base font-semibold">{title}</span>
    {:else}
      <span class="text-text-base font-semibold">{title}</span>
    {/if}
  </nav>
  {#if actions}
    <div class="relative z-30 ml-auto flex shrink-0 items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</div>
