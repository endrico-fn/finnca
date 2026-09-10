<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { accountBalanceMinor, fromMinor, formatIDR } from '$lib/accounting/finance';
  import { getAccountCleanPath } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import { Tabs, EmptyState, Icon, Button, Card, Pagination } from '$lib/components/ui';

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
    searchQuery;
    categoryFilter;
    searchScope;
    accPage = 1;
  });

  const pagedAccounts = $derived(
    filteredAccounts.slice((accPage - 1) * PAGE_SIZE, accPage * PAGE_SIZE)
  );

  const fmt = (n: number) => n.toLocaleString('en-US');
  function getCleanPath(accId: string): string {
    return getAccountCleanPath(accId, ledger.accountsById);
  }
</script>

<Card title="{i18n.t.account} {filteredAccounts.length}" class="col-span-8 h-full">
  {#snippet header()}
    <div class="flex w-full items-center gap-4">
      <Tabs
        tabs={[
          { id: 'ALL', label: 'ALL' },
          { id: 'ASSET', label: 'ASSETS' },
          { id: 'LIABILITY', label: 'LIAB' },
          { id: 'EQUITY', label: 'EQUITY' },
          { id: 'INCOME', label: 'INCOME' },
          { id: 'EXPENSE', label: 'EXP' },
        ]}
        active={categoryFilter}
        onSelect={(id) => {
          categoryFilter = id;
        }}
      />
      <span
        class="text-text-dim ml-auto text-[9px] tracking-widest uppercase tabular-nums font-proto"
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
            class="group hover:bg-bg-row-hover -mx-1 flex cursor-default items-end gap-2 px-1 transition-colors"
          >
            <span class="font-proto text-text-muted w-8 shrink-0 text-[11px]">{acc.code}</span>
            <span class="font-proto text-text-base shrink-0 text-[11px]"
              >{getCleanPath(acc.id)}</span
            >

            <div
              class="border-line/80 group-hover:border-line mb-[3px] flex-1 border-b border-dotted transition-colors"
            ></div>

            <span
              class="font-proto text-text-strong shrink-0 text-[11.5px] font-medium tabular-nums font-proto"
            >
              {formatIDR(fromMinor(acc.currency, Math.abs(bal)))}
            </span>
            <span class="font-proto text-text-dim mb-[1px] w-6 shrink-0 text-right text-[9px]"
              >{acc.currency}</span
            >
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="border-line/40 mt-auto flex shrink-0 items-center border-t pt-1">
    <Pagination
      bind:currentPage={accPage}
      totalPages={totalAccPages}
      layout="spread"
      class="flex-1 text-[11px]"
    />
  </div>
</Card>
