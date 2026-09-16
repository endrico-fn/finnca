<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { i18n } from '$lib/core/i18n.svelte';
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
    leafAccounts = [],
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
    leafAccounts?: Account[];
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

<div class="border-line flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2.5">
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
    panelClass="w-72"
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
        <FilterOption
          label={`${acc.code} ${acc.name}`}
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
  <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
    {filteredCount}/{totalCount}
    {i18n.t.txUnit}
  </span>
  {#if q || from || to || accFilter}
    <Button
      variant="ghost"
      size="sm"
      onclick={() => {
        q = '';
        from = '';
        to = '';
        accFilter = '';
        onFilterChange?.();
      }}
    >
      {i18n.t.reset}
    </Button>
  {/if}

  <div class="border-line ml-auto hidden items-center gap-1.5 border-l pl-2 lg:flex shrink-0">
    <span
      class="border-line bg-bg-btn text-text-muted font-proto border px-1.5 py-0.5 text-[10px] uppercase tracking-wider"
      title="Navigate rows up/down"
    >
      J/K: NAV
    </span>
    <span
      class="border-line bg-bg-btn text-text-muted font-proto border px-1.5 py-0.5 text-[10px] uppercase tracking-wider"
      title="Expand or collapse splits"
    >
      ENTER: EXPAND
    </span>
    <span
      class="border-line bg-bg-btn text-text-muted font-proto border px-1.5 py-0.5 text-[10px] uppercase tracking-wider"
      title="Edit selected transaction"
    >
      E: EDIT
    </span>
  </div>
</div>
