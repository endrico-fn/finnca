<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Icon } from '$lib/components/ui';

  let {
    title,
    hint = '',
    icon = '',
    actionLabel = '',
    actionHref = '',
    onAction,
    children,
  }: {
    title: string;
    hint?: string;
    icon?: string;
    actionLabel?: string;
    actionHref?: string;
    onAction?: () => void;
    children?: Snippet;
  } = $props();
</script>

<div class="flex h-full flex-col items-center justify-center gap-2 py-10 text-center">
  {#if icon}
    <div class="text-text-dim mb-1"><Icon name={icon as any} size={24} /></div>
  {/if}
  <p class="text-text-muted font-proto text-[12px]">{title}</p>
  {#if hint}
    <p class="text-text-muted/80 max-w-sm text-[11px]">{hint}</p>
  {/if}
  {#if children}
    {@render children()}
  {/if}
  {#if actionLabel && onAction}
    <button
      type="button"
      onclick={onAction}
      class="sharp-btn btn-ghost text-teal hover:border-teal mt-2 px-4 py-1.5 text-[11px]"
    >
      {actionLabel}
    </button>
  {:else if actionLabel && actionHref}
    <a
      href={actionHref}
      class="sharp-btn btn-ghost text-teal hover:border-teal mt-2 px-4 py-1.5 text-[11px]"
    >
      {actionLabel}
    </a>
  {/if}
</div>
