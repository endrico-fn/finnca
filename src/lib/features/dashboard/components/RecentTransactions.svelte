<script lang="ts">
  import type { Transaction } from '$lib/core/types';
  import type { Account } from '$lib/core/ipc/bindings';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { EmptyState, Card, Badge } from '$lib/components/ui';

  let {
    searchQuery,
    searchScope,
    transactions = [],
    accountsById = new Map(),
    onNewTx,
  }: {
    searchQuery: string;
    searchScope: 'ALL' | 'ACCOUNT' | 'RECENT';
    transactions?: Transaction[];
    accountsById?: Map<string, Account>;
    onNewTx?: () => void;
  } = $props();

  const filteredTransactions = $derived(
    transactions
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

  function getTxCategory(tx: Transaction): string {
    const types = tx.splits.map((s) => accountsById.get(s.accountId)?.account_type).filter(Boolean);
    if (types.includes('INCOME')) return i18n.t.txCatIncome;
    if (types.includes('EXPENSE')) return i18n.t.txCatExpense;
    if (types.includes('LIABILITY')) return i18n.t.txCatLiability;
    if (types.includes('EQUITY')) return i18n.t.txCatEquity;
    if (types.filter((t) => t === 'ASSET').length >= 2) return i18n.t.txCatTransfer;
    return i18n.t.txCatAsset;
  }
</script>

<Card title={i18n.t.recentTransactionsTitle} class="col-span-4 h-full">
  <div class="mt-2 min-h-0 flex-1 overflow-y-auto pr-2.5">
    {#each filteredTransactions.slice(0, 16) as tx (tx.id)}
      {@const total = tx.splits.filter((s) => s.amount > 0).reduce((s, sp) => s + sp.amount, 0)}
      {@const category = getTxCategory(tx)}
      <div
        class="border-line/30 font-proto hover:bg-bg-btn text-smaller flex items-baseline justify-between gap-2 border-b px-1 py-1 transition-colors first:pt-0 last:border-0"
      >
        <div class="flex min-w-0 items-baseline gap-1.5">
          <Badge size="s" tone="neutral">{category}</Badge>
          <span class="text-text-muted shrink-0 tabular-nums">{tx.date}</span>
          <span class="text-text-strong truncate">{tx.description}</span>
        </div>
        <span
          class="font-proto text-text-base shrink-0 text-right font-bold whitespace-nowrap tabular-nums"
        >
          {formatMinorToDisplay(total, tx.currency)}
        </span>
      </div>
    {:else}
      <EmptyState
        title={i18n.t.noTransactionsFound}
        hint={i18n.t.journalEntriesAppearHere}
        actionLabel={i18n.t.recordEntry}
        onAction={() => (onNewTx ? onNewTx() : modalState.openQuickTx())}
      />
    {/each}
  </div>
</Card>
