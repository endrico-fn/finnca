<script lang="ts">
  import type { Account, JournalEntryView } from '$lib/core/ipc/bindings';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { EmptyState, Card, Badge } from '$lib/components/ui';

  let {
    searchQuery,
    searchScope,
    entries = [],
    transactions = [],
    accountsById = new Map(),
    onNewTx,
    class: extraClass = '',
  }: {
    searchQuery: string;
    searchScope: 'ALL' | 'ACCOUNT' | 'RECENT';
    entries?: JournalEntryView[];
    transactions?: JournalEntryView[];
    accountsById?: Map<string, Account>;
    onNewTx?: () => void;
    class?: string;
  } = $props();

  const allEntries = $derived(entries.length > 0 ? entries : transactions);

  const filteredTransactions = $derived(
    allEntries
      .filter((t) => {
        if (!searchQuery.trim() || searchScope === 'ACCOUNT') return true;
        const q = searchQuery.toLowerCase();
        return (
          t.description.toLowerCase().includes(q) ||
          t.date.includes(q) ||
          (t.notes && t.notes.toLowerCase().includes(q)) ||
          t.postings.some((s) => s.memo?.toLowerCase().includes(q))
        );
      })
      .sort((a, b) => b.date.localeCompare(a.date))
  );

  let expandedId = $state<string | null>(null);

  function getTxMeta(tx: JournalEntryView) {
    const types = tx.postings
      .map((s) => s.account_type || accountsById.get(s.account_id)?.account_type)
      .filter(Boolean);

    if (types.includes('INCOME')) {
      return { category: i18n.t.txCatIncome, tone: 'ok' as const, sign: '+', color: 'text-income' };
    }
    if (types.includes('EXPENSE')) {
      return {
        category: i18n.t.txCatExpense,
        tone: 'err' as const,
        sign: '-',
        color: 'text-expense',
      };
    }
    if (types.includes('LIABILITY')) {
      return {
        category: i18n.t.txCatLiability,
        tone: 'warn' as const,
        sign: '',
        color: 'text-warning',
      };
    }
    if (types.includes('EQUITY')) {
      return {
        category: i18n.t.txCatEquity,
        tone: 'neutral' as const,
        sign: '',
        color: 'text-text-strong',
      };
    }
    if (types.filter((t) => t === 'ASSET').length >= 2) {
      return {
        category: i18n.t.txCatTransfer,
        tone: 'neutral' as const,
        sign: '↔ ',
        color: 'text-teal',
      };
    }
    return {
      category: i18n.t.txCatAsset,
      tone: 'neutral' as const,
      sign: '',
      color: 'text-text-base',
    };
  }

  function handleTxClick(tx: JournalEntryView) {
    modalState.openInspector({
      entry: tx,
      isNew: false,
    });
  }

  function toggleExpand(id: string, e: MouseEvent) {
    e.stopPropagation();
    expandedId = expandedId === id ? null : id;
  }
</script>

<Card
  title={i18n.t.recentTransactionsTitle}
  count={filteredTransactions.length}
  class="flex h-full flex-col {extraClass}"
>
  <div
    class="mt-2 min-h-0 flex-1 overflow-y-auto {filteredTransactions.length === 0 ? '' : 'pr-3'}"
  >
    {#if filteredTransactions.length === 0}
      <EmptyState
        title={i18n.t.noTransactionsFound}
        hint={i18n.t.journalEntriesAppearHere}
        actionLabel={i18n.t.recordEntry}
        onAction={() => (onNewTx ? onNewTx() : modalState.openInspector({ mode: 'journal' }))}
      />
    {:else}
      <div class="flex flex-col gap-0.5 pr-1">
        {#each filteredTransactions.slice(0, 40) as tx (tx.id)}
          {@const total = tx.postings
            .filter((s) => s.amount > 0)
            .reduce((s, sp) => s + sp.amount, 0)}
          {@const meta = getTxMeta(tx)}
          {@const isExpanded = expandedId === tx.id}
          <div class="group">
            <div
              role="button"
              tabindex="0"
              onclick={() => handleTxClick(tx)}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  handleTxClick(tx);
                }
              }}
              class="hover:bg-bg-btn flex cursor-pointer items-center gap-2 px-1.5 py-0.5 transition-colors"
            >
              <Badge size="s" tone={meta.tone} class="shrink-0">{meta.category}</Badge>
              <span class="font-proto text-text-muted text-smaller shrink-0 tabular-nums">
                {tx.date}
              </span>
              <span
                class="font-proto text-text-base group-hover:text-teal text-smaller truncate transition-colors"
              >
                {tx.description}
              </span>

              <div
                class="border-line/60 group-hover:border-line mb-0.5 flex-1 border-b border-dotted transition-colors"
              ></div>

              <span
                class="font-proto {meta.color} text-smaller shrink-0 font-medium whitespace-nowrap tabular-nums"
              >
                {meta.sign}{formatMinorGrouping(total, tx.currency)}
              </span>
              <span class="font-proto text-text-dim text-smaller w-7 shrink-0 text-right">
                {tx.currency}
              </span>

              {#if tx.postings.length > 1}
                <button
                  type="button"
                  onclick={(e) => toggleExpand(tx.id, e)}
                  class="text-text-muted hover:text-teal font-proto text-smaller shrink-0 transition-colors"
                  aria-label={i18n.t.splitsLabel}
                >
                  {isExpanded ? '▲' : '▼'}
                </button>
              {/if}
            </div>

            {#if isExpanded}
              <div class="anim-row-expand border-line/30 border-b pr-2 pb-1 pl-19">
                {#each tx.postings as posting (posting.id ?? posting.account_id)}
                  <div class="text-smaller flex items-center justify-between gap-2 py-0.5">
                    <span class="text-text-dim font-proto truncate">
                      {posting.account_name ||
                        accountsById.get(posting.account_id)?.name ||
                        posting.account_code ||
                        posting.account_id}
                    </span>
                    <span class="font-proto text-text-muted shrink-0 tabular-nums">
                      {posting.action === 'DEBIT' ? 'Dr' : 'Cr'}
                      {formatMinorGrouping(posting.amount, tx.currency)}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</Card>
