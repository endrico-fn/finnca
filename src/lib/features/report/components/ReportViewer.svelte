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
  import ReportRailNav from './ReportRailNav.svelte';
  import { REPORT_GROUPS, REPORT_TAB_LABELS, reportGroupOf } from '../state/reportNav';

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
      class="font-proto text-small h-8 px-3 font-bold tracking-wider whitespace-nowrap flex items-center gap-2"
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

  <div class="border-line flex min-h-0 flex-1 overflow-hidden border">
    <!-- Desktop / Tablet Left-Hand Rail Navigation -->
    <div class="hidden md:flex">
      <ReportRailNav
        currentTab={tabRouter.current}
        onSelectTab={(tab) => tabRouter.setTab(tab)}
      />
    </div>

    <!-- Main Report Canvas -->
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden bg-bg-app">
      <!-- Top Canvas Toolbar with Date Controls & Section Indicator -->
      <div
        class="border-line/60 bg-bg-card/40 flex shrink-0 flex-wrap items-center justify-between gap-3 border-b px-4 py-2 select-none"
      >
        <div class="flex items-center gap-2">
          <span class="font-proto text-text-dim text-smaller font-bold tracking-wider uppercase">
            {i18n.t[activeGroupMeta.labelKey]}
          </span>
          <span class="text-text-muted text-smaller">/</span>
          <span class="font-proto text-text-white text-small font-bold">
            {currentTabLabel}
          </span>
        </div>

        <div class="flex items-center gap-3">
          {#if isRangeTab}
            <DateRangeDropdown bind:from bind:to />
          {:else if isAsOfTab}
            <div class="flex h-7 items-center gap-2">
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

      <!-- Mobile Category Navigation Strip -->
      <div
        class="border-line/60 bg-bg-card/30 flex shrink-0 items-center gap-1 overflow-x-auto border-b px-3 py-1.5 md:hidden select-none"
      >
        {#each REPORT_GROUPS as group (group.id)}
          {@const isGroupActive = activeGroupMeta.id === group.id}
          <button
            type="button"
            onclick={() => {
              if (!isGroupActive) tabRouter.setTab(group.defaultTab);
            }}
            class="font-proto text-smaller flex h-6 cursor-pointer items-center px-2 uppercase transition-colors whitespace-nowrap {isGroupActive
              ? 'border-line/80 bg-bg-card text-text-white border font-bold'
              : 'text-text-muted hover:text-text-white border border-transparent'}"
          >
            {#if isGroupActive}
              <span class="bg-teal mr-1.5 size-1 shrink-0"></span>
            {/if}
            {i18n.t[group.labelKey]}
          </button>
        {/each}
      </div>

      <!-- Mobile Single-Row Horizontal Tab Strip -->
      <div
        class="border-line/40 bg-bg-app flex shrink-0 items-center gap-1.5 overflow-x-auto border-b px-3 py-1.5 md:hidden select-none"
      >
        {#each activeGroupMeta.tabs as tabId (tabId)}
          {@const isSelected = tabRouter.current === tabId}
          <button
            type="button"
            onclick={() => tabRouter.setTab(tabId)}
            class="font-proto text-smaller flex h-7 cursor-pointer items-center px-2.5 uppercase transition-colors whitespace-nowrap {isSelected
              ? 'border-teal bg-teal/15 text-teal border font-bold'
              : 'border-line/50 bg-bg-btn text-text-muted hover:text-text-white border'}"
          >
            {i18n.t[REPORT_TAB_LABELS[tabId]]}
          </button>
        {/each}
      </div>

      <!-- Canvas Scroll Area -->
      <div class="flex min-h-0 w-full flex-1 flex-col overflow-y-auto p-4">
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
    </div>
  </div>
</PageLayout>
