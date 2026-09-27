<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { journalState } from '../state/journalDraft.svelte';
  import {
    listAccountsCmd,
    type Account,
    type JournalEntryView,
  } from '$lib/core/ipc/bindings';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { closingBooksState } from '$lib/core/state/ledgerLock.svelte';
  import JournalFilterBar from './JournalFilterBar.svelte';
  import JournalTotalsBar from './JournalTotalsBar.svelte';
  import JournalTable from './JournalTable.svelte';
  import { Splash, ErrorState, PageLayout, Card, Button, Icon } from '$lib/components/ui';
  import {
    filterTransactions,
    filterBaseTransactions,
    computeAccountTxCounts,
    computeJournalTotals,
  } from '../state/journalFilterUtils';

  let accounts = $state<Account[]>([]);
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));

  let q = $state('');
  let from = $state('');
  let to = $state('');
  let accFilter = $state('');
  let statusFilter = $state<'all' | 'due' | 'overdue'>('all');
  let expandedId = $state<string | null>(null);
  let txPage = $state(1);
  const TX_PAGE_SIZE = 20;

  onMount(() => {
    closingBooksState.load();
    journalState.loadEntries();
    listAccountsCmd()
      .then((items) => {
        accounts = items.map((i) => i.account);
      })
      .catch(() => {
        accounts = [];
      });
    if (page.url.searchParams.get('new') === '1' || page.url.searchParams.get('new') === 'true') {
      modalState.openQuickTx();
    }
    const filterParam = page.url.searchParams.get('filter');
    if (filterParam === 'due' || filterParam === 'overdue') {
      statusFilter = filterParam;
    }
    const unsubTx = eventBus.on('transaction:posted', () => journalState.loadEntries());
    const unsubAcc = eventBus.on('accounts:changed', () => journalState.loadEntries());
    return () => {
      unsubTx();
      unsubAcc();
    };
  });

  function resetTxPage() {
    txPage = 1;
  }

  const filtered = $derived(
    filterTransactions(journalState.entries, q, from, to, accFilter, statusFilter)
  );

  const totalTxPages = $derived(Math.max(1, Math.ceil(filtered.length / TX_PAGE_SIZE)));
  const pagedTxs = $derived(filtered.slice((txPage - 1) * TX_PAGE_SIZE, txPage * TX_PAGE_SIZE));

  const baseTxs = $derived(filterBaseTransactions(journalState.entries, q, from, to, statusFilter));
  const accTxCounts = $derived(computeAccountTxCounts(baseTxs));

  const leafAccounts = $derived(
    accounts
      .filter((a) => !a.placeholder)
      .sort((a, b) => a.code.localeCompare(b.code, undefined, { numeric: true }))
  );

  const selectedAcc = $derived(accFilter ? accountsById.get(accFilter) : undefined);

  const totals = $derived(computeJournalTotals(filtered));
  const idrDebit = $derived(totals.idrDebit);
  const idrCredit = $derived(totals.idrCredit);
  const usdDebit = $derived(totals.usdDebit);
  const usdCredit = $derived(totals.usdCredit);

  let selectedRowIndex = $state<number | null>(null);

  function handleKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement) return;

    if (e.key === 'j' || e.key === 'ArrowDown') {
      e.preventDefault();
      if (selectedRowIndex === null) selectedRowIndex = 0;
      else selectedRowIndex = Math.min(pagedTxs.length - 1, selectedRowIndex + 1);

      const tx = pagedTxs[selectedRowIndex];
      if (tx) {
        document.getElementById(`tx-row-${tx.id}`)?.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'k' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (selectedRowIndex === null) selectedRowIndex = pagedTxs.length - 1;
      else selectedRowIndex = Math.max(0, selectedRowIndex - 1);

      const tx = pagedTxs[selectedRowIndex];
      if (tx) {
        document.getElementById(`tx-row-${tx.id}`)?.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === ' ' || e.key === 'x') {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        expandedId = expandedId === tx.id ? null : tx.id;
      }
    } else if (e.key === 'Enter' || e.key === 'e') {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        if (!closingBooksState.isDateLocked(tx.date)) {
          modalState.openInspector({ entry: tx, isNew: false });
          expandedId = null;
        }
      }
    } else if (e.key === 'n' || e.key === 'N') {
      e.preventDefault();
      expandedId = null;
      modalState.openQuickTx();
    } else if (e.key === 't' || e.key === 'T') {
      e.preventDefault();
      expandedId = null;
      modalState.openTransfer();
    } else if (e.altKey && (e.key === 'd' || e.key === 'D')) {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        expandedId = null;
        modalState.openQuickTx({
          id: crypto.randomUUID(),
          date: todayString(),
          description: `${tx.description} (Copy)`,
          reference_no: '',
          due_date: null,
          plan_id: null,
          currency: tx.currency,
          fx_rate: tx.fx_rate,
          notes: tx.notes,
          postings: tx.postings.map((p) => ({
            account_id: p.account_id,
            amount: p.amount,
            memo: p.memo,
            reconcile: 'n',
          })),
        });
      }
    } else if (e.key === 'Escape') {
      if (modalState.inspectorOpen) {
        modalState.closeInspector();
      } else {
        selectedRowIndex = null;
        expandedId = null;
      }
    }
  }

  function handleToggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function handleToggleEdit(tx: JournalEntryView) {
    if (closingBooksState.isDateLocked(tx.date)) return;
    if (modalState.inspectorOpen && modalState.inspectorEntry?.id === tx.id) {
      modalState.closeInspector();
    } else {
      modalState.openInspector({ entry: tx, isNew: false });
      expandedId = null;
    }
  }

  function handleSettle(tx: JournalEntryView) {
    modalState.confirm({
      title: i18n.t.settleInvoiceTitle,
      message: `${tx.description} (${tx.reference_no ?? tx.date})`,
      confirmLabel: i18n.t.settleInvoiceAction,
      cancelLabel: i18n.t.cancelModalBtn,
      danger: false,
      onConfirm: async () => {
        const cleanNotes = tx.notes?.replace(/\[Settled\]/g, '').trim() || '';
        const newNotes = cleanNotes ? `${cleanNotes} [Settled]` : '[Settled]';
        const cleanDue = tx.due_date?.replace(/\s*\[Settled\]/i, '').trim() || null;
        await journalState.saveEntry({
          id: tx.id,
          date: tx.date,
          description: tx.description,
          reference_no: tx.reference_no,
          due_date: cleanDue,
          plan_id: tx.plan_id,
          currency: tx.currency,
          fx_rate: tx.fx_rate,
          notes: newNotes,
          postings: tx.postings.map((p) => ({
            id: p.id,
            account_id: p.account_id,
            amount: p.amount,
            memo: p.memo,
            reconcile: p.reconcile,
          })),
        });
      },
    });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<PageLayout title={i18n.t.journal}>
  {#snippet actions()}
    <div class="flex shrink-0 items-center gap-1.5">
      <Button
        variant="ghost"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        ariaLabel={i18n.t.quickTransferTitle}
        onclick={() => {
          expandedId = null;
          modalState.openTransfer();
        }}
      >
        <span class="flex items-center gap-1.5">
          <Icon name="transfer" size={13} />
          <span>{i18n.t.modeTransfer}</span>
          <span class="border-line/80 text-text-dim border px-1 py-0.5 text-smaller">T</span>
        </span>
      </Button>

      <Button
        variant="primary"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        ariaLabel={i18n.t.newEntry}
        onclick={() => {
          expandedId = null;
          modalState.openQuickTx();
        }}
      >
        <span class="flex items-center gap-1.5">
          <span>{i18n.t.newEntry}</span>
          <span class="border-line/80 text-text-dim border px-1 py-0.5 text-smaller">N</span>
        </span>
      </Button>
    </div>
  {/snippet}

  {#if journalState.loading && journalState.entries.length === 0}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if journalState.error}
    <ErrorState message={journalState.error} onRetry={() => journalState.loadEntries()} />
  {:else}
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <Card padding={false} class="border-line bg-bg-card min-h-0 min-w-0 flex-1 flex flex-col overflow-hidden border">
        <JournalFilterBar
          bind:q
          bind:from
          bind:to
          bind:accFilter
          bind:statusFilter
          {leafAccounts}
          {accountsById}
          {accTxCounts}
          baseTxsCount={baseTxs.length}
          filteredCount={filtered.length}
          totalCount={journalState.entries.length}
          {selectedAcc}
          onFilterChange={resetTxPage}
        />

        <JournalTable
          pagedEntries={pagedTxs}
          {accountsById}
          editing={modalState.inspectorEntry}
          {expandedId}
          {selectedRowIndex}
          totalCount={journalState.entries.length}
          onToggleExpand={handleToggleExpand}
          onToggleEdit={handleToggleEdit}
          onSelectRow={(idx) => (selectedRowIndex = idx)}
          onSettle={handleSettle}
        />

        <JournalTotalsBar
          {idrDebit}
          {idrCredit}
          {usdDebit}
          {usdCredit}
          bind:txPage
          {totalTxPages}
          filteredCount={filtered.length}
        />
      </Card>
    </div>
  {/if}
</PageLayout>
