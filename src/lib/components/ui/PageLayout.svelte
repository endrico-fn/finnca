<script lang="ts">
  import type { Snippet } from 'svelte';
  import PageHeader from './PageHeader.svelte';

  let {
    children,
    title,
    crumb,
    crumbHref,
    crumbAction = null,
    parentCrumb,
    parentHref,
    actions,
    class: className = '',
  }: {
    children: Snippet;
    title?: string;
    crumb?: string;
    crumbHref?: string;
    crumbAction?: (() => void) | null;
    parentCrumb?: string;
    parentHref?: string;
    actions?: Snippet;
    class?: string;
  } = $props();
</script>

<section aria-label={title} class="app-page {className}">
  {#if title}
    <PageHeader {title} {crumb} {crumbHref} {crumbAction} {parentCrumb} {parentHref} {actions} />
  {/if}

  <div class="flex min-h-0 flex-1 flex-col">
    {@render children()}
  </div>
</section>
