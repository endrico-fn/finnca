<script lang="ts">
  import {
    type DailyDataPoint,
    type AccountReportRow,
    reportState,
  } from '$lib/features/report/state/report.svelte';
  import { formatIDR } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Card, Tabs } from '$lib/components/ui';
  import type { Account } from '$lib/core/ipc/bindings';
  import {
    getPointValue,
    calculateChartCoords,
    calculateZeroLineY,
    type TrendsMetric,
    type TrendsPeriod,
  } from '../state/trendsChartUtils';
  import TrendsChart from './TrendsChart.svelte';
  import TrendsBars from './TrendsBars.svelte';
  import TrendsMetricCards from './TrendsMetricCards.svelte';
  import TrendsHealthCards from './TrendsHealthCards.svelte';

  let {
    bs,
    monthlyExpense,
    trendsPeriod = $bindable(),
    trendsMetric = $bindable(),
    trendsDateRange,
    historicalPoints,
  } = $props<{
    bs: {
      assets: number;
      liabilities: number;
      equity: number;
      netIncome: number;
      balanced: boolean;
    };
    monthlyExpense: number;
    trendsPeriod: TrendsPeriod;
    trendsMetric: TrendsMetric;
    trendsDateRange: { startStr: string; endStr: string };
    historicalPoints: DailyDataPoint[];
  }>();

  let hoveredIndex = $state<number | null>(null);
  let chartMode = $state<'line' | 'bars'>('line');

  const flowBars = $derived(
    historicalPoints.map((p: DailyDataPoint, i: number) => ({
      date: p.date,
      value:
        i === 0
          ? 0
          : getPointValue(p, trendsMetric) - getPointValue(historicalPoints[i - 1], trendsMetric),
    }))
  );

  const modeTabs = $derived([
    { id: 'line', label: i18n.t.trendsViewLine },
    { id: 'bars', label: i18n.t.trendsViewBars },
  ]);

  const activePoint = $derived.by(() => {
    if (historicalPoints.length === 0) return null;
    if (hoveredIndex !== null && hoveredIndex >= 0 && hoveredIndex < historicalPoints.length) {
      return historicalPoints[hoveredIndex];
    }
    return historicalPoints[historicalPoints.length - 1];
  });

  const baselinePoint = $derived(historicalPoints.length > 0 ? historicalPoints[0] : null);

  const currentValue = $derived(getPointValue(activePoint, trendsMetric));
  const baselineValue = $derived(getPointValue(baselinePoint, trendsMetric));
  const deltaNominal = $derived(currentValue - baselineValue);
  const deltaPercent = $derived.by(() => {
    if (baselineValue !== 0) {
      return (deltaNominal / Math.abs(baselineValue)) * 100;
    }
    return 0;
  });
  const isNewCapital = $derived(baselineValue === 0 && currentValue > 0);
  const isHealthyChange = $derived(
    trendsMetric === 'liabilities' ? deltaNominal <= 0 : deltaNominal >= 0
  );
  const neonColor = $derived(isHealthyChange ? 'var(--color-income)' : 'var(--color-expense)');

  const activeDateFormatted = $derived.by(() => {
    if (!activePoint) return '';
    const [y, m, d] = activePoint.date.split('-').map(Number);
    const dateObj = new Date(y, m - 1, d);
    return dateObj.toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
      weekday: 'short',
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  });

  const chartWidth = 900;
  const chartHeight = 240;
  const padX = 16;
  const padY = 24;

  const chartCoords = $derived(
    calculateChartCoords(historicalPoints, trendsMetric, chartWidth, chartHeight, padX, padY)
  );

  const zeroLineY = $derived(
    calculateZeroLineY(chartCoords.minVal, chartCoords.maxVal, chartHeight, padY)
  );

  const hoveredCoord = $derived.by(() => {
    if (hoveredIndex === null || !chartCoords.points[hoveredIndex]) return null;
    return chartCoords.points[hoveredIndex];
  });

  const debtRatio = $derived(
    bs.assets > 0 ? ((bs.liabilities / bs.assets) * 100).toFixed(1) : '0.0'
  );
  const debtStatus = $derived(
    Number(debtRatio) < 30
      ? i18n.t.healthy
      : Number(debtRatio) < 60
        ? i18n.t.caution
        : i18n.t.highLeverage
  );

  const runwayMonths = $derived.by(() => {
    const liquid = activePoint ? activePoint.liquidCash : 0;
    const mb = monthlyExpense > 0 ? monthlyExpense : 1;
    if (liquid <= 0) return '0.0';
    if (mb <= 0) return '> 99';
    const r = (liquid / mb).toFixed(1);
    return Number(r) > 99 ? '> 99' : r;
  });

  const topAssetAccounts = $derived.by(() => {
    const totalAssets = bs.assets > 0 ? bs.assets : 1;
    const rows: AccountReportRow[] = reportState.balanceSheet?.asset_rows ?? [];
    return rows
      .filter((r: AccountReportRow) => r.amount > 0)
      .sort((a: AccountReportRow, b: AccountReportRow) => b.amount - a.amount)
      .slice(0, 5)
      .map((r: AccountReportRow) => {
        const percent = Math.max(0, Math.min(100, (r.amount / totalAssets) * 100));
        return {
          account: { id: r.account_id, code: r.code, name: r.name } as Account,
          balIdr: r.amount,
          percent: percent.toFixed(1),
        };
      });
  });

  const heroTitle = $derived(
    trendsMetric === 'netWorth'
      ? i18n.t.metricNetWorth
      : trendsMetric === 'assets'
        ? i18n.t.metricTotalAssets
        : trendsMetric === 'liabilities'
          ? i18n.t.totalLiabilities
          : i18n.t.metricLiquidCash
  );
