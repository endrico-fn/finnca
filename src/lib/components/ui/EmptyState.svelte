<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Icon, type IconName } from '$lib/components/ui';
  import Button from './Button.svelte';

  let {
    title,
    hint = '',
    icon,
    actionLabel = '',
    actionHref = '',
    onAction,
    children,
  }: {
    title: string;
    hint?: string;
    icon?: IconName;
    actionLabel?: string;
    actionHref?: string;
    onAction?: () => void;
    children?: Snippet;
  } = $props();
</script>

<div class="flex h-full flex-col items-center justify-center gap-2 py-10 text-center">
  {#if icon}
    <div class="text-text-dim mb-1"><Icon name={icon} size={24} /></div>
  {/if}
  <p class="text-text-muted font-proto text-small">{title}</p>
  {#if hint}
    <p class="text-text-muted/80 text-small max-w-sm">{hint}</p>
  {/if}
  {#if children}
    {@render children()}
  {/if}
  {#if actionLabel && onAction}
    <Button variant="ghost" size="sm" onclick={onAction} class="mt-2">
      {actionLabel}
    </Button>
  {:else if actionLabel && actionHref}
    <Button variant="ghost" size="sm" href={actionHref} class="mt-2">
      {actionLabel}
    </Button>
  {/if}
</div>
