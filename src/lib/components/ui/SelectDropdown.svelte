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
  let openUpward = $state(false);
  let query = $state('');
  let containerEl: HTMLElement | null = $state(null);

  const selectedOption = $derived(options.find((o) => o.value === value));

  const visibleOptions = $derived(
    !searchable || !query.trim()
      ? options
      : options.filter((o) =>
          `${o.label} ${o.sublabel ?? ''}`.toLowerCase().includes(query.toLowerCase().trim())
        )
  );

  function toggle() {
    if (disabled) return;
    if (!open && containerEl) {
      const rect = containerEl.getBoundingClientRect();
      const spaceBelow = window.innerHeight - rect.bottom;
      const spaceAbove = rect.top;
      if (placement === 'top') {
        openUpward = true;
      } else if (placement === 'bottom') {
        openUpward = false;
      } else {
        openUpward = spaceBelow < 220 && spaceAbove > spaceBelow;
      }
    }
    query = '';
    open = !open;
  }

  function handleClickOutside(e: MouseEvent) {
    if (containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
    return () => window.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="relative inline-block {className}" bind:this={containerEl}>
  <button
    type="button"
    {disabled}
    onclick={toggle}
    aria-haspopup="listbox"
    aria-expanded={open}
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
      class="border-line bg-bg-card text-small font-proto absolute {openUpward
        ? 'bottom-full mb-1'
        : 'top-full mt-1'} left-0 z-50 max-h-60 w-max max-w-xs min-w-full overflow-y-auto border py-1 select-none {menuClass}"
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
      {#each visibleOptions as opt (opt.value)}
        <button
          type="button"
          onclick={() => {
            value = opt.value;
            onSelect?.(opt.value);
            open = false;
          }}
          class="font-proto text-small flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-left transition-colors {opt.value ===
          value
            ? 'bg-teal/10 text-teal font-semibold'
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
        </button>
      {:else}
        <p class="text-text-muted text-small font-proto px-3 py-2 text-center">
          {i18n.t.selectNoOptions}
        </p>
      {/each}
    </div>
  {/if}
</div>
