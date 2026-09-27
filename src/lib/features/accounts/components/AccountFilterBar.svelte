<script lang="ts">
  import type { AccountType } from '$lib/core/ipc/bindings';
  import { ACCOUNT_TYPES, accountTypeLabel, ACCOUNT_TYPE_BG } from '../state/accounts.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { SearchBar, FilterMenu, FilterSection, FilterOption, Button } from '$lib/components/ui';

  let {
    searchQuery = $bindable(''),
    selectedType = $bindable<AccountType | 'ALL'>('ALL'),
    placeholderOnly = $bindable(false),
    showHidden = $bindable(false),
    typeCounts,
    phCount,
    filteredCount,
    totalCount,
    onReset,
  }: {
    searchQuery?: string;
    selectedType?: AccountType | 'ALL';
    placeholderOnly?: boolean;
    showHidden?: boolean;
    typeCounts: Record<AccountType, number> & { ALL: number };
    phCount: number;
    filteredCount: number;
    totalCount: number;
    onReset: () => void;
  } = $props();

  let filterOpen = $state(false);

  const isFilterActive = $derived(selectedType !== 'ALL' || showHidden || placeholderOnly);
  const activeFilterCount = $derived(
    (selectedType !== 'ALL' ? 1 : 0) + (showHidden ? 1 : 0) + (placeholderOnly ? 1 : 0)
  );
  const filterButtonLabel = $derived(
    selectedType === 'ALL' ? i18n.t.filterBtn : accountTypeLabel(selectedType).toUpperCase()
  );
</script>

<div class="border-line flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2">
  <SearchBar
    bind:value={searchQuery}
    placeholder={i18n.t.searchAccountPlaceholder}
    class="max-w-none min-w-40 flex-1"
  />

  <FilterMenu
    bind:open={filterOpen}
    label={filterButtonLabel}
    active={isFilterActive}
    count={activeFilterCount}
    {onReset}
    resetLabel={i18n.t.reset}
    resetDisabled={!isFilterActive}
  >
    <FilterSection title={i18n.t.filterTypeTitle}>
      <FilterOption
        label={i18n.t.filterAllLabel}
        count={typeCounts.ALL}
        selected={selectedType === 'ALL'}
        check={false}
        onclick={() => {
          selectedType = 'ALL';
          filterOpen = false;
        }}
      />
      {#each ACCOUNT_TYPES as typeKey (typeKey)}
        <FilterOption
          label={accountTypeLabel(typeKey).toUpperCase()}
          dot={ACCOUNT_TYPE_BG[typeKey]}
          count={typeCounts[typeKey]}
          selected={selectedType === typeKey}
          onclick={() => {
            selectedType = typeKey;
            filterOpen = false;
          }}
        />
      {/each}
    </FilterSection>

    <div class="border-line/60 border-t">
      <FilterSection title={i18n.t.filterAttrTitle} layout="list">
        <FilterOption
          label={i18n.t.filterPhOnly}
          count={phCount}
          selected={placeholderOnly}
          onclick={() => (placeholderOnly = !placeholderOnly)}
        />
        <FilterOption
          label={showHidden ? i18n.t.toggleHiddenShow : i18n.t.toggleHiddenHide}
          tone="warning"
          selected={showHidden}
          title={i18n.t.toggleHiddenTitle}
          onclick={() => (showHidden = !showHidden)}
        />
      </FilterSection>
    </div>
  </FilterMenu>
  <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
    {filteredCount}/{totalCount}
  </span>
  {#if searchQuery || isFilterActive}
    <Button
      variant="ghost"
      size="sm"
      onclick={() => {
        searchQuery = '';
        onReset();
      }}
    >
      {i18n.t.reset}
    </Button>
  {/if}
</div>
