<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    open = $bindable(false),
    label,
    active = false,
    count = 0,
    onReset,
    resetLabel = '',
    resetDisabled = false,
    panelClass = 'w-64',
    align = 'right',
    onOpenChange = null,
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
    align?: 'left' | 'right';
    onOpenChange?: ((open: boolean) => void) | null;
    class?: string;
    children: Snippet;
  } = $props();

  function toggle() {
    const next = !open;
    if (onOpenChange) onOpenChange(next);
    else open = next;
  }

  function setClosed() {
    if (onOpenChange) onOpenChange(false);
    else open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      e.stopPropagation();
      setClosed();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="relative shrink-0 {className}">
  <button
    type="button"
    onclick={toggle}
    aria-haspopup="listbox"
    aria-expanded={open}
    class="font-proto text-smaller inline-flex h-7 cursor-pointer items-center gap-2 border px-2.5 uppercase transition-colors select-none {open ||
    active
      ? 'border-teal/60 bg-bg-row-active text-text-strong'
      : 'border-line bg-bg-card text-text-strong hover:border-text-dim hover:text-text-white'}"
  >
    {#if active}
      <span class="bg-teal size-1.5 shrink-0"></span>
    {/if}
    <span class="max-w-48 truncate font-semibold tracking-wider">{label}</span>
    {#if count > 0}
      <span class="tabular-nums opacity-60">({count})</span>
    {/if}
    <span
      class="inline-flex transition-transform duration-150 {open ? 'text-teal rotate-180' : ''}"
    >
      <Icon name="chev-down" size={10} />
    </span>
  </button>

  {#if open}
    <button
      type="button"
      aria-label={i18n.t.closeBtn}
      class="fixed inset-0 z-[var(--z-popover)] cursor-default"
      onclick={setClosed}
    ></button>
    <div
      class="border-line bg-bg-card font-proto absolute {align === 'left'
        ? 'left-0'
        : 'right-0'} z-[var(--z-popover)] mt-1 border shadow-xl select-none {panelClass}"
    >
      {@render children()}
      {#if onReset}
        <div class="border-line/60 flex justify-end border-t px-2.5 py-1.5">
          <button
            type="button"
            onclick={onReset}
            disabled={resetDisabled}
            class="font-proto text-text-muted hover:text-text-base text-smaller px-1 py-0.5 uppercase transition-colors disabled:cursor-not-allowed disabled:opacity-40"
          >
            {resetLabel}
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
