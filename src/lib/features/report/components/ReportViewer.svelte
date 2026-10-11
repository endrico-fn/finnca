<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { todayString } from '$lib/core/format/date';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { session } from '$lib/core/state/session.svelte';
  import { reportState, type ReportTab } from '$lib/features/report/state/report.svelte';
  import { onMount, untrack } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { PageLayout, Button, Icon, DateRangeDropdown, DateDropdown } from '$lib/components/ui';
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
  import ReportDropdownTabs from './ReportDropdownTabs.svelte';
  import { REPORT_TAB_LABELS } from '../state/reportNav';

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

  const currentTabLabel = $derived.by(() => {
    const labelKey = REPORT_TAB_LABELS[tabRouter.current];
    return labelKey ? i18n.t[labelKey] : i18n.t.report;
  });

  const isRangeTab = $derived(
    tabRouter.current === 'pnl' ||
      tabRouter.current === 'cashflow' ||
      tabRouter.current === 'spending' ||
      tabRouter.current === 'fx' ||
      tabRouter.current === 'income-exp' ||
      tabRouter.current === 'trends' ||
      tabRouter.current === 'networth'
  );
  const isAsOfTab = $derived(
    tabRouter.current === 'bs' ||
      tabRouter.current === 'tb' ||
      tabRouter.current === 'debt' ||
      tabRouter.current === 'forecast'
  );

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

<PageLayout crumb={i18n.t.report} crumbHref="/app/reports" title={currentTabLabel}>
  {#snippet actions()}
    <Button
      variant="ghost"
      onclick={() => (exportOpen = true)}
      class="border-line hover:border-teal hover:text-teal font-proto text-smaller flex h-8 items-center gap-1.5 border px-3 font-bold tracking-wider whitespace-nowrap uppercase"
    >
      <Icon name="export" size={12} />
      <span>{i18n.t.exportReportBtn}</span>
    </Button>
  {/snippet}

  <!-- Top Toolbar with 3 Dropdown Tabs on Left & Date Filter Controls on Right -->
  <div
    class="border-line relative z-20 mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2"
  >
    <div class="flex min-w-0 items-center gap-1">
      <ReportDropdownTabs
        currentTab={tabRouter.current}
        onSelectTab={(tab) => tabRouter.setTab(tab)}
      />
    </div>

    <!-- Right Side: Date Filter positioned directly beneath the Header Export Action -->
    <div class="flex shrink-0 items-center gap-2">
      {#if isRangeTab}
        <DateRangeDropdown bind:from bind:to />
      {:else if isAsOfTab}
        <div class="flex h-6 items-center gap-2">
          <span class="font-proto text-text-dim text-smaller tracking-wider uppercase">
            {i18n.t.asOfTodayLabel}:
          </span>
          <DateDropdown bind:value={to} />
          {#if to && to > todayString()}
            <span class="text-teal font-proto text-smaller animate-pulse font-bold">
              {i18n.t.forecastBadge}
            </span>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Content Canvas Scroll Area -->
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
    {#key tabRouter.current}
      <div
        class="flex min-h-full flex-col"
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

  <ExportOverlay
    bind:open={exportOpen}
    reportTitle={currentTabLabel}
    supportsPDF={tabRouter.current !== 'trends' && tabRouter.current !== 'income-exp'}
    onExport={handleExport}
  />
</PageLayout>
