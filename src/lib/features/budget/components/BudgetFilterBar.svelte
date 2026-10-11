<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { SearchBar, Button } from '$lib/components/ui';

  export type BudgetFilterStatus = 'ALL' | 'funded' | 'overspent' | 'unassigned';

  let {
    searchQuery = $bindable(''),
    filterStatus = $bindable<BudgetFilterStatus>('ALL'),
    statusCounts,
    filteredCount,
    totalCount,
    onReset,
  }: {
    searchQuery?: string;
    filterStatus?: BudgetFilterStatus;
    statusCounts: Record<BudgetFilterStatus, number>;
    filteredCount: number;
    totalCount: number;
    onReset: () => void;
  } = $props();

  const isFilterActive = $derived(searchQuery.trim() !== '' || filterStatus !== 'ALL');
</script>

<div class="border-line bg-bg-card flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2">
  <SearchBar
    bind:value={searchQuery}
    placeholder={i18n.t.budgetSearchPlaceholder}
    class="max-w-none min-w-36 flex-1"
  />

  <div class="border-line flex shrink-0 items-center border">
    <button
      type="button"
      class="font-proto text-smaller px-2.5 py-1.5 transition-colors {filterStatus === 'ALL'
        ? 'bg-bg-btn text-text-strong font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => (filterStatus = 'ALL')}
    >
      {i18n.t.filterAllLabel} ({statusCounts.ALL})
    </button>
    <button
      type="button"
      class="border-line font-proto text-smaller border-l px-2.5 py-1.5 transition-colors {filterStatus ===
      'funded'
        ? 'bg-bg-row-active text-teal font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => (filterStatus = 'funded')}
    >
      {i18n.t.filterFunded} ({statusCounts.funded})
    </button>
    <button
      type="button"
      class="border-line font-proto text-smaller border-l px-2.5 py-1.5 transition-colors {filterStatus ===
      'overspent'
        ? 'bg-danger-bg text-danger font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => (filterStatus = 'overspent')}
    >
      {i18n.t.filterOverspent} ({statusCounts.overspent})
    </button>
    <button
      type="button"
      class="border-line font-proto text-smaller border-l px-2.5 py-1.5 transition-colors {filterStatus ===
      'unassigned'
        ? 'bg-bg-btn text-text-dim font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => (filterStatus = 'unassigned')}
    >
      {i18n.t.filterUnassigned} ({statusCounts.unassigned})
    </button>
  </div>

  <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
    {filteredCount}/{totalCount}
    {i18n.t.envelopesLabel}
  </span>

  {#if isFilterActive}
    <Button
      variant="ghost"
      size="sm"
      onclick={() => {
        searchQuery = '';
        filterStatus = 'ALL';
        onReset();
      }}
    >
      {i18n.t.reset}
    </Button>
  {/if}
</div>
