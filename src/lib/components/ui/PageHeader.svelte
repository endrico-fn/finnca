<script lang="ts">
  import type { Snippet } from 'svelte';
  import { resolve } from '$app/paths';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    crumb,
    crumbHref = '/app',
    crumbAction = null,
    parentCrumb,
    parentHref = '/app',
    title,
    actions,
  }: {
    crumb?: string;
    crumbHref?: string;
    crumbAction?: (() => void) | null;
    parentCrumb?: string;
    parentHref?: string;
    title: string;
    actions?: Snippet;
  } = $props();
</script>

<header class="app-header relative z-30 gap-4">
  <nav
    aria-label={i18n.t.breadcrumbNav}
    class="font-proto text-medium tracking-section flex shrink-0 items-center gap-2 uppercase"
  >
    {#if parentCrumb}
      <a
        href={resolve(parentHref as '/app')}
        class="text-text-muted hover:text-text-strong transition-colors"
      >
        {parentCrumb}
      </a>
      <span class="text-text-muted/60 font-bold">/</span>
    {/if}
    {#if crumb}
      {#if crumbAction}
        <button
          type="button"
          onclick={crumbAction}
          class="font-proto text-medium tracking-section text-text-muted hover:text-text-strong cursor-pointer border-0 bg-transparent p-0 uppercase transition-colors"
        >
          {crumb}
        </button>
      {:else}
        <a
          href={resolve(crumbHref as '/app')}
          class="text-text-muted hover:text-text-strong transition-colors"
        >
          {crumb}
        </a>
      {/if}
      <span class="text-text-muted/60 font-bold">/</span>
    {/if}
    <h2 class="text-text-strong text-medium font-semibold">{title}</h2>
  </nav>
  {#if actions}
    <div class="relative z-30 ml-auto flex shrink-0 items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</header>
