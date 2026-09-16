<script lang="ts">
  import type { ReconcileState } from '$lib/core/types';
  import { i18n } from '$lib/core/i18n.svelte';
  import {
    SearchBar,
    FilterMenu,
    FilterSection,
    FilterOption,
    DateRangeDropdown,
    Button,
  } from '$lib/components/ui';

  let {
    q = $bindable(''),
    from = $bindable(''),
    to = $bindable(''),
    reconcileFilter = $bindable<'ALL' | ReconcileState>('ALL'),
    statusCounts,
    filteredCount,
    totalCount,
    onReset,
  }: {
    q?: string;
    from?: string;
    to?: string;
    reconcileFilter?: 'ALL' | ReconcileState;
    statusCounts: Record<'ALL' | ReconcileState, number>;
    filteredCount: number;
    totalCount: number;
    onReset: () => void;
  } = $props();

  let statusFilterOpen = $state(false);

  const statusOptions = $derived([
    { id: 'ALL' as const, label: i18n.t.filterAllR },
    { id: 'n' as const, label: i18n.t.filterNew },
    { id: 'c' as const, label: i18n.t.filterCleared },
    { id: 'y' as const, label: i18n.t.filterReconciled },
  ]);

  const statusFilterLabel = $derived(
    statusOptions.find((o) => o.id === reconcileFilter)?.label ?? i18n.t.filterBtn
  );
</script>

<div class="flex shrink-0 flex-wrap items-center gap-2">
  <SearchBar
    bind:value={q}
    placeholder={i18n.t.searchLedgerPlaceholder}
    class="max-w-none min-w-40 flex-1"
  />
  <FilterMenu
    bind:open={statusFilterOpen}
    label={statusFilterLabel}
    active={reconcileFilter !== 'ALL'}
    count={reconcileFilter !== 'ALL' ? 1 : 0}
    onReset={() => {
      reconcileFilter = 'ALL';
    }}
    resetLabel={i18n.t.reset}
    resetDisabled={reconcileFilter === 'ALL'}
  >
    <FilterSection title={i18n.t.filterStatusTitle} layout="list">
      {#each statusOptions as opt (opt.id)}
        <FilterOption
          label={opt.label}
          count={statusCounts[opt.id]}
          selected={reconcileFilter === opt.id}
          check={opt.id !== 'ALL'}
          onclick={() => {
            reconcileFilter = opt.id;
            statusFilterOpen = false;
          }}
        />
      {/each}
    </FilterSection>
  </FilterMenu>
  <DateRangeDropdown bind:from bind:to size="md" />
  <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
    {filteredCount}/{totalCount}
  </span>
  {#if q || from || to || reconcileFilter !== 'ALL'}
    <div class="text-smaller ml-auto flex items-center gap-1">
      <Button variant="ghost" size="sm" onclick={onReset}>{i18n.t.reset}</Button>
    </div>
  {/if}
</div>
