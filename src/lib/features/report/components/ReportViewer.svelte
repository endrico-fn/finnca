<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { reportState, type ReportTab } from '$lib/features/report/state/report.svelte';
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
  import ReportTabBar from './ReportTabBar.svelte';

  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import BalanceSheet from './BalanceSheet.svelte';
  import ProfitLoss from './ProfitLoss.svelte';
  import BudgetVsActual from './BudgetVsActual.svelte';
  import TrialBalance from './TrialBalance.svelte';
  import DebtReport from './DebtReport.svelte';
  import Trends from './Trends.svelte';
  import Cashflow from './Cashflow.svelte';
  import FxReport from './FxReport.svelte';

  const validTabs = [
    'bs',
    'pnl',
    'cashflow',
    'tb',
    'budget-actual',
    'debt',
    'trends',
    'fx',
  ] as const;

  const tabRouter = createTabRouter<ReportTab>('bs', validTabs, 'tab');

  let from = $state('');
  let to = $state('');
  let exportOpen = $state(false);

  const currentTabLabel = $derived.by(() => {
    switch (tabRouter.current) {
      case 'bs':
        return i18n.t.balanceSheetTitle;
      case 'pnl':
        return i18n.t.pnlTitle;
      case 'cashflow':
        return i18n.t.cashflowTab;
      case 'tb':
        return i18n.t.trialBalanceTitle;
      case 'budget-actual':
        return i18n.t.budgetVsActualTitle;
      case 'debt':
        return i18n.t.debtReportTitle;
      case 'trends':
        return i18n.t.trendsTitle;
      case 'fx':
        return i18n.t.fxRevalTab;
      default:
        return i18n.t.report;
    }
  });

  let trendsPeriod = $state<TrendsPeriod>('1M');
  let trendsMetric = $state<TrendsMetric>('netWorth');

  const trendsDateRange = $derived(computeTrendsDateRange(trendsPeriod));

  onMount(() => {
    reportState.loadBalanceSheet();
    reportState.loadProfitLoss();
    reportState.loadTrialBalance();
    reportState.loadHistoricalTrends(trendsDateRange.startStr, trendsDateRange.endStr);
    reportState.loadFxRevaluation();

    const unlistenFx = eventBus.on('fx:rate_changed', ({ rate }) => {
      reportState.loadFxRevaluation(undefined, rate);
      reportState.loadHistoricalTrends(trendsDateRange.startStr, trendsDateRange.endStr, rate);
    });

    return () => {
      unlistenFx();
    };
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
      i18n.t.dialogCsvFilter,
      reportState.balanceSheet,
      reportState.profitLoss
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

  <div class="flex min-h-0 flex-1 flex-col overflow-y-auto w-full">
    {#if tabRouter.current === 'bs'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <BalanceSheet asOf={to} />
      </div>
    {:else if tabRouter.current === 'pnl'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <ProfitLoss {from} {to} />
      </div>
    {:else if tabRouter.current === 'cashflow'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <Cashflow {from} {to} />
      </div>
    {:else if tabRouter.current === 'tb'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <TrialBalance asOf={to} />
      </div>
    {:else if tabRouter.current === 'budget-actual'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <BudgetVsActual />
      </div>
    {:else if tabRouter.current === 'debt'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <DebtReport asOf={to} />
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
    {:else if tabRouter.current === 'fx'}
      <div class="flex min-h-0 flex-1 flex-col w-full">
        <FxReport />
      </div>
    {/if}
  </div>
</PageLayout>
