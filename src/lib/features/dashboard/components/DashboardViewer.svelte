<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteDate } from 'svelte/reactivity';
  import {
    listAccountsCmd,
    listJournalEntriesCmd,
    getLedgerTotalsCmd,
    type AccountBalanceView,
    type JournalEntryView,
    type LedgerTotalsView,
  } from '$lib/core/ipc/bindings';
  import type { Transaction } from '$lib/core/types';
  import { getPref } from '$lib/core/state/prefs';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import TransferModal from '$lib/features/journal/components/TransferModal.svelte';
  import { Splash, ErrorState, PageLayout, Button, SearchBar } from '$lib/components/ui';

  import { todayString, diffCalendarDays } from '$lib/core/format/date';

  import SummaryCards from './SummaryCards.svelte';
  import CashflowCard from './CashflowCard.svelte';
  import AccountList from './AccountList.svelte';
  import RecentTransactions from './RecentTransactions.svelte';

  let loading = $state(true);
  let error = $state<string | null>(null);
  let accountItems = $state<AccountBalanceView[]>([]);
  let entries = $state<JournalEntryView[]>([]);
  let totals = $state<LedgerTotalsView | null>(null);

  function toTransaction(entry: JournalEntryView): Transaction {
    return {
      id: entry.id,
      date: entry.date,
      description: entry.description,
      notes: entry.notes ?? undefined,
      currency: entry.currency as 'IDR' | 'USD',
      fxRateAtTransaction: entry.fx_rate,
      splits: entry.postings.map((p) => ({
        id: p.id,
        accountId: p.account_id,
        amount: p.amount,
        memo: p.memo ?? undefined,
        reconcile: p.reconcile === 'y' ? 'y' : p.reconcile === 'c' ? 'c' : 'n',
      })),
    };
  }

  const transactions = $derived(entries.map(toTransaction));

  async function loadDashboard() {
    loading = true;
    error = null;
    try {
      const [entryList, tot, items] = await Promise.all([
        listJournalEntriesCmd(100),
        getLedgerTotalsCmd(),
        listAccountsCmd(),
      ]);
      entries = entryList;
      totals = tot;
      accountItems = items;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadDashboard();
    const unsubTx = eventBus.on('transaction:posted', () => loadDashboard());
    const unsubAcc = eventBus.on('accounts:changed', () => loadDashboard());
    return () => {
      unsubTx();
      unsubAcc();
    };
  });

  const assets = $derived(totals?.total_assets ?? 0);
  const liabilities = $derived(totals?.total_liabilities ?? 0);
  const netWorth = $derived(assets - liabilities);
  const debtRatio = $derived(assets > 0 ? ((liabilities / assets) * 100).toFixed(1) : '0.0');
  const fxRate = $derived(getPref('finnca_fx_rate', 16000));
  const usdAssets = $derived(fxRate > 0 ? Math.round((assets / fxRate) * 100) : 0);

  const unbalancedCount = $derived(totals && !totals.is_balance_sheet_aligned ? 1 : 0);
  const accounts = $derived(accountItems.map((i) => i.account));
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));
  const visibleAccounts = $derived(accounts.filter((a) => !a.hidden));

  const uncategorizedCount = $derived(
    transactions.filter((t) =>
      t.splits.some((s) => {
        const acc = accountsById.get(s.accountId);
        if (!acc || acc.placeholder) return true;
        const name = acc.name.toLowerCase();
        return (
          name.includes('uncategorized') || name.includes('imbalance') || name.includes('unknown')
        );
      })
    ).length
  );
  const pendingCount = $derived(
    transactions.filter((t) => t.splits.some((s) => s.reconcile === 'c')).length
  );

  const reconciledDates = $derived(
    transactions
      .filter((t) => t.splits.some((s) => s.reconcile === 'y'))
      .map((t) => t.date)
      .sort((a, b) => b.localeCompare(a))
  );
  const lastReconText = $derived.by(() => {
    if (reconciledDates.length === 0) {
      return transactions.length === 0 ? i18n.t.reconNever : i18n.t.reconPending;
    }
    const days = Math.max(0, diffCalendarDays(todayString(), reconciledDates[0]));
    return days === 0 ? i18n.t.reconToday : i18n.t.reconDaysAgo.replace('{days}', String(days));
  });

  let transferOpen = $state(false);

  let searchQuery = $state<string>('');
  let searchInput = $state<string>('');
  let searchScope = $state<'ALL' | 'ACCOUNT' | 'RECENT'>('ALL');

  $effect(() => {
    const next = searchInput;
    const timer = setTimeout(() => {
      searchQuery = next;
    }, 250);
    return () => clearTimeout(timer);
  });

  const monthlyBurn = $derived.by(() => {
    const today = new SvelteDate();
    const d30 = new SvelteDate(today);
    d30.setDate(d30.getDate() - 30);
    const d30Str = todayString(d30);
    let exp = 0;
    for (const t of transactions) {
      if (t.date >= d30Str) {
        for (const s of t.splits) {
          const acc = accountsById.get(s.accountId);
          if (acc?.account_type === 'EXPENSE') {
            exp += s.amount;
          }
        }
      }
    }
    return exp;
  });

  const runwayMonths = $derived.by(() => {
    if (assets <= 0) return 0;
    if (monthlyBurn <= 0) return 999;
    return Math.round((assets / monthlyBurn) * 10) / 10;
  });
  const quickRatio = $derived.by(() => {
    if (liabilities <= 0) return 999;
    return Math.round((assets / liabilities) * 100) / 100;
  });
</script>

<PageLayout title={i18n.t.dashboard}>
  {#snippet actions()}
    <SearchBar
      bind:value={searchInput}
      placeholder={searchScope === 'ALL'
        ? i18n.t.searchAllPlaceholder
        : searchScope === 'ACCOUNT'
          ? i18n.t.searchAccountPlaceholder
          : i18n.t.searchRecentPlaceholder}
      scope={{
        value: searchScope,
        options: [
          { value: 'ALL', label: i18n.t.searchScopeAll },
          { value: 'ACCOUNT', label: i18n.t.searchScopeAccount },
          { value: 'RECENT', label: i18n.t.searchScopeRecent },
        ],
        onchange: (v) => (searchScope = v as typeof searchScope),
      }}
    />
    <div class="flex shrink-0 items-center gap-1.5">
      <Button
        variant="primary"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        title={i18n.t.newTransaction}
        ariaLabel={i18n.t.newTransaction}
        onclick={() => modalState.openQuickTx()}
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

  {#if loading}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if error}
    <ErrorState message={error} onRetry={loadDashboard} />
  {:else}
    <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
      <SummaryCards
        {assets}
        {usdAssets}
        accountCount={visibleAccounts.length}
        {liabilities}
        {debtRatio}
        {netWorth}
        unbalanced={unbalancedCount}
        uncategorized={uncategorizedCount}
        pending={pendingCount}
        lastRecon={lastReconText}
        {runwayMonths}
        {quickRatio}
      />

      <div class="grid min-h-0 flex-1 grid-cols-12 grid-rows-1 gap-2 overflow-hidden">
        <AccountList {searchQuery} {searchScope} accounts={accountItems} />
        <RecentTransactions
          {searchQuery}
          {searchScope}
          {transactions}
          {accountsById}
          onNewTx={() => modalState.openQuickTx()}
        />
      </div>

      <CashflowCard {transactions} {accountsById} />
    </div>
  {/if}

  <TransferModal bind:open={transferOpen} onSuccess={loadDashboard} />
</PageLayout>
