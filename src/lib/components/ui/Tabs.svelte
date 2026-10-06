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
    variant?: 'pill' | 'outline' | 'segmented';
    fullWidth?: boolean;
    class?: string;
  } = $props();

  const rootCls = $derived(
    `font-proto text-smaller items-center gap-1 uppercase ${fullWidth ? 'flex w-full' : 'inline-flex'} ${className}`
  );
  const btnFlex = $derived(fullWidth ? 'flex-1' : 'shrink-0');

  let tabButtons: HTMLButtonElement[] = [];

  function handleKeydown(e: KeyboardEvent, index: number) {
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      const nextIdx = (index + 1) % tabs.length;
      const nextTab = tabs[nextIdx];
      if (nextTab) {
        onSelect(nextTab.id);
        tabButtons[nextIdx]?.focus();
      }
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      const prevIdx = (index - 1 + tabs.length) % tabs.length;
      const prevTab = tabs[prevIdx];
      if (prevTab) {
        onSelect(prevTab.id);
        tabButtons[prevIdx]?.focus();
      }
    } else if (e.key === 'Home') {
      e.preventDefault();
      const firstTab = tabs[0];
      if (firstTab) {
        onSelect(firstTab.id);
        tabButtons[0]?.focus();
      }
    } else if (e.key === 'End') {
      e.preventDefault();
      const lastTab = tabs[tabs.length - 1];
      if (lastTab) {
        onSelect(lastTab.id);
        tabButtons[tabs.length - 1]?.focus();
      }
    }
  }
</script>

<div class={rootCls} role="tablist" aria-orientation="horizontal">
  {#each tabs as tab, index (tab.id || tab)}
    {#if variant === 'segmented'}
      <button
        type="button"
        role="tab"
        id={`tab-${tab.id}`}
        aria-selected={active === tab.id}
        tabindex={active === tab.id ? 0 : -1}
        bind:this={tabButtons[index]}
        onclick={() => onSelect(tab.id)}
        onkeydown={(e) => handleKeydown(e, index)}
        class="font-proto text-smaller px-2.5 py-1 tracking-wider whitespace-nowrap uppercase transition-colors select-none {btnFlex} {active ===
        tab.id
          ? 'border-teal text-teal border-b-2 font-bold'
          : 'text-text-muted hover:text-text-base'}"
      >
        {tab.label}
        {#if tab.count != null}
          <span class="tabular-nums opacity-60">({tab.count})</span>
        {/if}
      </button>
    {:else if variant === 'pill'}
      <button
        type="button"
        role="tab"
        id={`tab-${tab.id}`}
        aria-selected={active === tab.id}
        tabindex={active === tab.id ? 0 : -1}
        bind:this={tabButtons[index]}
        onclick={() => onSelect(tab.id)}
        onkeydown={(e) => handleKeydown(e, index)}
        class="font-proto text-smaller inline-flex h-6 cursor-pointer items-center justify-center gap-1.5 border px-2 whitespace-nowrap transition-colors select-none {btnFlex} {active ===
        tab.id
          ? 'bg-bg-row-active border-line text-text-strong font-semibold'
          : 'text-text-muted hover:text-text-base hover:bg-bg-card hover:border-line/40 border-transparent bg-transparent'}"
      >
        {#if tab.dot}
          <span
            class="size-1.5 shrink-0 {tab.dot.startsWith('bg-') ? tab.dot : ''}"
            style={tab.dot.startsWith('bg-') ? undefined : `background:${tab.dot}`}
          ></span>
        {/if}
        {tab.label}
        {#if tab.count != null}
          <span class="tabular-nums opacity-60">({tab.count})</span>
        {/if}
      </button>
    {:else}
      <button
        type="button"
        role="tab"
        id={`tab-${tab.id}`}
        aria-selected={active === tab.id}
        tabindex={active === tab.id ? 0 : -1}
        bind:this={tabButtons[index]}
        onclick={() => onSelect(tab.id)}
        onkeydown={(e) => handleKeydown(e, index)}
        class="font-proto text-smaller inline-flex h-6 cursor-pointer items-center justify-center gap-1.5 border px-2 whitespace-nowrap transition-colors select-none {btnFlex} {active ===
        tab.id
          ? 'border-teal/60 bg-bg-row-active text-text-strong font-semibold'
          : 'border-line text-text-muted hover:border-text-base hover:text-text-base hover:bg-bg-card/40'}"
      >
        {#if tab.dot}
          <span
            class="size-1.5 shrink-0 {tab.dot.startsWith('bg-') ? tab.dot : ''}"
            style={tab.dot.startsWith('bg-') ? undefined : `background:${tab.dot}`}
          ></span>
        {/if}
        {tab.label}
        {#if tab.count != null}
          <span class="tabular-nums opacity-60">({tab.count})</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>
