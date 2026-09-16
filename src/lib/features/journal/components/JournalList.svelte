<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { journalState } from '../state/journalDraft.svelte';
  import { listAccountsCmd, type Account } from '$lib/core/ipc/bindings';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import JournalFilterBar from './JournalFilterBar.svelte';
  import JournalTotalsBar from './JournalTotalsBar.svelte';
  import JournalTable from './JournalTable.svelte';
  import TransferModal from './TransferModal.svelte';
  import type { Transaction } from '../state/journalDraft.svelte';
  import { Splash, ErrorState, PageLayout, Card, Button } from '$lib/components/ui';
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
  let editing = $state<Transaction | null>(null);
  let expandedId = $state<string | null>(null);
  let txPage = $state(1);
  let transferOpen = $state(false);
  const TX_PAGE_SIZE = 20;

  onMount(() => {
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

  const filtered = $derived(filterTransactions(journalState.transactions, q, from, to, accFilter));

  const totalTxPages = $derived(Math.max(1, Math.ceil(filtered.length / TX_PAGE_SIZE)));
  const pagedTxs = $derived(filtered.slice((txPage - 1) * TX_PAGE_SIZE, txPage * TX_PAGE_SIZE));

  const baseTxs = $derived(filterBaseTransactions(journalState.transactions, q, from, to));
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
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    if (e.key === 'j') {
      e.preventDefault();
      if (selectedRowIndex === null) selectedRowIndex = 0;
      else selectedRowIndex = Math.min(pagedTxs.length - 1, selectedRowIndex + 1);

      const tx = pagedTxs[selectedRowIndex];
      if (tx) {
        document.getElementById(`tx-row-${tx.id}`)?.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'k') {
      e.preventDefault();
      if (selectedRowIndex === null) selectedRowIndex = pagedTxs.length - 1;
      else selectedRowIndex = Math.max(0, selectedRowIndex - 1);

      const tx = pagedTxs[selectedRowIndex];
      if (tx) {
        document.getElementById(`tx-row-${tx.id}`)?.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'Enter' || e.key === 'x') {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        expandedId = expandedId === tx.id ? null : tx.id;
      }
    } else if (e.key === 'e') {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        editing = editing?.id === tx.id ? null : tx;
        expandedId = null;
      }
    } else if (e.key === 'Escape') {
      selectedRowIndex = null;
      expandedId = null;
    }
  }

  async function handleSave(tx: Transaction) {
    await journalState.saveTransaction(tx);
    editing = null;
  }

  async function handleDelete(id: string) {
    const tx = journalState.transactions.find((t) => t.id === id);
    modalState.confirm({
      title: i18n.t.confirmDeleteJournalTitle,
      message: i18n.t.confirmDeleteJournalMsg.replace('{desc}', tx?.description ?? id),
      confirmLabel: i18n.t.confirmBtn,
      cancelLabel: i18n.t.cancelModalBtn,
      danger: true,
      onConfirm: async () => {
        await journalState.deleteTransaction(id);
        editing = null;
      },
    });
  }

  function handleToggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function handleToggleEdit(tx: Transaction) {
    editing = editing?.id === tx.id ? null : tx;
    expandedId = null;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<PageLayout title={i18n.t.journal}>
  {#snippet actions()}
    <div class="flex shrink-0 items-center gap-1.5">
      <Button
        variant="primary"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={i18n.t.newTransaction}
        ariaLabel={i18n.t.newTransaction}
        onclick={() => {
          editing = null;
          expandedId = null;
          modalState.openQuickTx();
        }}
      >
        {i18n.t.newTransaction}
      </Button>
      <Button
        variant="ghost"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={i18n.t.transfersTitle}
        ariaLabel={i18n.t.transfersTitle}
        onclick={() => (transferOpen = true)}
      >
        {i18n.t.transfersTitle}
      </Button>
    </div>
  {/snippet}

  {#if journalState.loading && journalState.entries.length === 0}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if journalState.error}
    <ErrorState message={journalState.error} onRetry={() => journalState.loadEntries()} />
  {:else}
    <Card padding={false} class="min-h-0 flex-1">
      <JournalFilterBar
        bind:q
        bind:from
        bind:to
        bind:accFilter
        {leafAccounts}
        {accTxCounts}
        baseTxsCount={baseTxs.length}
        filteredCount={filtered.length}
        totalCount={journalState.transactions.length}
        {selectedAcc}
        onFilterChange={resetTxPage}
      />

      <JournalTable
        {pagedTxs}
        {accountsById}
        {editing}
        {expandedId}
        {selectedRowIndex}
        totalCount={journalState.transactions.length}
        onToggleExpand={handleToggleExpand}
        onToggleEdit={handleToggleEdit}
        onSave={handleSave}
        onDelete={handleDelete}
        onSelectRow={(idx) => (selectedRowIndex = idx)}
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
  {/if}

  <TransferModal bind:open={transferOpen} onSuccess={() => journalState.loadEntries()} />
</PageLayout>
