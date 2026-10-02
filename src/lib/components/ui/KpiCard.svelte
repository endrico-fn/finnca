<script lang="ts">
  import type { Snippet } from 'svelte';
  import Badge from './Badge.svelte';
  import type { BadgeTone } from './badgeTone';

  let {
    label,
    value = '',
    subValue,
    labelClass = 'text-text-muted',
    valueClass = 'text-text-base',
    badge = '',
    badgeTone = 'neutral',
    selected = false,
    onclick,
    children,
  }: {
    label: string;
    value?: string;
    subValue?: string;
    labelClass?: string;
    valueClass?: string;
    badge?: string;
    badgeTone?: BadgeTone;
    selected?: boolean;
    onclick?: () => void;
    children?: Snippet;
  } = $props();

  const rootCls = $derived(
    `sharp-card space-y-1.5 px-3 pt-2.5 pb-2.5 ${selected ? 'border-teal/60 bg-bg-row-active' : ''} ${onclick ? 'w-full cursor-pointer text-left transition-colors hover:border-text-dim' : ''}`
  );
</script>

{#if onclick}
  <button type="button" {onclick} aria-pressed={selected} class={rootCls}>
    <div class="flex min-h-5 items-center justify-between gap-1.5 leading-none">
      <span class="label-xs font-proto truncate font-bold {selected ? 'text-teal' : labelClass}">
        {label}
      </span>
      {#if badge}
        <Badge size="m" tone={selected ? 'teal' : badgeTone}>{badge}</Badge>
      {/if}
    </div>
    {#if children}
      {@render children()}
    {:else}
      <span class="font-proto text-medium block leading-tight font-bold tabular-nums {valueClass}">
        {value}
      </span>
      {#if subValue}
        <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
          {subValue}
        </span>
      {/if}
    {/if}
  </button>
{:else}
  <div class={rootCls}>
    <div class="flex min-h-5 items-center justify-between gap-1.5 leading-none">
      <span class="label-xs font-proto truncate font-bold {labelClass}">
        {label}
      </span>
      {#if badge}
        <Badge size="m" tone={badgeTone}>{badge}</Badge>
      {/if}
    </div>
    {#if children}
      {@render children()}
    {:else}
      <span class="font-proto text-medium block leading-tight font-bold tabular-nums {valueClass}">
        {value}
      </span>
      {#if subValue}
        <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
          {subValue}
        </span>
      {/if}
    {/if}
  </div>
{/if}
