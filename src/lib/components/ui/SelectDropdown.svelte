<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import Icon from './Icon.svelte';

  export interface DropdownOption {
    value: string;
    label: string;
    sublabel?: string;
  }

  let {
    options,
    value = $bindable(),
    placeholder = '',
    onSelect,
    size = 'md',
    placement = 'auto',
    class: className = '',
    menuClass = '',
    disabled = false,
    searchable = false,
    searchPlaceholder,
  }: {
    options: DropdownOption[];
    value: string;
    placeholder?: string;
    onSelect?: (val: string) => void;
    size?: 'sm' | 'md';
    placement?: 'auto' | 'top' | 'bottom';
    class?: string;
    menuClass?: string;
    disabled?: boolean;
    searchable?: boolean;
    searchPlaceholder?: string;
  } = $props();

  let open = $state(false);
  let query = $state('');
  let containerEl: HTMLElement | null = $state(null);
  let buttonEl: HTMLButtonElement | null = $state(null);
  let searchInputEl: HTMLInputElement | null = $state(null);
  let highlightedIndex = $state(0);
  let optionRefs: HTMLElement[] = [];
  let menuCoords = $state({
    top: 0,
    left: 0,
    minWidth: 0,
    maxWidth: 320,
    maxHeight: 240,
    openUpward: false,
  });

  const selectedOption = $derived(options.find((o) => o.value === value));

  const visibleOptions = $derived(
    !searchable || !query.trim()
      ? options
      : options.filter((o) =>
          `${o.label} ${o.sublabel ?? ''}`.toLowerCase().includes(query.toLowerCase().trim())
        )
  );

  $effect(() => {
    if (open) {
      const idx = visibleOptions.findIndex((o) => o.value === value);
      highlightedIndex = idx >= 0 ? idx : 0;
      if (searchable) {
        setTimeout(() => searchInputEl?.focus(), 0);
      }
    }
  });

  function scrollHighlightedIntoView() {
    const el = optionRefs[highlightedIndex];
    if (el) {
      el.scrollIntoView({ block: 'nearest' });
    }
  }

  function updateCoords() {
    if (!containerEl || typeof window === 'undefined') return;
    const rect = containerEl.getBoundingClientRect();
    if (rect.bottom < 0 || rect.top > window.innerHeight) {
      open = false;
      return;
    }
    const spaceBelow = window.innerHeight - rect.bottom;
    const spaceAbove = rect.top;
    const openUp =
      placement === 'top'
        ? true
        : placement === 'bottom'
          ? false
          : spaceBelow < 220 && spaceAbove > spaceBelow;

    const minWidth = rect.width;
    const maxWidth = Math.min(320, window.innerWidth - 16);
    const left = Math.max(8, Math.min(window.innerWidth - minWidth - 8, rect.left));
    const availableHeight = openUp ? spaceAbove - 16 : spaceBelow - 16;
    const maxHeight = Math.max(100, Math.min(240, Math.floor(availableHeight)));

    menuCoords = {
      top: openUp ? rect.top : rect.bottom,
      left,
      minWidth,
      maxWidth,
      maxHeight,
      openUpward: openUp,
    };
  }

  function toggle() {
    if (disabled) return;
    if (!open && containerEl) {
      updateCoords();
      query = '';
      open = true;
    } else {
      open = false;
    }
  }

  function handleContainerKeydown(e: KeyboardEvent) {
    if (disabled) return;
    if (!open) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        toggle();
      }
      return;
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (visibleOptions.length > 0) {
        highlightedIndex = (highlightedIndex + 1) % visibleOptions.length;
        scrollHighlightedIntoView();
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (visibleOptions.length > 0) {
        highlightedIndex = (highlightedIndex - 1 + visibleOptions.length) % visibleOptions.length;
        scrollHighlightedIntoView();
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const opt = visibleOptions[highlightedIndex];
      if (opt) {
        value = opt.value;
        onSelect?.(opt.value);
        open = false;
        buttonEl?.focus();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      open = false;
      buttonEl?.focus();
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }

  function handleScrollOrResize() {
    if (open) {
      updateCoords();
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    window.addEventListener('scroll', handleScrollOrResize, true);
    window.addEventListener('resize', handleScrollOrResize);
    return () => {
      window.removeEventListener('click', handleClickOutside);
      window.removeEventListener('scroll', handleScrollOrResize, true);
      window.removeEventListener('resize', handleScrollOrResize);
    };
  });
</script>

<div
  class="relative inline-block {className}"
  bind:this={containerEl}
  onkeydown={handleContainerKeydown}
  role="presentation"
>
  <button
    type="button"
    bind:this={buttonEl}
    {disabled}
    onclick={toggle}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-controls={open ? 'select-dropdown-listbox' : undefined}
    aria-label={placeholder || i18n.t.selectDefaultPlaceholder}
    class="sharp-btn {size === 'sm'
      ? 'text-smaller h-7 px-2'
      : 'text-small h-8 px-2.5'} font-proto inline-flex w-full cursor-pointer items-center justify-between gap-2 border transition-colors select-none disabled:cursor-not-allowed disabled:opacity-40 {open
      ? 'border-teal/60 bg-bg-row-active text-text-strong'
      : 'border-line bg-bg-card text-text-strong hover:border-text-dim'}"
  >
    <span class="truncate"
      >{selectedOption?.label ?? placeholder ?? i18n.t.selectDefaultPlaceholder}</span
    >
    <span
      class="text-text-muted inline-flex shrink-0 transition-transform duration-150 {open
        ? 'text-teal rotate-180'
        : ''}"
    >
      <Icon name="chev-down" size={size === 'sm' ? 8 : 9} />
    </span>
  </button>

  {#if open}
    <div
      id="select-dropdown-listbox"
      role="listbox"
      aria-label={placeholder || i18n.t.selectDefaultPlaceholder}
      aria-activedescendant={visibleOptions[highlightedIndex]
        ? `select-opt-${visibleOptions[highlightedIndex].value}`
        : undefined}
      tabindex="-1"
      class="border-line bg-bg-card text-small font-proto fixed z-[var(--z-popover)] overflow-y-auto border py-1 shadow-xl select-none {menuClass}"
      style="left: {menuCoords.left}px; {menuCoords.openUpward
        ? `bottom: ${typeof window !== 'undefined' ? window.innerHeight - menuCoords.top + 4 : 0}px;`
        : `top: ${menuCoords.top + 4}px;`} min-width: {menuCoords.minWidth}px; max-width: {menuCoords.maxWidth}px; max-height: {menuCoords.maxHeight}px;"
    >
      {#if searchable}
        <div class="border-line/60 bg-bg-card sticky top-0 border-b px-2 py-1.5">
          <div
            class="border-line bg-bg-app focus-within:border-teal flex h-6 items-center gap-1.5 border px-2 transition-colors"
          >
            <span class="text-text-base inline-flex">
              <Icon name="search" size={10} />
            </span>
            <input
              bind:this={searchInputEl}
              type="text"
              bind:value={query}
              placeholder={searchPlaceholder ?? i18n.t.searchAllPlaceholder}
              aria-label={searchPlaceholder ?? i18n.t.searchAllPlaceholder}
              autocomplete="off"
              spellcheck="false"
              class="text-text-strong placeholder:text-text-muted text-small font-aux h-full w-full bg-transparent outline-none"
            />
          </div>
        </div>
      {/if}
      {#each visibleOptions as opt, idx (opt.value)}
        <div
          role="option"
          id={`select-opt-${opt.value}`}
          aria-selected={opt.value === value}
          tabindex="-1"
          bind:this={optionRefs[idx]}
          onclick={() => {
            value = opt.value;
            onSelect?.(opt.value);
            open = false;
            buttonEl?.focus();
          }}
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              value = opt.value;
              onSelect?.(opt.value);
              open = false;
              buttonEl?.focus();
            }
          }}
          onmouseenter={() => (highlightedIndex = idx)}
          class="font-proto text-small flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-left transition-colors {opt.value ===
          value
            ? 'bg-teal/10 text-teal font-semibold'
            : highlightedIndex === idx
              ? 'bg-line/40 text-text-strong'
              : 'text-text-base hover:bg-line/40 hover:text-text-strong'}"
        >
          <div class="flex min-w-0 flex-col pr-2">
            <span class="truncate">{opt.label}</span>
            {#if opt.sublabel}
              <span class="text-text-muted text-smaller font-proto truncate">{opt.sublabel}</span>
            {/if}
          </div>
          {#if opt.value === value}
            <span class="text-teal text-smaller shrink-0 font-bold">✓</span>
          {/if}
        </div>
      {:else}
        <p class="text-text-muted text-small font-proto px-3 py-2 text-center">
          {i18n.t.selectNoOptions}
        </p>
      {/each}
    </div>
  {/if}
</div>
