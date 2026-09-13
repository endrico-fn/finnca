<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    open = $bindable(false),
    label,
    active = false,
    count = 0,
    onReset,
    resetLabel = '',
    resetDisabled = false,
    panelClass = 'w-64',
    class: className = '',
    children,
  }: {
    open?: boolean;
    label: string;
    active?: boolean;
    count?: number;
    onReset?: () => void;
    resetLabel?: string;
    resetDisabled?: boolean;
    panelClass?: string;
    class?: string;
    children: Snippet;
  } = $props();
</script>

<div class="relative shrink-0 {className}">
  <button
    type="button"
    onclick={() => (open = !open)}
    aria-haspopup="listbox"
    aria-expanded={open}
    class="font-proto inline-flex h-7 cursor-pointer items-center gap-2 border px-2.5 text-smaller uppercase transition-colors select-none {open ||
    active
      ? 'border-teal/60 bg-bg-row-active text-text-strong'
      : 'border-line bg-bg-card text-text-strong hover:border-text-dim hover:text-text-white'}"
  >
    {#if active}
      <span class="size-1.5 shrink-0 bg-teal"></span>
    {/if}
    <span class="font-semibold tracking-wider max-w-48 truncate">{label}</span>
    {#if count > 0}
      <span class="tabular-nums opacity-60">({count})</span>
    {/if}
    <span class="inline-flex transition-transform duration-150 {open ? 'rotate-180' : ''}">
      <Icon name="chev-down" size={10} />
    </span>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)}></div>
    <div
      class="border-line bg-bg-card font-proto absolute right-0 z-50 mt-1 border select-none {panelClass}"
    >
      {@render children()}
      {#if onReset}
        <div class="border-line/60 flex justify-end border-t px-2.5 py-1.5">
          <button
            type="button"
            onclick={onReset}
            disabled={resetDisabled}
            class="font-proto text-text-muted hover:text-text-base px-1 py-0.5 text-smaller uppercase transition-colors disabled:cursor-not-allowed disabled:opacity-40"
          >
            {resetLabel}
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
