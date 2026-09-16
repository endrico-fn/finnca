<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { reportState } from '$lib/features/report/state/report.svelte';
  import { onMount, untrack } from 'svelte';
  import { PageLayout, Button, Icon } from '$lib/components/ui';
  import { createTabRouter } from '$lib/core/router/tabRouter.svelte';
  import { APP_NAME } from '$lib/core/types';
  import ExportOverlay from './ExportOverlay.svelte';
  import { exportReportPDF, exportReportCSV } from '../export';
  import {
    computeTrendsDateRange,
    type TrendsPeriod,
    type TrendsMetric,
  } from '../state/trendsChartUtils';
  import ReportTabBar, { type ReportTab } from './ReportTabBar.svelte';

  import TrialBalance from './TrialBalance.svelte';
  import DebtReport from './DebtReport.svelte';
  import Trends from './Trends.svelte';
  import Cashflow from './Cashflow.svelte';
  import FxReport from './FxReport.svelte';

  const validTabs = ['tb', 'trends', 'debt', 'cashflow', 'fx'] as const;

  const tabRouter = createTabRouter<ReportTab>('tb', validTabs, 'tab');

  let from = $state('');
  let to = $state('');
  let exportOpen = $state(false);

  const currentTabLabel = $derived.by(() => {
    switch (tabRouter.current) {
      case 'tb':
        return i18n.t.trialBalanceTitle;
      case 'trends':
        return i18n.t.trendsTitle;
      case 'debt':
        return i18n.t.debtReportTitle;
      case 'cashflow':
        return i18n.t.cashflowTab;
      case 'fx':
        return i18n.t.fxRevalTab;
      default:
        return i18n.t.report;
    }
  });

  let trendsPeriod = $state<TrendsPeriod>('1M');
  let trendsMetric = $state<TrendsMetric>('netWorth');

  const trendsDateRange = $derived(computeTrendsDateRange(trendsPeriod));

  onMount(async () => {
    await Promise.all([
      reportState.loadBalanceSheet(),
      reportState.loadProfitLoss(),
      reportState.loadTrialBalance(),
      reportState.loadHistoricalTrends(trendsDateRange.startStr, trendsDateRange.endStr),
      reportState.loadFxRevaluation(),
    ]);
  });

  $effect(() => {
    const { startStr, endStr } = trendsDateRange;
    untrack(() => {
      reportState.loadHistoricalTrends(startStr, endStr);
    });
  });

  const bs = $derived({
    assets: reportState.balanceSheet?.total_assets ?? 0,
    liabilities: reportState.balanceSheet?.total_liabilities ?? 0,
    equity: reportState.balanceSheet?.total_equity ?? 0,
    netIncome: reportState.balanceSheet?.net_income ?? 0,
    balanced: reportState.balanceSheet?.is_balanced ?? false,
  });

  const monthlyBurn = $derived.by(() => {
    const pnl = reportState.profitLoss;
    return pnl && pnl.total_expenses > 0 ? pnl.total_expenses : 0;
  });

  const historicalPoints = $derived(reportState.historicalPoints);

  async function exportCSV() {
    const defaultFilename = `finnca_${tabRouter.current}_${todayString()}.csv`;
    const savedPath = await exportReportCSV(
      tabRouter.current,
      historicalPoints,
      reportState.trialBalance,
      defaultFilename,
      i18n.t.dialogCsvFilter
    );

    if (savedPath) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.exportSuccess,
        message: savedPath,
      });
    }
  }

  async function exportPDF(rangeFrom: string, rangeTo: string) {
    const vaultName = session.currentVault || APP_NAME;
    const periodLabel = rangeFrom && rangeTo ? `${rangeFrom} → ${rangeTo}` : i18n.t.allTime;
    await exportReportPDF({
      vaultName,
      reportTitle: currentTabLabel,
      period: periodLabel,
      tab: tabRouter.current,
      trialBalance: reportState.trialBalance,
      profitLoss: reportState.profitLoss,
      balanceSheet: reportState.balanceSheet,
    });
    notificationState.addNotification({
      type: 'LEDGER_INTEGRITY',
      priority: 'low',
      title: i18n.t.exportSuccess,
      message: `PDF — ${currentTabLabel}`,
    });
  }

  async function handleExport(format: 'pdf' | 'csv', rangeFrom: string, rangeTo: string) {
    if (format === 'csv') {
      await exportCSV();
    } else {
      await exportPDF(rangeFrom, rangeTo);
    }
  }
</script>

<PageLayout crumb={i18n.t.report} crumbHref="/app/reports" title={currentTabLabel}>
  {#snippet actions()}
    <Button variant="ghost" onclick={() => (exportOpen = true)} class="flex items-center gap-1.5">
      {i18n.t.exportReportBtn}
      <Icon name="export" size={14} />
    </Button>
  {/snippet}

  <ExportOverlay
    bind:open={exportOpen}
    reportTitle={currentTabLabel}
    supportsPDF={tabRouter.current !== 'trends'}
    onExport={handleExport}
  />

  <ReportTabBar
    currentTab={tabRouter.current}
    onSelectTab={(tab) => tabRouter.setTab(tab)}
    bind:from
    bind:to
  />

  <div class="-mr-3 flex min-h-0 flex-1 overflow-hidden">
    {#if tabRouter.current === 'tb'}
      <div class="flex min-h-0 flex-1 pr-3">
        <TrialBalance asOf={to} />
      </div>
    {:else if tabRouter.current === 'trends'}
      <Trends
        {bs}
        monthlyExpense={monthlyBurn > 0 ? monthlyBurn : 1}
        bind:trendsPeriod
        bind:trendsMetric
        {trendsDateRange}
        {historicalPoints}
      />
    {:else if tabRouter.current === 'debt'}
      <div class="flex min-h-0 flex-1 pr-3">
        <DebtReport asOf={to} />
      </div>
    {:else if tabRouter.current === 'cashflow'}
      <div class="flex min-h-0 flex-1 pr-3">
        <Cashflow {from} {to} />
      </div>
    {:else if tabRouter.current === 'fx'}
      <div class="flex min-h-0 flex-1 pr-3">
        <FxReport />
      </div>
    {/if}
  </div>
</PageLayout>
