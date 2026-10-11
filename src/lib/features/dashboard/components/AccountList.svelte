<script lang="ts">
  import type { AccountBalanceView, AccountType } from '$lib/core/ipc/bindings';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { ACCOUNT_TYPES, accountTypeLabel } from '$lib/core/format/account';
  import { i18n } from '$lib/core/i18n.svelte';
  import {
    SelectDropdown,
    EmptyState,
    Card,
    Pagination,
    Badge,
    AccountHoverCard,
  } from '$lib/components/ui';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import AccountTrendChart from './AccountTrendChart.svelte';

  let {
    searchQuery,
    searchScope,
    accounts = [],
    class: extraClass = '',
  }: {
    searchQuery: string;
    searchScope: 'ALL' | 'ACCOUNT' | 'RECENT';
    accounts?: AccountBalanceView[];
    class?: string;
  } = $props();

  let viewMode = $state<'table' | 'chart'>('table');

  function handleAccountClick(item: AccountBalanceView) {
    if (!item.account.placeholder) {
      goto(resolve('/app/accounts/[code]', { code: item.account.code }));
    } else {
      goto(resolve('/app/accounts'));
    }
  }

  let categoryFilter = $state<string>('ALL');
  let accPage = $state(1);
  const PAGE_SIZE = 10;

  const filterOptions = $derived([
    { value: 'ALL', label: i18n.t.filterAllLabel },
    ...ACCOUNT_TYPES.map((t: AccountType) => ({
      value: t,
      label: accountTypeLabel(t).toUpperCase(),
    })),
  ]);

  const visibleAccounts = $derived(accounts.filter((i) => !i.account.hidden));

  const filteredAccounts = $derived(
    visibleAccounts
      .filter((i) => categoryFilter === 'ALL' || i.account.account_type === categoryFilter)
      .filter((i) => {
        if (!searchQuery.trim() || searchScope === 'RECENT') return true;
        const q = searchQuery.toLowerCase();
        return i.account.code.toLowerCase().includes(q) || i.account.name.toLowerCase().includes(q);
      })
  );

  const totalAccPages = $derived(Math.max(1, Math.ceil(filteredAccounts.length / PAGE_SIZE)));

  $effect(() => {
    void searchQuery;
    void categoryFilter;
    void searchScope;
    accPage = 1;
  });

  const pagedAccounts = $derived(
    filteredAccounts.slice((accPage - 1) * PAGE_SIZE, accPage * PAGE_SIZE)
  );
</script>

<Card
  title={i18n.t.account}
  count={viewMode === 'table' ? filteredAccounts.length : undefined}
  class="flex h-full flex-col {extraClass}"
>
  {#snippet header()}
    <div class="flex min-w-0 flex-1 items-center justify-end gap-3">
      <div class="border-line flex shrink-0 items-center border">
        <button
          type="button"
          class="font-proto text-smaller px-2 py-0.5 tracking-wider uppercase transition-colors {viewMode ===
          'table'
            ? 'bg-bg-btn text-teal font-bold'
            : 'text-text-dim hover:text-text-base'}"
          onclick={() => (viewMode = 'table')}
        >
          {i18n.t.viewModeTable}
        </button>
        <div class="bg-line h-3.5 w-px"></div>
        <button
          type="button"
          class="font-proto text-smaller px-2 py-0.5 tracking-wider uppercase transition-colors {viewMode ===
          'chart'
            ? 'bg-bg-btn text-teal font-bold'
            : 'text-text-dim hover:text-text-base'}"
          onclick={() => (viewMode = 'chart')}
        >
          {i18n.t.viewModeVisual}
        </button>
      </div>

      {#if viewMode === 'table'}
        <SelectDropdown
          size="sm"
          options={filterOptions}
          bind:value={categoryFilter}
          class="w-32 shrink-0"
        />
        <span
          class="text-text-dim font-proto text-smaller ml-auto shrink-0 tracking-widest uppercase tabular-nums"
        >
          {i18n.t.showingOf
            .replace('{shown}', String(pagedAccounts.length))
            .replace('{total}', String(filteredAccounts.length))}
        </span>
      {/if}
    </div>
  {/snippet}

  {#if viewMode === 'table'}
    <div class="mt-2 min-h-0 flex-1 overflow-y-auto pr-3">
      {#if pagedAccounts.length === 0}
        <EmptyState
          title={i18n.t.noMatchingAccounts}
          actionLabel={i18n.t.newAccount}
          actionHref="/app/accounts"
        />
      {:else}
        <div class="flex flex-col gap-1.5 pr-1">
          {#each pagedAccounts as item (item.account.id)}
            <AccountHoverCard
              account={item.account}
              directBalance={item.direct_balance}
              rollupBalance={item.recursive_balance}
              currency={item.account.currency}
              class="w-full"
            >
              <div
                role="button"
                tabindex="0"
                onclick={() => handleAccountClick(item)}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    handleAccountClick(item);
                  }
                }}
                class="group hover:bg-bg-btn flex cursor-pointer items-center gap-2 px-1.5 py-0.5 transition-colors first:pt-0"
              >
                <span class="font-proto text-text-muted text-smaller w-8 shrink-0"
                  >{item.account.code}</span
                >
                <span class="font-proto text-text-base text-smaller shrink-0"
                  >{item.account.name}</span
                >
                {#if item.account.placeholder}
                  <Badge size="s" tone="neutral" class="shrink-0">{i18n.t.badgePh}</Badge>
                {/if}

                <div
                  class="border-line/80 group-hover:border-line mb-0.5 flex-1 border-b border-dotted transition-colors"
                ></div>

                <span
                  class="font-proto text-text-strong text-smaller shrink-0 font-medium tabular-nums"
                >
                  {formatMinorGrouping(Math.abs(item.recursive_balance), item.account.currency)}
                </span>
                <span class="font-proto text-text-dim text-smaller w-7 shrink-0 text-right"
                  >{item.account.currency}</span
                >
              </div>
            </AccountHoverCard>
          {/each}
        </div>
      {/if}
    </div>

    <div class="border-line/40 mt-auto flex shrink-0 items-center border-t pt-1.5">
      <Pagination
        bind:currentPage={accPage}
        totalPages={totalAccPages}
        layout="spread"
        class="flex-1"
      />
    </div>
  {:else}
    <div class="mt-2 min-h-0 flex-1">
      <AccountTrendChart class="h-full w-full" />
    </div>
  {/if}
</Card>
