<script lang="ts">
  import { type DailyDataPoint } from '$lib/features/report/state/report.svelte';
  import { reportState } from '$lib/features/report/state/report.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { KpiCard, AnimatedCounter } from '$lib/components/ui';
  import {
    calculateChartCoords,
    calculateZeroLineY,
    type TrendsPeriod,
  } from '../state/trendsChartUtils';
  import type { Account, AccountReportRow } from '$lib/core/ipc/bindings';
  import TrendsChart from './TrendsChart.svelte';
  import TrendsHealthCards from './TrendsHealthCards.svelte';

  let {
    bs,
    monthlyExpense,
    historicalPoints,
    trendsPeriod = $bindable('3M'),
    trendsDateRange,
  }: {
    bs: {
      assets: number;
      liabilities: number;
      equity: number;
      netIncome: number;
      balanced: boolean;
    };
    monthlyExpense: number;
    historicalPoints: DailyDataPoint[];
    trendsPeriod: TrendsPeriod;
    trendsDateRange: { startStr: string; endStr: string };
  } = $props();

  const peakPoint = $derived(
    historicalPoints.length > 0
      ? historicalPoints.reduce((best, p) => (p.netWorth > best.netWorth ? p : best))
      : null
  );

  const currentPoint = $derived(
    historicalPoints.length > 0 ? historicalPoints[historicalPoints.length - 1] : null
  );

  const startPoint = $derived(historicalPoints.length > 0 ? historicalPoints[0] : null);

  const chartWidth = 900;
  const chartHeight = 240;
  const padX = 16;
  const padY = 24;

  const chartCoords = $derived(
    calculateChartCoords(historicalPoints, 'netWorth', chartWidth, chartHeight, padX, padY)
  );

  const zeroLineY = $derived(
    calculateZeroLineY(chartCoords.minVal, chartCoords.maxVal, chartHeight, padY)
  );

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
    const liquid = currentPoint ? currentPoint.liquidCash : 0;
    if (liquid <= 0) return '0.0';
    const mb = monthlyExpense > 0 ? monthlyExpense : 1;
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

  const PERIODS: { id: TrendsPeriod; label: string }[] = [
    { id: '1W', label: '1W' },
    { id: '1M', label: '1M' },
    { id: '3M', label: '3M' },
    { id: 'YTD', label: 'YTD' },
    { id: '1Y', label: '1Y' },
    { id: 'ALL', label: 'ALL' },
  ];
</script>

<div class="flex min-h-0 flex-1 flex-col space-y-4 overflow-y-auto pr-3">
  <!-- Milestones -->
  <div class="grid shrink-0 grid-cols-3 gap-2">
    <KpiCard label={i18n.t.networthPeakLabel}>
      <AnimatedCounter
        value={peakPoint?.netWorth ?? 0}
        currency="IDR"
        class="text-medium text-teal block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block">{peakPoint?.date ?? '—'}</span>
    </KpiCard>
    <KpiCard label={i18n.t.networthCurrentLabel}>
      <AnimatedCounter
        value={currentPoint?.netWorth ?? 0}
        currency="IDR"
        class="text-medium {(currentPoint?.netWorth ?? 0) >= 0
          ? 'text-income'
          : 'text-expense'} block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block">{currentPoint?.date ?? '—'}</span>
    </KpiCard>
    <KpiCard label={i18n.t.networthStartLabel}>
      <AnimatedCounter
        value={startPoint?.netWorth ?? 0}
        currency="IDR"
        class="text-medium text-text-base block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block">{trendsDateRange.startStr}</span>
    </KpiCard>
  </div>

  <div class="sharp-card flex flex-1 flex-col p-4">
    <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
      <h3 class="font-proto text-small text-text-strong tracking-widest uppercase">
        {i18n.t.networthProgressionTitle}
      </h3>
      <div class="border-line flex rounded-none border">
        {#each PERIODS as p (p.id)}
          <button
            type="button"
            class="hover:bg-bg-btn text-smaller flex h-7 cursor-pointer items-center justify-center px-3 font-medium transition-colors
              {trendsPeriod === p.id ? 'bg-bg-btn text-teal' : 'text-text-dim'}"
            onclick={() => (trendsPeriod = p.id)}
          >
            {p.label}
          </button>
        {/each}
      </div>
    </div>

    <TrendsChart
      {chartCoords}
      {chartWidth}
      {chartHeight}
      {padX}
      {zeroLineY}
      neonColor="var(--color-teal)"
      hasHistoricalData={historicalPoints.length > 0}
      onHoverChange={() => {}}
    />
  </div>

  <TrendsHealthCards {topAssetAccounts} {debtRatio} {debtStatus} {runwayMonths} />
</div>
