<script module lang="ts">
  import type {
    AccountBalanceView,
    DashboardMetricsView,
    JournalEntryView,
    LedgerTotalsView,
    DueRecurringPlanView,
  } from '$lib/core/ipc/bindings';

  interface DashboardSnapshot {
    vault: string;
    accountItems: AccountBalanceView[];
    entries: JournalEntryView[];
    totals: LedgerTotalsView | null;
    metrics: DashboardMetricsView | null;
    duePlans: DueRecurringPlanView[];
  }

  let snapshot: DashboardSnapshot | null = null;
</script>

<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import {
    listAccountsCmd,
    listJournalEntriesCmd,
    getLedgerTotalsCmd,
    getDashboardMetricsCmd,
    getDueRecurringPlansCmd,
    postDueRecurringBatchCmd,
  } from '$lib/core/ipc/bindings';
  import { fxState } from '$lib/core/state/fx.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { ErrorState, PageLayout, Button, SearchBar } from '$lib/components/ui';

  import { todayString } from '$lib/core/format/date';

  import TacticalMetricRibbon from './TacticalMetricRibbon.svelte';
  import CashflowCard from './CashflowCard.svelte';
  import AccountList from './AccountList.svelte';
  import RecentTransactions from './RecentTransactions.svelte';
  import DashboardSkeleton from './DashboardSkeleton.svelte';

  const vaultKey = $derived(session.vaultPath ?? '');

  let loading = $state(true);
  let error = $state<string | null>(null);
  let accountItems = $state<DashboardSnapshot['accountItems']>([]);
  let entries = $state<DashboardSnapshot['entries']>([]);
  let totals = $state<DashboardSnapshot['totals'] | null>(null);
  let metrics = $state<DashboardSnapshot['metrics'] | null>(null);
  let duePlans = $state<DashboardSnapshot['duePlans']>([]);
  let postingDue = $state(false);

  async function handlePostDueDirectly() {
    const dueItems = duePlans.filter((p) => p.days_diff <= 0);
    if (dueItems.length === 0 || postingDue) return;
    postingDue = true;
    try {
      const planIds = dueItems.map((p) => p.plan_id);
      await postDueRecurringBatchCmd({
        plan_ids: planIds,
        date: todayString(),
      });
      notificationState.addNotification({
        type: 'INFO',
        title: i18n.t.recurringRunnerTitle,
        message: i18n.t.recurringBatchSuccess.replace('{count}', String(dueItems.length)),
        priority: 'medium',
      });
      eventBus.emit('transaction:posted', { id: '' });
      await loadDashboard(true);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      postingDue = false;
    }
  }

  async function loadDashboard(background = false) {
    if (!background) {
      loading = true;
    }
    error = null;
    try {
      const fxRate = fxState.rate;
      const [entryList, tot, items, met, due] = await Promise.all([
        listJournalEntriesCmd(100),
        getLedgerTotalsCmd(fxRate),
        listAccountsCmd(),
        getDashboardMetricsCmd(fxRate, todayString()),
        getDueRecurringPlansCmd(todayString()).catch(() => []),
      ]);
      entries = entryList;
      totals = tot;
      accountItems = items;
      metrics = met;
      duePlans = due;
      snapshot = { vault: vaultKey, accountItems, entries, totals, metrics, duePlans };
    } catch (e) {
      if (!background || entries.length === 0) {
        error = e instanceof Error ? e.message : String(e);
      }
    } finally {
      if (!background) {
        loading = false;
      }
    }
  }

  onMount(() => {
    if (snapshot && snapshot.vault === vaultKey) {
      accountItems = snapshot.accountItems;
      entries = snapshot.entries;
      totals = snapshot.totals;
      metrics = snapshot.metrics;
      duePlans = snapshot.duePlans;
      loading = false;
      void loadDashboard(true);
    } else {
      void loadDashboard(false);
    }
    const refresh = () => void loadDashboard(entries.length > 0 || totals !== null);
    const unsubTx = eventBus.on('transaction:posted', refresh);
    const unsubAcc = eventBus.on('accounts:changed', refresh);
    return () => {
      unsubTx();
      unsubAcc();
    };
  });

  const assets = $derived(totals?.total_assets ?? 0);
  const liabilities = $derived(totals?.total_liabilities ?? 0);
  const netWorth = $derived(assets - liabilities);
  const debtRatio = $derived(assets > 0 ? ((liabilities / assets) * 100).toFixed(1) : '0.0');
  const fxRate = $derived(fxState.rate);
  const usdAssets = $derived(fxRate > 0 ? Math.round((assets / fxRate) * 100) : 0);
  const thisMonthIncome = $derived(metrics?.this_month_income ?? totals?.total_income ?? 0);
  const thisMonthNet = $derived(metrics?.this_month_net ?? totals?.net_income ?? 0);
  const activeSavingsRate = $derived(
    thisMonthIncome > 0 ? Math.max(0, (thisMonthNet / thisMonthIncome) * 100).toFixed(1) : '0.0'
  );

  const accounts = $derived(accountItems.map((i) => i.account));
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));

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
        ariaLabel={i18n.t.newEntry}
        onclick={() => modalState.openInspector({ mode: 'journal' })}
      >
        {i18n.t.newEntry}
      </Button>
      <Button
        variant="ghost"
        class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
        ariaLabel={i18n.t.transfersTitle}
        onclick={() => modalState.openInspector({ mode: 'transfer' })}
      >
        {i18n.t.transfersTitle}
      </Button>
    </div>
  {/snippet}

  {#if loading}
    <DashboardSkeleton />
  {:else if error}
    <ErrorState message={error} onRetry={() => loadDashboard(false)} />
  {:else}
    <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
      <!-- Alerts banner if any -->
      {#if (metrics?.overdue_invoices_count ?? 0) > 0 || (metrics?.upcoming_due_count ?? 0) > 0 || duePlans.length > 0}
        <div class="flex shrink-0 flex-wrap items-center gap-2">
          {#if (metrics?.overdue_invoices_count ?? 0) > 0 || (metrics?.upcoming_due_count ?? 0) > 0}
            <div
              class="border-line bg-bg-card flex flex-1 items-center justify-between border px-3 py-1.5"
            >
              <div class="flex items-center gap-2">
                <span
                  class="font-proto text-smaller text-text-dim font-bold tracking-wider uppercase"
                >
                  {i18n.t.widgetUpcomingTitle}
                </span>
                {#if (metrics?.overdue_invoices_count ?? 0) > 0}
                  <button
                    type="button"
                    class="badge-err font-proto text-smaller flex items-center gap-1.5 px-2 py-0.5 transition-opacity hover:opacity-80"
                    onclick={() => goto(resolve('/app/journal?filter=overdue' as '/app/journal'))}
                  >
                    <span class="font-bold">{metrics?.overdue_invoices_count}</span>
                    <span>{i18n.t.badgeOverdue}</span>
                  </button>
                {/if}
                {#if (metrics?.upcoming_due_count ?? 0) > 0}
                  <button
                    type="button"
                    class="badge-warn font-proto text-smaller flex items-center gap-1.5 px-2 py-0.5 transition-opacity hover:opacity-80"
                    onclick={() => goto(resolve('/app/journal?filter=due' as '/app/journal'))}
                  >
                    <span class="font-bold">{metrics?.upcoming_due_count}</span>
                    <span>{i18n.t.badgeDue}</span>
                  </button>
                {/if}
              </div>
              <button
                type="button"
                class="text-teal font-proto text-smaller tracking-wider uppercase hover:underline"
                onclick={() => goto(resolve('/app/journal?filter=due' as '/app/journal'))}
              >
                &rarr;
              </button>
            </div>
          {/if}

          {#if duePlans.length > 0}
            {@const dueCount = duePlans.filter((p) => p.days_diff <= 0).length}
            <div
              class="border-line bg-bg-card flex flex-1 items-center justify-between border px-3 py-1.5"
            >
              <div class="flex items-center gap-2">
                <span class="font-proto text-smaller text-teal font-bold tracking-wider uppercase">
                  {i18n.t.recurringDueBannerTitle}
                </span>
                {#if dueCount > 0}
                  <button
                    type="button"
                    class="badge-warn font-proto text-smaller flex items-center gap-1.5 px-2 py-0.5 transition-opacity hover:opacity-80"
                    onclick={() => goto(resolve('/app/plan'))}
                  >
                    <span class="font-bold">{dueCount}</span>
                    <span>{i18n.t.recurringDueToday}</span>
                  </button>
                  <Button
                    variant="primary"
                    size="sm"
                    loading={postingDue}
                    class="font-proto text-smaller px-2 font-bold tracking-wider uppercase"
                    onclick={handlePostDueDirectly}
                  >
                    {i18n.t.recurringRecordNow}
                  </Button>
                {/if}
              </div>
              <button
                type="button"
                class="text-teal font-proto text-smaller tracking-wider uppercase hover:underline"
                onclick={() => goto(resolve('/app/plan'))}
              >
                {i18n.t.recurringReviewDetail} &rarr;
              </button>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Top Row: Compact Tactical Metric Ribbon -->
      <TacticalMetricRibbon
        {netWorth}
        {assets}
        {usdAssets}
        {liabilities}
        {debtRatio}
        {thisMonthNet}
        savingsRate={activeSavingsRate}
      />

      <!-- Middle Row: Workstation with AccountList and RecentTransactions side by side -->
      <div class="grid min-h-0 flex-1 grid-cols-12 gap-2 overflow-hidden">
        <AccountList
          {searchQuery}
          {searchScope}
          accounts={accountItems}
          class="col-span-12 h-full lg:col-span-8"
        />
        <RecentTransactions
          {searchQuery}
          {searchScope}
          {entries}
          {accountsById}
          onNewTx={() => modalState.openInspector({ mode: 'journal' })}
          class="col-span-12 h-full lg:col-span-4"
        />
      </div>

      <!-- Bottom Row: Cashflow Card spanning full width -->
      <div class="shrink-0">
        <CashflowCard {entries} {accountsById} {totals} {metrics} />
      </div>
    </div>
  {/if}
</PageLayout>
