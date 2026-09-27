<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { reportState, type ReportTab } from '$lib/features/report/state/report.svelte';
  import { onMount, untrack } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { PageLayout, Button, Icon } from '$lib/components/ui';
  import { createTabRouter } from '$lib/core/router/tabRouter.svelte';
  import { APP_NAME, APP_SLUG } from '$lib/core/types';
  import ExportOverlay from './ExportOverlay.svelte';
  import { exportReportPDF } from '../exportPdf';
  import { exportReportCSV } from '../exportCsv';
  import {
    computeTrendsDateRange,
    type TrendsPeriod,
    type TrendsMetric,
  } from '../state/trendsChartUtils';
  import ReportTabBar from './ReportTabBar.svelte';
  import { REPORT_GROUPS, reportGroupOf } from '../state/reportNav';

  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import BalanceSheet from './BalanceSheet.svelte';
  import ProfitLoss from './ProfitLoss.svelte';
  import TrialBalance from './TrialBalance.svelte';
  import DebtReport from './DebtReport.svelte';
  import Trends from './Trends.svelte';
  import FxReport from './FxReport.svelte';
  import SpendingAnalysis from './SpendingAnalysis.svelte';
  import CashFlow from './CashFlow.svelte';
  import CashForecast from './CashForecast.svelte';
  import NetWorthProgression from './NetWorthProgression.svelte';
  import IncomeExpenseChart from './IncomeExpenseChart.svelte';

  const validTabs = [
    'networth',
    'income-exp',
    'bs',
    'pnl',
    'cashflow',
    'tb',
    'debt',
    'trends',
    'spending',
    'forecast',
    'fx',
  ] as const;

  const tabRouter = createTabRouter<ReportTab>('tb', validTabs, 'tab');

  let from = $state('');
  let to = $state('');
  let exportOpen = $state(false);

  const activeGroupMeta = $derived(
    REPORT_GROUPS.find((g) => g.id === reportGroupOf(tabRouter.current)) ?? REPORT_GROUPS[0]
  );

  const currentTabLabel = $derived.by(() => {
    switch (tabRouter.current) {
      case 'networth':
        return i18n.t.networthProgressionTitle;
      case 'income-exp':
        return i18n.t.incomeExpChartTitle;
      case 'bs':
        return i18n.t.balanceSheetTitle;
      case 'pnl':
        return i18n.t.pnlTitle;
      case 'cashflow':
        return i18n.t.cashFlowTitle;
      case 'tb':
        return i18n.t.trialBalanceTitle;
      case 'debt':
        return i18n.t.debtReportTitle;
      case 'trends':
        return i18n.t.trendsTitle;
      case 'spending':
        return i18n.t.spendingTitle;
      case 'forecast':
        return i18n.t.cashForecastTitle;
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
    const defaultFilename = `${APP_SLUG}_${tabRouter.current}_${todayString()}.csv`;
    const savedPath = await exportReportCSV(
      tabRouter.current,
      historicalPoints,
      reportState.trialBalance,
      defaultFilename,
      i18n.t.dialogCsvFilter,
      reportState.balanceSheet,
      reportState.profitLoss,
      reportState.cashFlow
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
      cashFlow: reportState.cashFlow,
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

<PageLayout
  parentCrumb={i18n.t.report}
  parentHref="/app/reports"
  crumb={i18n.t[activeGroupMeta.labelKey]}
  crumbAction={() => tabRouter.setTab(activeGroupMeta.defaultTab)}
  title={currentTabLabel}
>
  {#snippet actions()}
    <Button
      variant="ghost"
      onclick={() => (exportOpen = true)}
      class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap flex items-center gap-1.5"
    >
      {i18n.t.exportReportBtn}
      <Icon name="export" size={14} />
    </Button>
  {/snippet}

  <ExportOverlay
    bind:open={exportOpen}
    reportTitle={currentTabLabel}
    supportsPDF={tabRouter.current !== 'trends' && tabRouter.current !== 'income-exp'}
    onExport={handleExport}
  />

  <ReportTabBar
    currentTab={tabRouter.current}
    onSelectTab={(tab) => tabRouter.setTab(tab)}
    bind:from
    bind:to
  />

  <div class="flex min-h-0 w-full flex-1 flex-col overflow-y-auto">
    {#key tabRouter.current}
      <div
        class="flex min-h-0 flex-1 flex-col"
        in:fly={{ y: 4, duration: 140, easing: cubicOut }}
        out:fade={{ duration: 60 }}
      >
        {#if tabRouter.current === 'networth'}
          <NetWorthProgression
            {bs}
            monthlyExpense={monthlyBurn > 0 ? monthlyBurn : 1}
            {historicalPoints}
            {trendsDateRange}
            bind:trendsPeriod
          />
        {:else if tabRouter.current === 'income-exp'}
          <IncomeExpenseChart {from} {to} />
        {:else if tabRouter.current === 'bs'}
          <BalanceSheet asOf={to} />
        {:else if tabRouter.current === 'pnl'}
          <ProfitLoss {from} {to} />
        {:else if tabRouter.current === 'cashflow'}
          <CashFlow {from} {to} />
        {:else if tabRouter.current === 'tb'}
          <TrialBalance asOf={to} />
        {:else if tabRouter.current === 'debt'}
          <DebtReport asOf={to} />
        {:else if tabRouter.current === 'trends'}
          <Trends
            {bs}
            monthlyExpense={monthlyBurn > 0 ? monthlyBurn : 1}
            bind:trendsPeriod
            bind:trendsMetric
            {trendsDateRange}
            {historicalPoints}
          />
        {:else if tabRouter.current === 'spending'}
          <SpendingAnalysis {from} {to} />
        {:else if tabRouter.current === 'forecast'}
          <CashForecast />
        {:else if tabRouter.current === 'fx'}
          <FxReport />
        {/if}
      </div>
    {/key}
  </div>
</PageLayout>
