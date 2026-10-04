<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import {
    getAccountLedgerCmd,
    getJournalEntryCmd,
    setPostingReconciledCmd,
    type AccountRunningLedgerItem,
    type ReconcileStatus,
    type CreateJournalEntryInput,
  } from '$lib/core/ipc/bindings';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { accountTypeLabel } from '$lib/core/format/account';
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import AccountModal from '$lib/features/accounts/components/AccountModal.svelte';
  import { createEmptyJournalDraft } from '$lib/features/journal/state/journalFormUtils';
  import type { Currency } from '$lib/core/types';
  import AccountLedgerSummary from './AccountLedgerSummary.svelte';
  import AccountLedgerTable from './AccountLedgerTable.svelte';
  import AccountLedgerFilterBar from './AccountLedgerFilterBar.svelte';
  import {
    isDebitNormal,
    checkAbnormalBalance,
    filterLedgerEntries,
    computeLedgerStats,
    computeStatusCounts,
  } from '../state/accountLedgerUtils';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { PageLayout, Card, Button, Splash, ErrorState } from '$lib/components/ui';

  let loadError = $state<string | null>(null);
  let ledgerItems = $state<AccountRunningLedgerItem[]>([]);

  const accountItems = $derived(accountsState.items);
  const loading = $derived(accountsState.loading && accountsState.items.length === 0);

  const code = $derived(page.params.code ?? '');
  const currentItem = $derived(accountItems.find((i) => i.account.code === code) ?? null);
  const account = $derived(currentItem?.account ?? null);
  const isPlaceholder = $derived(account?.placeholder ?? false);
  const balance = $derived(currentItem?.direct_balance ?? 0);

  async function loadData() {
    loadError = null;
    try {
      if (accountsState.items.length === 0 && !accountsState.loading) {
        await accountsState.load();
      }
      if (account) {
        ledgerItems = await getAccountLedgerCmd(account.id);
      } else {
        ledgerItems = [];
      }
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    if (account?.id) {
      getAccountLedgerCmd(account.id)
        .then((items) => {
          ledgerItems = items;
        })
        .catch((e) => {
          loadError = e instanceof Error ? e.message : String(e);
        });
    } else {
      ledgerItems = [];
    }
  });

  onMount(() => {
    loadData();
    const unsubTx = eventBus.on('transaction:posted', () => loadData());
    const unsubAcc = eventBus.on('accounts:changed', () => loadData());
    return () => {
      unsubTx();
      unsubAcc();
    };
  });

  let q = $state('');
  let from = $state('');
  let to = $state('');
  let reconcileFilter = $state<'ALL' | ReconcileStatus>('ALL');
  let addSubModalOpen = $state(false);

  const initialDraftForAccount = $derived.by((): CreateJournalEntryInput | undefined => {
    if (!account || isPlaceholder) return undefined;
    return createEmptyJournalDraft(account.id, '', (account.currency as Currency) || 'IDR');
  });

  function clearRange() {
    from = '';
    to = '';
    q = '';
    reconcileFilter = 'ALL';
  }

  const filteredEntries = $derived(filterLedgerEntries(ledgerItems, q, from, to, reconcileFilter));
  const statusCounts = $derived(computeStatusCounts(ledgerItems, q, from, to));
  const periodStats = $derived(computeLedgerStats(filteredEntries));
  const isAbnormal = $derived(checkAbnormalBalance(account, balance));

  async function handleEdit(entryId: string) {
    try {
      const entry = await getJournalEntryCmd(entryId);
      modalState.openInspector({ entry, isNew: false });
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  async function toggleReconcile(entry: AccountRunningLedgerItem) {
    const order: ReconcileStatus[] = ['n', 'c', 'y'];
    const cur = entry.reconcile;
    const next = order[(order.indexOf(cur) + 1) % 3];
    try {
      await setPostingReconciledCmd(entry.posting_id, next);
      if (account) {
        ledgerItems = await getAccountLedgerCmd(account.id);
      }
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
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
            variant="primary"
            class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
            title={i18n.t.addEntry}
            ariaLabel={i18n.t.addEntry}
            onclick={() => {
              modalState.openInspector({ draft: initialDraftForAccount, isNew: true });
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
      class="border-line bg-bg-card min-h-0 flex-1 border"
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
      class="border-line bg-bg-card flex min-h-0 flex-1 flex-col overflow-hidden border"
      padding={false}
      borderHeader
    >
      {#snippet header()}
        <div class="font-proto text-smaller shrink-0 text-right">
          <span class="text-text-muted">
            {ledgerItems.length}
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

      {#if isAbnormal}
        <div
          class="border-expense/40 bg-expense/10 text-expense font-proto text-smaller border-b px-3 py-1.5"
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
        totalCount={ledgerItems.length}
        onReset={clearRange}
      />

      <AccountLedgerSummary
        currency={account.currency}
        {periodStats}
        filteredCount={filteredEntries.length}
        totalCount={ledgerItems.length}
      />

      <AccountLedgerTable
        entries={filteredEntries}
        accountCurrency={account.currency}
        hasAnyEntries={ledgerItems.length > 0}
        {isPlaceholder}
        onAddSubAccount={() => (addSubModalOpen = true)}
        onToggleReconcile={toggleReconcile}
        onEdit={handleEdit}
        onNewEntry={() => {
          modalState.openInspector({ draft: initialDraftForAccount, isNew: true });
        }}
      />
    </Card>
  {/if}

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
