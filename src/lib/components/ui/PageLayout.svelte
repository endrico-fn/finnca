<script lang="ts">
  import type { Snippet } from 'svelte';
  import PageHeader from './PageHeader.svelte';

  let {
    children,
    title,
    crumb,
    crumbHref,
    actions,
    class: className = '',
  }: {
    children: Snippet;
    title?: string;
    crumb?: string;
    crumbHref?: string;
    actions?: Snippet;
    class?: string;
  } = $props();
</script>

<div class="app-page {className}">
  {#if title}
    <PageHeader {title} {crumb} {crumbHref} {actions} />
  {/if}

  <div class="flex min-h-0 flex-1 flex-col {title ? 'pt-0.5' : ''}">
    {@render children()}
  </div>
</div>
