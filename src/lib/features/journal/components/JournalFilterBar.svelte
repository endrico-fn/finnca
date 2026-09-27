<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { i18n } from '$lib/core/i18n.svelte';
  import { getAccountBreadcrumb } from '$lib/core/format/account';
  import {
    SearchBar,
    Button,
    FilterMenu,
    FilterSection,
    FilterOption,
    DateRangeDropdown,
  } from '$lib/components/ui';

  let {
    q = $bindable(''),
    from = $bindable(''),
    to = $bindable(''),
    accFilter = $bindable(''),
    statusFilter = $bindable<'all' | 'due' | 'overdue'>('all'),
    leafAccounts = [],
    accountsById,
    accTxCounts = {},
    baseTxsCount = 0,
    filteredCount = 0,
    totalCount = 0,
    selectedAcc,
    onFilterChange,
  }: {
    q?: string;
    from?: string;
    to?: string;
    accFilter?: string;
    statusFilter?: 'all' | 'due' | 'overdue';
    leafAccounts?: Account[];
    accountsById?: Map<string, Account>;
    accTxCounts?: Record<string, number>;
    baseTxsCount?: number;
    filteredCount?: number;
    totalCount?: number;
    selectedAcc?: Account;
    onFilterChange?: () => void;
  } = $props();

  let accFilterOpen = $state(false);

  const accFilterLabel = $derived(
    selectedAcc ? `${selectedAcc.code} ${selectedAcc.name}` : i18n.t.filterBtn
  );
</script>

<div class="border-line bg-bg-card flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2">
  <SearchBar
    bind:value={q}
    oninput={onFilterChange}
    onclear={onFilterChange}
    placeholder={i18n.t.searchPlaceholder}
    class="max-w-none min-w-36 flex-1"
  />
  <FilterMenu
    bind:open={accFilterOpen}
    label={accFilterLabel}
    active={accFilter !== ''}
    count={accFilter !== '' ? 1 : 0}
    onReset={() => {
      accFilter = '';
      onFilterChange?.();
    }}
    resetLabel={i18n.t.reset}
    resetDisabled={accFilter === ''}
    panelClass="w-84 max-w-sm"
  >
    <FilterSection title={i18n.t.filterAccountTitle} layout="list">
      <FilterOption
        label={`${i18n.t.allTxs} — ${i18n.t.allAccountsFilter}`}
        count={baseTxsCount}
        selected={accFilter === ''}
        check={false}
        onclick={() => {
          accFilter = '';
          onFilterChange?.();
          accFilterOpen = false;
        }}
      />
      {#each leafAccounts as acc (acc.id)}
        {@const bc = accountsById ? getAccountBreadcrumb(acc, accountsById) : []}
        {@const parentPath = bc.length > 1 ? bc.slice(0, -1).join(' > ') : ''}
        <FilterOption
          label={parentPath ? `${acc.code} ${acc.name} — ${parentPath}` : `${acc.code} ${acc.name}`}
          count={accTxCounts[acc.id] ?? 0}
          selected={accFilter === acc.id}
          onclick={() => {
            accFilter = acc.id;
            onFilterChange?.();
            accFilterOpen = false;
          }}
        />
      {/each}
    </FilterSection>
  </FilterMenu>
  <DateRangeDropdown bind:from bind:to onChange={onFilterChange} size="md" />

  <div class="border-line flex shrink-0 items-center border">
    <button
      type="button"
      class="font-proto text-smaller px-2.5 py-1.5 transition-colors {statusFilter === 'all'
        ? 'bg-bg-btn text-text-strong font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => {
        statusFilter = 'all';
        onFilterChange?.();
      }}
    >
      {i18n.t.filterAllEntries}
    </button>
    <button
      type="button"
      class="border-line font-proto text-smaller border-l px-2.5 py-1.5 transition-colors {statusFilter ===
      'due'
        ? 'bg-warning-bg text-warning font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => {
        statusFilter = 'due';
        onFilterChange?.();
      }}
    >
      {i18n.t.filterDueInvoices}
    </button>
    <button
      type="button"
      class="border-line font-proto text-smaller border-l px-2.5 py-1.5 transition-colors {statusFilter ===
      'overdue'
        ? 'bg-danger-bg text-danger font-bold'
        : 'text-text-muted hover:text-text-base'}"
      onclick={() => {
        statusFilter = 'overdue';
        onFilterChange?.();
      }}
    >
      {i18n.t.filterOverdue}
    </button>
  </div>

  <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
    {filteredCount}/{totalCount}
    {i18n.t.txUnit}
  </span>
  {#if q || from || to || accFilter || statusFilter !== 'all'}
    <Button
      variant="ghost"
      size="sm"
      onclick={() => {
        q = '';
        from = '';
        to = '';
        accFilter = '';
        statusFilter = 'all';
        onFilterChange?.();
      }}
    >
      {i18n.t.reset}
    </Button>
  {/if}

  <div class="border-line ml-auto hidden shrink-0 items-center gap-1.5 border-l pl-2 lg:flex">
    <span
      class="border-line bg-bg-btn text-text-muted font-proto text-smaller border px-1.5 py-0.5 tracking-wider uppercase"
      title={i18n.t.shortcutNavRows}
    >
      {i18n.t.shortcutNavBadge}
    </span>
    <span
      class="border-line bg-bg-btn text-text-muted font-proto text-smaller border px-1.5 py-0.5 tracking-wider uppercase"
      title={i18n.t.shortcutExpandSplits}
    >
      {i18n.t.shortcutExpandBadge}
    </span>
    <span
      class="border-line bg-bg-btn text-text-muted font-proto text-smaller border px-1.5 py-0.5 tracking-wider uppercase"
      title={i18n.t.shortcutEditTx}
    >
      {i18n.t.shortcutEditBadge}
    </span>
  </div>
</div>
