<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import {
    journalEntryToTransaction,
    type Transaction,
    type ReconcileState,
    type Currency,
  } from '$lib/core/types';
  import {
    listJournalEntriesCmd,
    postJournalEntryCmd,
    updateJournalEntryCmd,
    deleteJournalEntryCmd,
    type JournalEntryView,
    type PostingInput,
  } from '$lib/core/ipc/bindings';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { accountTypeLabel } from '$lib/core/format/account';
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import JournalEntryForm from '$lib/features/journal/components/JournalEntryForm.svelte';
  import AccountModal from '$lib/features/accounts/components/AccountModal.svelte';
  import { todayString } from '$lib/core/format/date';
  import AccountLedgerSummary from './AccountLedgerSummary.svelte';
  import AccountLedgerTable, { type LedgerEntryRow } from './AccountLedgerTable.svelte';
  import AccountLedgerFilterBar from './AccountLedgerFilterBar.svelte';
  import {
    isDebitNormal,
    checkAbnormalBalance,
    buildAccountEntries,
    filterLedgerEntries,
    computeLedgerStats,
    computeStatusCounts,
  } from '../state/accountLedgerUtils';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { PageLayout, Card, Button, ModalShell, Splash, ErrorState } from '$lib/components/ui';

  let loadError = $state<string | null>(null);
  let entries = $state<JournalEntryView[]>([]);

  const accountItems = $derived(accountsState.items);
  const loading = $derived(accountsState.loading && accountsState.items.length === 0);

  const accounts = $derived(accountItems.map((i) => i.account));
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));

  const code = $derived(page.params.code ?? '');
  const currentItem = $derived(accountItems.find((i) => i.account.code === code) ?? null);
  const account = $derived(currentItem?.account ?? null);
  const isPlaceholder = $derived(account?.placeholder ?? false);
  const balance = $derived(currentItem?.direct_balance ?? 0);

  const transactions = $derived(entries.map(journalEntryToTransaction));

  async function loadData() {
    loadError = null;
    try {
      const [entryList] = await Promise.all([
        listJournalEntriesCmd(),
        accountsState.items.length === 0 && !accountsState.loading
          ? accountsState.load()
          : Promise.resolve(),
      ]);
      entries = entryList;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(() => {
    loadData();
    const unsubTx = eventBus.on('transaction:posted', () => loadData());
    const unsubAcc = eventBus.on('accounts:changed', () => loadData());
    return () => {
      unsubTx();
      unsubAcc();
    };
  });

  const allEntries = $derived(buildAccountEntries(account, transactions));

  let q = $state('');
  let from = $state('');
  let to = $state('');
  let reconcileFilter = $state<'ALL' | ReconcileState>('ALL');
  let showNew = $state(false);
  let editing = $state<Transaction | null>(null);
  let transferOpen = $state(false);
  let addSubModalOpen = $state(false);

  const initialDraftForAccount = $derived.by((): Transaction | undefined => {
    if (!account || isPlaceholder) return undefined;
    return {
      id: crypto.randomUUID(),
      date: todayString(),
      dueDate: '',
      settled: false,
      description: '',
      num: '',
      notes: '',
      currency: (account.currency as Currency) || 'IDR',
      splits: [
        {
          id: crypto.randomUUID(),
          accountId: account.id,
          amount: 0,
          reconcile: 'n',
        },
        {
          id: crypto.randomUUID(),
          accountId: '',
          amount: 0,
          reconcile: 'n',
        },
      ],
    };
  });

  function clearRange() {
    from = '';
    to = '';
    q = '';
    reconcileFilter = 'ALL';
  }

  const filteredEntries = $derived(filterLedgerEntries(allEntries, q, from, to, reconcileFilter));
  const statusCounts = $derived(computeStatusCounts(allEntries, q, from, to));
  const periodStats = $derived(computeLedgerStats(filteredEntries));
  const isAbnormal = $derived(checkAbnormalBalance(account, balance));

  async function persistTx(tx: Transaction) {
    const postings: PostingInput[] = tx.splits.map((s) => ({
      id: s.id || undefined,
      account_id: s.accountId,
      amount: Math.round(s.amount),
      memo: s.memo || null,
      action: null,
      reconcile: (s.reconcile === 'y' ? 'y' : s.reconcile === 'c' ? 'c' : null) as 'c' | 'y' | null,
    }));

    if (tx.id && entries.some((e) => e.id === tx.id)) {
      await updateJournalEntryCmd(tx.id, {
        date: tx.date,
        description: tx.description,
        notes: tx.notes || null,
        currency: tx.currency,
        fx_rate: tx.fxRateAtTransaction ?? null,
        postings,
      });
    } else {
      await postJournalEntryCmd({
        date: tx.date,
        description: tx.description,
        notes: tx.notes || null,
        currency: tx.currency,
        fx_rate: tx.fxRateAtTransaction ?? null,
        postings,
      });
    }
    eventBus.emit('transaction:posted', { id: tx.id });
    eventBus.emit('accounts:changed', undefined);
  }

  async function handleSave(tx: Transaction) {
    await persistTx(tx);
    await loadData();
    editing = null;
    showNew = false;
  }

  async function toggleReconcile(entry: LedgerEntryRow) {
    const order: ReconcileState[] = ['n', 'c', 'y'];
    const cur = entry.split.reconcile;
    const next = order[(order.indexOf(cur) + 1) % 3];
    const cloned: Transaction = $state.snapshot(entry.tx);
    const target = cloned.splits.find((s) => s.id === entry.split.id);
    if (target) {
      target.reconcile = next;
      await persistTx(cloned);
      await loadData();
    }
  }
</script>

<PageLayout
  crumb={i18n.t.account}
  crumbHref="/app/accounts"
  title={account
    ? `${account.code} — ${account.name}`
    : i18n.t.accountNotFound.replace('{code}', code)}
>
  {#snippet actions()}
    {#if account}
      {#if !isPlaceholder}
        <div class="flex shrink-0 items-center gap-1.5">
          <Button
            variant="ghost"
            class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
            title={i18n.t.transfersTitle}
            ariaLabel={i18n.t.transfersTitle}
            onclick={() => (transferOpen = true)}
          >
            {i18n.t.transfersTitle}
          </Button>
          <Button
            variant="primary"
            class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
            title={i18n.t.addEntry}
            ariaLabel={i18n.t.addEntry}
            onclick={() => {
              showNew = true;
              editing = null;
            }}
          >
            {i18n.t.addEntry}
          </Button>
        </div>
      {:else}
        <div class="flex shrink-0 items-center gap-1.5">
          <Button
            variant="primary"
            class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
            title={i18n.t.subAccount}
            ariaLabel={i18n.t.subAccount}
            onclick={() => (addSubModalOpen = true)}
          >
            {i18n.t.subAccount}
          </Button>
          <Button
            variant="ghost"
            class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
            title={i18n.t.editAccount}
            ariaLabel={i18n.t.editAccount}
            href="/app/accounts"
          >
            {i18n.t.editAccount}
          </Button>
        </div>
      {/if}
    {/if}
  {/snippet}

  {#if loading}
    <div class="sharp-card flex flex-1 items-center justify-center p-8">
      <Splash variant="inline" label={i18n.t.loadingFinnca} />
    </div>
  {:else if loadError && !account}
    <ErrorState message={loadError} onRetry={loadData} />
  {:else if !account}
    <div class="sharp-card flex flex-1 items-center justify-center p-8">
      <div class="text-center">
        <p class="text-expense text-small">
          {i18n.t.accountNotFound.replace('{code}', code)}
        </p>
        <Button variant="primary" href="/app/accounts" class="mt-4 inline-block">
          {i18n.t.backToAccounts}
        </Button>
      </div>
    </div>
  {:else if isPlaceholder}
    <Card
      title="{account.code} — {account.name}"
      badge={i18n.t.badgePhPlaceholder}
      badgeTone="warn"
      class="min-h-0 flex-1"
    >
      <div class="space-y-2 p-3">
        <p class="text-text-muted text-small">{i18n.t.placeholderNotPostable}</p>
        <p class="text-text-muted text-small">{i18n.t.placeholderGroupDetail}</p>
        <div class="mt-4 flex gap-2 pt-2">
          <Button variant="primary" href="/app/accounts">{i18n.t.editInAccount}</Button>
          <Button variant="ghost" href="/app/accounts">{i18n.t.backToAccounts}</Button>
        </div>
      </div>
    </Card>
  {:else}
    <Card
      title="{account.code} — {account.name}"
      description={`${accountTypeLabel(account.account_type)} • ${account.currency} • ${i18n.t.accountPostableTag}${account.note ? ` • ${account.note}` : ''}`}
      class="min-h-0 flex-1"
      padding={false}
    >
      {#snippet header()}
        <div class="font-proto text-smaller shrink-0 text-right">
          <span class="text-text-muted">
            {allEntries.length}
            {i18n.t.entriesLabel} • {filteredEntries.length}
            {i18n.t.filteredLabel}
          </span>
          <span class="text-line mx-2">|</span>
          <span class="text-text-dim">
            {i18n.t.runningBalanceLabel}
            <strong class="text-text-strong ml-1 font-bold tabular-nums">
              {formatMinorToDisplay(balance, account.currency)}
            </strong>
          </span>
        </div>
      {/snippet}

      <div class="flex flex-col gap-2 px-3 pt-2 pb-2">
        {#if isAbnormal}
          <div
            class="border-expense/40 bg-expense/10 text-expense font-proto text-smaller border px-2 py-1"
          >
            {i18n.t.alertTag}{i18n.t.abnormalBalance.replace(
              '{type}',
              accountTypeLabel(account.account_type)
            )}
            ({isDebitNormal(account.account_type) ? i18n.t.shouldBeDebit : i18n.t.shouldBeCredit})
          </div>
        {/if}

        <AccountLedgerFilterBar
          bind:q
          bind:from
          bind:to
          bind:reconcileFilter
          {statusCounts}
          filteredCount={filteredEntries.length}
          totalCount={allEntries.length}
          onReset={clearRange}
        />

        <AccountLedgerSummary
          currency={account.currency}
          {periodStats}
          filteredCount={filteredEntries.length}
          totalCount={allEntries.length}
        />
      </div>

      {#if editing}
        <div class="mb-3 shrink-0 px-3 pt-1 pb-2">
          <svelte:boundary>
            <JournalEntryForm
              tx={editing}
              onSave={handleSave}
              onCancel={() => (editing = null)}
              onDelete={async (id: string) => {
                const tx = transactions.find((t) => t.id === id);
                modalState.confirm({
                  title: i18n.t.confirmDeleteJournalTitle,
                  message: i18n.t.confirmDeleteJournalMsg.replace('{desc}', tx?.description ?? id),
                  confirmLabel: i18n.t.confirmBtn,
                  cancelLabel: i18n.t.cancelModalBtn,
                  danger: true,
                  onConfirm: async () => {
                    await deleteJournalEntryCmd(id);
                    eventBus.emit('transaction:posted', { id });
                    eventBus.emit('accounts:changed', undefined);
                    await loadData();
                    editing = null;
                  },
                });
              }}
            />
            {#snippet failed()}
              <div class="badge-err font-proto text-small mt-2 px-3 py-2">
                {i18n.t.quickTxLoadFailed}
              </div>
            {/snippet}
          </svelte:boundary>
        </div>
      {/if}

      <AccountLedgerTable
        entries={filteredEntries}
        {accountsById}
        accountCurrency={account.currency}
        hasAnyEntries={allEntries.length > 0}
        {isPlaceholder}
        onAddSubAccount={() => (addSubModalOpen = true)}
        onToggleReconcile={toggleReconcile}
        onEdit={(tx) => (editing = tx)}
        onNewEntry={() => {
          showNew = true;
          editing = null;
        }}
      />
    </Card>
  {/if}

  {#if showNew && account && !isPlaceholder}
    <ModalShell
      bind:open={showNew}
      title={i18n.t.addEntry}
      maxWidth="max-w-5xl"
      onClose={() => (showNew = false)}
    >
      <svelte:boundary>
        <JournalEntryForm
          tx={null}
          initialDraft={initialDraftForAccount}
          onSave={handleSave}
          onCancel={() => (showNew = false)}
        />
        {#snippet failed()}
          <div class="badge-err font-proto text-small mt-2 px-3 py-2">
            {i18n.t.quickTxLoadFailed}
          </div>
        {/snippet}
      </svelte:boundary>
    </ModalShell>
  {/if}

  <TransferModal bind:open={transferOpen} initialFrom={account?.id ?? ''} onSuccess={loadData} />

  {#if account}
    <AccountModal
      bind:open={addSubModalOpen}
      parentId={account.id}
      onSaved={async () => {
        await loadData();
        await accountsState.load();
      }}
    />
  {/if}
</PageLayout>
