<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';

  export interface DropdownOption {
    value: string;
    label: string;
    sublabel?: string;
  }

  let {
    options,
    value = $bindable(),
    placeholder = 'Select...',
    onSelect,
    size = 'md',
    class: className = '',
    menuClass = '',
    disabled = false,
  }: {
    options: DropdownOption[];
    value: string;
    placeholder?: string;
    onSelect?: (val: string) => void;
    size?: 'sm' | 'md';
    class?: string;
    menuClass?: string;
    disabled?: boolean;
  } = $props();

  let open = $state(false);
  let containerEl: HTMLElement | null = $state(null);

  const selectedOption = $derived(options.find((o) => o.value === value));

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
    onclick={() => !disabled && (open = !open)}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={placeholder}
    class="sharp-btn btn-ghost {size === 'sm'
      ? 'h-7 px-2 text-[10.5px]'
      : 'h-8 px-2.5 text-[11px]'} font-proto border-line bg-bg-app hover:border-text-dim text-text-strong inline-flex w-full cursor-pointer items-center justify-between gap-2 border transition-colors select-none disabled:cursor-not-allowed disabled:opacity-40"
  >
    <span class="truncate">{selectedOption?.label ?? placeholder}</span>
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
      class="border-line bg-bg-card absolute top-full left-0 z-50 mt-1 max-h-60 w-max max-w-xs min-w-full overflow-y-auto border py-1 font-mono text-[11px] select-none {menuClass}"
    >
      {#each options as opt (opt.value)}
        <button
          type="button"
          onclick={() => {
            value = opt.value;
            onSelect?.(opt.value);
            open = false;
          }}
          class="font-proto flex w-full cursor-pointer items-center justify-between px-3 py-1.5 text-left text-[11px] transition-colors {opt.value ===
          value
            ? 'bg-teal/10 text-teal font-semibold'
            : 'text-text-base hover:bg-line/40 hover:text-text-strong'}"
        >
          <div class="flex min-w-0 flex-col pr-2">
            <span class="truncate">{opt.label}</span>
            {#if opt.sublabel}
              <span class="text-text-muted truncate font-mono text-[9px]">{opt.sublabel}</span>
            {/if}
          </div>
          {#if opt.value === value}
            <span class="text-teal shrink-0 text-[10px] font-bold">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
