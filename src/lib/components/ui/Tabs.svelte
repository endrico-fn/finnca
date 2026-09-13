<script lang="ts">
  let {
    tabs,
    active,
    onSelect,
    variant = 'pill',
    fullWidth = false,
    class: className = '',
  }: {
    tabs: Array<{ id: string; label: string; dot?: string; count?: number }>;
    active: string;
    onSelect: (id: string) => void;
    variant?: 'pill' | 'outline';
    fullWidth?: boolean;
    class?: string;
  } = $props();

  const rootCls = $derived(
    `font-proto text-smaller items-center gap-1 uppercase ${fullWidth ? 'flex w-full' : 'inline-flex'} ${className}`
  );
  const btnFlex = $derived(fullWidth ? 'flex-1' : 'shrink-0');
</script>

<div class={rootCls}>
  {#each tabs as tab (tab.id || tab)}
    {#if variant === 'pill'}
      <button
        type="button"
        onclick={() => onSelect(tab.id)}
        aria-pressed={active === tab.id}
        class="font-proto text-smaller inline-flex h-6 cursor-pointer items-center justify-center gap-1.5 border px-2 whitespace-nowrap transition-colors select-none {btnFlex} {active ===
        tab.id
          ? 'bg-line/90 border-line text-text-strong font-semibold'
          : 'text-text-muted hover:text-text-base hover:bg-bg-card hover:border-line/40 border-transparent bg-transparent'}"
      >
        {#if tab.dot}
          <span class="size-1.5 shrink-0" style="background:{tab.dot}"></span>
        {/if}
        {tab.label}
        {#if tab.count != null}
          <span class="tabular-nums opacity-60">({tab.count})</span>
        {/if}
      </button>
    {:else}
      <button
        type="button"
        onclick={() => onSelect(tab.id)}
        aria-pressed={active === tab.id}
        class="font-proto text-smaller inline-flex h-6 cursor-pointer items-center justify-center gap-1.5 border px-2 whitespace-nowrap transition-colors select-none {btnFlex} {active ===
        tab.id
          ? 'border-teal/60 bg-bg-row-active text-text-strong font-semibold'
          : 'border-line text-text-muted hover:border-text-base hover:text-text-base hover:bg-bg-card/40'}"
      >
        {#if tab.dot}
          <span class="size-1.5 shrink-0" style="background:{tab.dot}"></span>
        {/if}
        {tab.label}
        {#if tab.count != null}
          <span class="tabular-nums opacity-60">({tab.count})</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>
