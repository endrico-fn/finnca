<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { fromMinor, formatIDR } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import { EmptyState, Card } from '$lib/components/ui';

  let { searchQuery, searchScope } = $props<{
    searchQuery: string;
    searchScope: 'ALL' | 'ACCOUNT' | 'RECENT';
  }>();

  const filteredTransactions = $derived(
    ledger.transactions
      .filter((t) => {
        if (!searchQuery.trim() || searchScope === 'ACCOUNT') return true;
        const q = searchQuery.toLowerCase();
        return (
          t.description.toLowerCase().includes(q) ||
          t.date.includes(q) ||
          (t.num && t.num.toLowerCase().includes(q)) ||
          t.splits.some((s) => s.memo?.toLowerCase().includes(q))
        );
      })
      .sort((a, b) => b.date.localeCompare(a.date))
  );

  function txClass(tx: (typeof ledger.transactions)[0]): 'income' | 'expense' | 'transfer' {
    const byId = ledger.accountsById;
    if (tx.splits.some((s) => byId.get(s.accountId)?.type === 'INCOME')) return 'income';
    if (tx.splits.some((s) => byId.get(s.accountId)?.type === 'EXPENSE')) return 'expense';
    return 'transfer';
  }
</script>

<Card title={i18n.t.recentTransaction + 'S'} class="col-span-4 h-full">
  <div class="mt-1 min-h-0 flex-1 overflow-y-auto pr-1">
    {#each filteredTransactions.slice(0, 16) as tx (tx.id)}
      {@const total = tx.splits.filter((s) => s.amount > 0).reduce((s, sp) => s + sp.amount, 0)}
      {@const kind = txClass(tx)}
      {@const amtColor =
        kind === 'income' ? 'text-income' : kind === 'expense' ? 'text-expense' : 'text-text-base'}
      <div
        class="border-line/30 font-proto hover:bg-bg-row-hover -mx-1 flex justify-between gap-3 border-b px-1 py-2 text-[11px] transition-colors last:border-0"
      >
        <div class="flex min-w-0 items-center gap-2">
          <span class="text-text-muted shrink-0">{tx.date}</span>
          <span class="text-text-strong truncate">{tx.description}</span>
        </div>
        <span class="{amtColor} shrink-0 text-right font-bold whitespace-nowrap tabular-nums font-proto">
          {formatIDR(fromMinor(tx.currency, total))}
        </span>
      </div>
    {:else}
      <EmptyState
        title={i18n.t.noTransactionsFound}
        hint={i18n.t.journalEntriesAppearHere}
        actionLabel={i18n.t.recordEntry}
        actionHref="/app/journal"
      />
    {/each}
  </div>
</Card>
