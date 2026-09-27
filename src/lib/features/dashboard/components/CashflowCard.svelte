<script lang="ts">
  import type {
    Account,
    LedgerTotalsView,
    JournalEntryView,
    DashboardMetricsView,
  } from '$lib/core/ipc/bindings';
  import { fxState } from '$lib/core/state/fx.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Tabs, Card, AnimatedCounter } from '$lib/components/ui';

  let {
    entries = [],
    transactions = [],
    accountsById = new Map(),
    totals = null,
    metrics = null,
  }: {
    entries?: JournalEntryView[];
    transactions?: JournalEntryView[];
    accountsById?: Map<string, Account>;
    totals?: LedgerTotalsView | null;
    metrics?: DashboardMetricsView | null;
  } = $props();

  const allEntries = $derived(entries.length > 0 ? entries : transactions);

  let cashflowPeriod = $state<'month' | 'all'>('month');

  function computePnl(entryList: JournalEntryView[], fromDate?: string, toDate?: string) {
    const fxRate = fxState.rate;
    let income = 0;
    let expense = 0;
    for (const t of entryList) {
      if (fromDate && t.date < fromDate) continue;
      if (toDate && t.date > toDate) continue;
      for (const s of t.postings) {
        const accType = s.account_type || accountsById.get(s.account_id)?.account_type;
        if (!accType) continue;
        const cur = t.currency || 'IDR';
        const idrAmount = cur === 'USD' ? Math.round((s.amount / 100) * fxRate) : s.amount;
        if (accType === 'INCOME') {
          income += -idrAmount;
        } else if (accType === 'EXPENSE') {
          expense += idrAmount;
        }
      }
    }
    return { income, expense, net: income - expense };
  }

  const activePnl = $derived.by(() => {
    if (cashflowPeriod === 'all' && totals) {
      return {
        income: totals.total_income,
        expense: totals.total_expenses,
        net: totals.net_income,
      };
    }
    if (metrics) {
      return {
        income: metrics.this_month_income,
        expense: metrics.this_month_expense,
        net: metrics.this_month_net,
      };
    }
    const now = new Date();
    const currentMonthStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
    return computePnl(allEntries, currentMonthStart, undefined);
  });

  const savingsRate = $derived(
    activePnl.income > 0 ? Math.max(0, (activePnl.net / activePnl.income) * 100).toFixed(1) : '0.0'
  );

  const incomeRatio = $derived(
    activePnl.income + activePnl.expense > 0
      ? Math.round((activePnl.income / (activePnl.income + activePnl.expense)) * 100)
      : 50
  );

  const expenseRatio = $derived(100 - incomeRatio);

  const netIsPositive = $derived(activePnl.net >= 0);
</script>

<Card title={i18n.t.pnlTitle}>
  {#snippet header()}
    <div class="flex items-center gap-2">
      <Tabs
        variant="outline"
        tabs={[
          { id: 'month', label: i18n.t.thisMonth },
          { id: 'all', label: i18n.t.allTime },
        ]}
        active={cashflowPeriod}
        onSelect={(id) => (cashflowPeriod = id as 'month' | 'all')}
      />
    </div>
  {/snippet}

  <div class="flex flex-col gap-2.5 pt-1">
    <div class="grid w-full grid-cols-2 gap-4 sm:grid-cols-4">
      <div class="min-w-0">
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.income}
        </p>
        <AnimatedCounter
          value={activePnl.income}
          currency="IDR"
          class="text-income text-medium mt-1 block truncate font-bold"
        />
      </div>
      <div class="min-w-0">
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.expense}
        </p>
        <AnimatedCounter
          value={activePnl.expense}
          currency="IDR"
          class="text-expense text-medium mt-1 block truncate font-bold"
        />
      </div>
      <div class="min-w-0">
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.net}
        </p>
        <AnimatedCounter
          value={activePnl.net}
          currency="IDR"
          class="{netIsPositive
            ? 'text-income'
            : 'text-expense'} text-medium mt-1 block truncate font-bold"
        />
      </div>
      <div class="min-w-0">
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.savingsRate}
        </p>
        <p class="text-teal font-proto text-medium mt-1 font-bold tabular-nums">{savingsRate}%</p>
      </div>
    </div>

    {#if activePnl.income > 0 || activePnl.expense > 0}
      <div class="flex h-1.5 w-full overflow-hidden">
        <div
          class="bg-income h-full transition-all duration-500"
          style="width: {incomeRatio}%"
          title="{i18n.t.income}: {incomeRatio}%"
        ></div>
        <div
          class="bg-expense h-full transition-all duration-500"
          style="width: {expenseRatio}%"
          title="{i18n.t.expense}: {expenseRatio}%"
        ></div>
      </div>
    {/if}
  </div>
</Card>
