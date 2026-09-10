<script lang="ts">
  let {
    tabs,
    active,
    onSelect,
    variant = 'pill',
    size = 'sm',
    class: className = '',
  }: {
    tabs: Array<{ id: string; label: string }>;
    active: string;
    onSelect: (id: string) => void;
    variant?: 'pill' | 'outline' | 'segmented';
    size?: 'sm' | 'md';
    class?: string;
  } = $props();

  const heightCls = $derived(size === 'md' ? 'h-7 px-3 text-[11px]' : 'h-6 px-2 text-[10px]');
</script>

<div
  class="inline-flex items-center {variant === 'segmented'
    ? 'border-line bg-bg-app gap-0.5 border p-0.5'
    : 'gap-1'} font-proto text-[10px] uppercase {className}"
>
  {#each tabs as tab (tab.id || tab)}
    {#if variant === 'pill'}
      <button
        type="button"
        onclick={() => onSelect(tab.id)}
        class="{heightCls} font-proto inline-flex shrink-0 cursor-pointer items-center justify-center border whitespace-nowrap transition-colors select-none {active ===
        tab.id
          ? 'bg-line/90 border-line text-text-strong font-semibold'
          : 'text-text-muted hover:text-text-base hover:bg-bg-card hover:border-line/40 border-transparent bg-transparent'}"
      >
        {tab.label}
      </button>
    {:else if variant === 'outline'}
      <button
        type="button"
        onclick={() => onSelect(tab.id)}
        class="{heightCls} font-proto inline-flex shrink-0 cursor-pointer items-center justify-center border whitespace-nowrap transition-colors select-none {active ===
        tab.id
          ? 'border-teal/60 bg-bg-row-active text-income font-semibold'
          : 'border-line text-text-muted hover:border-text-base hover:text-text-base hover:bg-bg-card/40'}"
      >
        {tab.label}
      </button>
    {:else if variant === 'segmented'}
      <button
        type="button"
        onclick={() => onSelect(tab.id)}
        class="{heightCls} font-proto inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap transition-colors select-none {active ===
        tab.id
          ? 'bg-line text-text-white font-semibold'
          : 'text-text-dim hover:text-text-base hover:bg-bg-card bg-transparent'}"
      >
        {tab.label}
      </button>
    {/if}
  {/each}
</div>
