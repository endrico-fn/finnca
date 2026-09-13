<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { accountBalanceMinor, formatMoney, getAccountCleanPath } from '$lib/accounting/finance';
  import { ACCOUNT_TYPES, accountTypeLabel, type AccountType } from '$lib/accounting/types';
  import { i18n } from '$lib/i18n.svelte';
  import { Tabs, EmptyState, Card, Pagination } from '$lib/components/ui';

  let { searchQuery, searchScope, childrenMap } = $props<{
    searchQuery: string;
    searchScope: 'ALL' | 'ACCOUNT' | 'RECENT';
    childrenMap: Map<string, string[]>;
  }>();

  let categoryFilter = $state<string>('ALL');
  let accPage = $state(1);
  const PAGE_SIZE = 10;

  const leafAccounts = $derived(ledger.accounts.filter((a) => !a.placeholder && !a.hidden));

  const filteredAccounts = $derived(
    leafAccounts
      .filter((a) => categoryFilter === 'ALL' || a.type === categoryFilter)
      .filter((a) => {
        if (!searchQuery.trim() || searchScope === 'RECENT') return true;
        const q = searchQuery.toLowerCase();
        return (
          a.code.toLowerCase().includes(q) ||
          a.name.toLowerCase().includes(q) ||
          getAccountCleanPath(a.id, ledger.accountsById).toLowerCase().includes(q)
        );
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

  function getCleanPath(accId: string): string {
    return getAccountCleanPath(accId, ledger.accountsById);
  }
</script>

<Card title="{i18n.t.account} {filteredAccounts.length}" class="col-span-8 h-full">
  {#snippet header()}
    <div class="flex min-w-0 flex-1 items-center justify-end gap-3">
      <Tabs
        variant="outline"
        tabs={[
          { id: 'ALL', label: i18n.t.filterAllLabel },
          ...ACCOUNT_TYPES.map((t: AccountType) => ({
            id: t,
            label: accountTypeLabel(t).toUpperCase(),
          })),
        ]}
        active={categoryFilter}
        onSelect={(id) => {
          categoryFilter = id;
        }}
        class="max-w-full overflow-x-auto"
      />
      <span
        class="text-text-dim font-proto text-smaller ml-auto shrink-0 tracking-widest uppercase tabular-nums"
      >
        {i18n.t.showingOf
          .replace('{shown}', String(pagedAccounts.length))
          .replace('{total}', String(filteredAccounts.length))}
      </span>
    </div>
  {/snippet}

  <div class="mt-2 min-h-0 flex-1 overflow-y-auto pr-1">
    {#if pagedAccounts.length === 0}
      <EmptyState
        title={i18n.t.noMatchingAccounts}
        actionLabel={i18n.t.newAccount}
        actionHref="/app/accounts"
      />
    {:else}
      <div class="flex flex-col gap-1.5">
        {#each pagedAccounts as acc (acc.id)}
          {@const bal = ledger.data ? accountBalanceMinor(acc.id, ledger.data, childrenMap) : 0}
          <div
            class="group hover:bg-bg-btn -mx-1 flex cursor-default items-end gap-2 px-1 transition-colors"
          >
            <span class="font-proto text-text-muted text-smaller w-8 shrink-0">{acc.code}</span>
            <span class="font-proto text-text-base text-smaller shrink-0"
              >{getCleanPath(acc.id)}</span
            >

            <div
              class="border-line/80 group-hover:border-line mb-0.5 flex-1 border-b border-dotted transition-colors"
            ></div>

            <span
              class="font-proto text-text-strong text-smaller shrink-0 font-medium tabular-nums"
            >
              {formatMoney(Math.abs(bal), acc.currency)}
            </span>
            <span class="font-proto text-text-dim text-smaller mb-px w-6 shrink-0 text-right"
              >{acc.currency}</span
            >
          </div>
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
</Card>
