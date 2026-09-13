<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    value = '',
    subValue,
    labelClass = 'text-text-muted',
    valueClass = 'text-text-base',
    badge = '',
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
    selected?: boolean;
    onclick?: () => void;
    children?: Snippet;
  } = $props();

  const rootCls = $derived(
    `sharp-card space-y-1 px-3 pt-2.5 pb-2.5 ${selected ? 'border-teal/60 bg-bg-row-active' : ''} ${onclick ? 'w-full cursor-pointer text-left transition-colors hover:border-text-dim' : ''}`
  );
</script>

{#if onclick}
  <button type="button" {onclick} aria-pressed={selected} class={rootCls}>
    <span
      class="font-proto text-smaller flex items-center justify-between gap-1 leading-none tracking-wider uppercase {selected
        ? 'text-teal'
        : labelClass}"
    >
      <span class="truncate {selected ? 'font-semibold' : ''}">{label}</span>
      {#if badge}
        <span
          class="shrink-0 border px-1 whitespace-nowrap {selected
            ? 'border-teal text-teal'
            : 'border-line text-text-dim'}">{badge}</span
        >
      {/if}
    </span>
    {#if children}
      {@render children()}
    {:else}
      <span class="font-proto text-large block leading-tight font-bold tabular-nums {valueClass}">
        {value}
      </span>
      {#if subValue}
        <span class="text-text-dim font-proto text-smaller block leading-tight">
          {subValue}
        </span>
      {/if}
    {/if}
  </button>
{:else}
  <div class={rootCls}>
    <span
      class="font-proto text-smaller flex items-center justify-between gap-1 leading-none tracking-wider uppercase {labelClass}"
    >
      <span class="truncate">{label}</span>
      {#if badge}
        <span class="border-line text-text-dim shrink-0 border px-1 whitespace-nowrap">{badge}</span
        >
      {/if}
    </span>
    {#if children}
      {@render children()}
    {:else}
      <span class="font-proto text-large block leading-tight font-bold tabular-nums {valueClass}">
        {value}
      </span>
      {#if subValue}
        <span class="text-text-dim font-proto text-smaller block leading-tight">
          {subValue}
        </span>
      {/if}
    {/if}
  </div>
{/if}