</script>

<div class="flex min-h-0 flex-1 flex-col space-y-2 overflow-y-auto pr-3">
  <Card title={heroTitle} class="relative shrink-0">
    {#snippet header()}
      <div class="flex items-center gap-3">
        <Tabs
          variant="outline"
          tabs={modeTabs}
          active={chartMode}
          onSelect={(id) => (chartMode = id as 'line' | 'bars')}
        />
        <span
          class="border-line bg-bg-app text-text-muted font-proto text-smaller inline-flex items-center justify-center border px-2 py-0.5 leading-none tracking-wider uppercase transition-opacity {hoveredCoord
            ? 'opacity-100'
            : 'pointer-events-none opacity-0'}"
        >
          {activeDateFormatted}
        </span>
        <div class="text-text-dim font-proto text-smaller hidden items-center gap-3 sm:flex">
          <div>
            {i18n.t.trendsHighLabel}:
            <span class="text-text-base">{formatIDR(chartCoords.maxVal)}</span>
          </div>
          <span class="text-line/40">|</span>
          <div>
            {i18n.t.trendsLowLabel}:
            <span class="text-text-base">{formatIDR(chartCoords.minVal)}</span>
          </div>
        </div>
      </div>
    {/snippet}

    <div>
      <div
        class="text-text-strong font-proto text-largest leading-none font-bold tracking-tight tabular-nums"
      >
        {formatIDR(currentValue)}
      </div>
      <div class="text-small mt-2 flex items-center gap-2">
        <span
          class="font-proto font-bold tabular-nums {isHealthyChange
            ? 'text-income'
            : 'text-expense'}"
        >
          {deltaNominal >= 0 ? '▲' : '▼'}
          {deltaNominal >= 0 ? '+' : ''}{formatIDR(deltaNominal)}
          {#if isNewCapital}
            {i18n.t.trendsNewBadge}
          {:else}
            ({deltaPercent >= 0 ? '+' : ''}{deltaPercent.toFixed(2)}%)
          {/if}
        </span>
        <span class="text-text-dim font-proto text-smaller tracking-wide uppercase">
          {hoveredCoord
            ? i18n.t.trendsVsDate.replace('{date}', trendsDateRange.startStr)
            : `${i18n.t.pastPeriodLabel} (${trendsPeriod})`}
        </span>
      </div>
    </div>

    {#if chartMode === 'line'}
      <TrendsChart
        {chartCoords}
        {chartWidth}
        {chartHeight}
        {padX}
        {neonColor}
        {zeroLineY}
        onHoverChange={(idx) => (hoveredIndex = idx)}
        {hoveredCoord}
        hasHistoricalData={historicalPoints.length > 0}
      />
    {:else}
      <TrendsBars
        bars={flowBars}
        {chartWidth}
        {chartHeight}
        {padX}
        {hoveredIndex}
        onHoverChange={(idx) => (hoveredIndex = idx)}
        hasHistoricalData={historicalPoints.length > 0}
      />
    {/if}

    <div class="border-line/60 mt-1 flex shrink-0 items-center justify-between border-t pt-3">
      <div class="flex items-center gap-1">
        {#each ['1W', '1M', '3M', 'YTD', '1Y', 'ALL'] as p (p)}
          <button
            type="button"
            onclick={() => (trendsPeriod = p as typeof trendsPeriod)}
            class="font-proto text-smaller h-6 cursor-pointer border px-2 transition-colors {trendsPeriod ===
            p
              ? 'border-teal bg-teal/15 text-teal font-bold'
              : 'border-line bg-bg-app hover:border-text-dim text-text-muted hover:text-text-base'}"
          >
            {p}
          </button>
        {/each}
      </div>
      <div class="text-text-dim font-proto text-smaller hidden sm:block">
        {trendsDateRange.startStr} — {trendsDateRange.endStr} ({historicalPoints.length}
        {i18n.t.daysCountLabel})
      </div>
    </div>
  </Card>

  <TrendsMetricCards
    {bs}
    {activePoint}
    {deltaNominal}
    {deltaPercent}
    {isNewCapital}
    bind:trendsMetric
  />

  <TrendsHealthCards {topAssetAccounts} {debtRatio} {debtStatus} {runwayMonths} />
</div>
