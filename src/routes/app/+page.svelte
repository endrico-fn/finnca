<script lang="ts">
  import { onMount } from 'svelte';
  import { ledger } from '$lib/accounting/store.svelte';
  import {
    balanceSheet,
    incomeStatement,
    fromMinor,
    buildChildrenMap,
    isBalanced as txBalanced,
  } from '$lib/accounting/finance';
  import { DEFAULT_FX_RATE } from '$lib/accounting/types';
  import { i18n } from '$lib/i18n.svelte';
  import TransferModal from '$lib/components/TransferModal.svelte';
  import {
    Splash,
    ErrorState,
    PageLayout,
    Button,
    SearchBar,
    Icon,
  } from '$lib/components/ui';
  import BalanceCard from './dashboard/_widgets/BalanceCard.svelte';
  import DebtCard from './dashboard/_widgets/DebtCard.svelte';
  import HealthCard from './dashboard/_widgets/HealthCard.svelte';
  import CashflowCard from './dashboard/_widgets/CashflowCard.svelte';
  import AccountList from './dashboard/_widgets/AccountList.svelte';
  import RecentTransactions from './dashboard/_widgets/RecentTransactions.svelte';

  onMount(() => {
    if (!ledger.data) ledger.load();
  });

  const bs = $derived(
    ledger.data
      ? balanceSheet(ledger.data, undefined, ledger.childrenMap)
      : { assets: 0, liabilities: 0, equity: 0, netIncome: 0, balanced: false }
  );

  const netWorth = $derived(bs.assets - bs.liabilities);
  const debtRatio = $derived(
    bs.assets > 0 ? ((bs.liabilities / bs.assets) * 100).toFixed(1) : '0.0'
  );
  const fxRate = $derived(ledger.data?.fxRate ?? DEFAULT_FX_RATE);
  const usdAssets = $derived(
    (bs.assets / fxRate).toLocaleString('en-US', { maximumFractionDigits: 0 })
  );

  // Real ledger health checks
  const unbalancedCount = $derived(ledger.transactions.filter((tx) => !txBalanced(tx)).length);
  const uncategorizedCount = $derived(
    ledger.transactions.filter((t) =>
      t.splits.some((s) => {
        const acc = ledger.accountsById.get(s.accountId);
        if (!acc || acc.placeholder) return true;
        const name = acc.name.toLowerCase();
        return (
          name.includes('uncategorized') || name.includes('imbalance') || name.includes('unknown')
        );
      })
    ).length
  );
  const pendingCount = $derived(
    ledger.transactions.filter((t) => t.splits.some((s) => s.reconcile === 'c')).length
  );

  const reconciledDates = $derived(
    ledger.transactions
      .filter((t) => t.splits.some((s) => s.reconcile === 'y'))
      .map((t) => t.date)
      .sort((a, b) => b.localeCompare(a))
  );
  const lastReconText = $derived(() => {
    if (reconciledDates.length === 0) {
      return ledger.transactions.length === 0 ? i18n.t.reconNever : i18n.t.reconPending;
    }
    const days = Math.max(
      0,
      Math.floor((Date.now() - new Date(reconciledDates[0]).getTime()) / 86_400_000)
    );
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

  const fmt = (n: number) => n.toLocaleString('en-US');
  const childrenMap = $derived(ledger.data ? buildChildrenMap(ledger.data.accounts) : new Map());
  const leafAccounts = $derived(ledger.accounts.filter((a) => !a.placeholder && !a.hidden));

  const pnlCurrent = $derived(
    ledger.data ? incomeStatement(ledger.data) : { income: 0, expense: 0, net: 0 }
  );
  const runwayMonths = $derived.by(() => {
    if (!bs || bs.assets <= 0) return 0;
    if (pnlCurrent.expense <= 0) return 999;
    return Math.round((bs.assets / pnlCurrent.expense) * 10) / 10;
  });
  const quickRatio = $derived.by(() => {
    if (!bs || bs.liabilities <= 0) return 999;
    return Math.round((bs.assets / bs.liabilities) * 100) / 100;
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
          { value: 'ALL', label: 'ALL' },
          { value: 'ACCOUNT', label: 'ACCOUNT' },
          { value: 'RECENT', label: 'RECENT' },
        ],
        onchange: (v) => (searchScope = v as typeof searchScope),
      }}
    />
    <div class="flex shrink-0 items-center gap-2">
      <Button variant="ghost" onclick={() => (transferOpen = true)}>
        <span class="text-income inline-flex items-center gap-1.5 whitespace-nowrap">
          <Icon name="transfer" size={12} />
          {i18n.t.transferTitle}
        </span>
      </Button>
      <Button variant="primary" href="/app/journal">
        <span class="inline-flex items-center gap-1.5 whitespace-nowrap">
          <Icon name="plus" size={12} />
          {i18n.t.newTransaction}
        </span>
      </Button>
    </div>
  {/snippet}

  {#if ledger.loading}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if ledger.error}
    <ErrorState message={ledger.error} onRetry={() => ledger.load()} />
  {:else}
    <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
      <div class="grid shrink-0 grid-cols-12 gap-2">
        <div class="col-span-4">
          <BalanceCard
            assets={fmt(fromMinor('IDR', bs.assets))}
            {usdAssets}
            accountCount={leafAccounts.length}
          />
        </div>

        <div class="col-span-4">
          <DebtCard
            liabilities={fmt(fromMinor('IDR', bs.liabilities))}
            {debtRatio}
            netWorth={fmt(fromMinor('IDR', netWorth))}
          />
        </div>

        <div class="col-span-4">
          <HealthCard
            unbalanced={unbalancedCount}
            uncategorized={uncategorizedCount}
            pending={pendingCount}
            lastRecon={lastReconText()}
            {runwayMonths}
            {quickRatio}
          />
        </div>
      </div>

      <div class="grid min-h-0 flex-1 grid-cols-12 grid-rows-1 gap-2 overflow-hidden">
        <AccountList {searchQuery} {searchScope} {childrenMap} />
        <RecentTransactions {searchQuery} {searchScope} />
      </div>

      <CashflowCard />
    </div>
  {/if}

  <TransferModal bind:open={transferOpen} />
</PageLayout>
