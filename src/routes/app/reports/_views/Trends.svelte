<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { Tabs, Card } from '$lib/components/ui';
  import {
    historicalDailyBalances,
    accountBalanceMinor,
    convertMinor,
    formatIDR,
    type DailyDataPoint,
  } from '$lib/accounting/finance';

  let {
    bs,
    monthlyExpense,
    childrenMap,
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
    childrenMap: Map<string, string[]>;
    trendsPeriod: '1W' | '1M' | '3M' | 'YTD' | '1Y' | 'ALL';
    trendsMetric: 'netWorth' | 'assets' | 'liabilities' | 'liquidCash';
    trendsDateRange: { startStr: string; endStr: string };
    historicalPoints: DailyDataPoint[];
  }>();

  let hoveredIndex = $state<number | null>(null);
  let chartSvgEl = $state<SVGSVGElement | null>(null);

  function getPointValue(
    pt: DailyDataPoint | null,
    metric: 'netWorth' | 'assets' | 'liabilities' | 'liquidCash'
  ): number {
    if (!pt) return 0;
    if (metric === 'assets') return pt.assets;
    if (metric === 'liabilities') return pt.liabilities;
    if (metric === 'liquidCash') return pt.liquidCash;
    return pt.netWorth;
  }

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
    if (deltaNominal > 0) return 100;
    if (deltaNominal < 0) return -100;
    return 0;
  });
  const isNewCapital = $derived(baselineValue === 0 && deltaNominal > 0);
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

  const chartCoords = $derived.by(() => {
    const pts = historicalPoints;
    if (pts.length === 0)
      return {
        pathD: '',
        areaD: '',
        points: [],
        minVal: 0,
        maxVal: 0,
        peakPoint: null,
        troughPoint: null,
      };

    const vals = pts.map((p: DailyDataPoint) => getPointValue(p, trendsMetric));
    let minVal = Math.min(...vals);
    let maxVal = Math.max(...vals);

    if (minVal === maxVal) {
      minVal -= 100000;
      maxVal += 100000;
    } else {
      const margin = (maxVal - minVal) * 0.08;
      minVal -= margin;
      maxVal += margin;
    }

    const rangeY = maxVal - minVal;
    const n = pts.length;

    let peakIndex = 0;
    let troughIndex = 0;
    vals.forEach((v: number, i: number) => {
      if (v > vals[peakIndex]) peakIndex = i;
      if (v < vals[troughIndex]) troughIndex = i;
    });

    const points = pts.map((p: DailyDataPoint, i: number) => {
      const val = getPointValue(p, trendsMetric);
      const x = n > 1 ? padX + (i / (n - 1)) * (chartWidth - 2 * padX) : chartWidth / 2;
      const y = chartHeight - padY - ((val - minVal) / rangeY) * (chartHeight - 2 * padY);
      return { x, y, val, point: p };
    });

    const pathD = points
      .map(
        (pt: { x: number; y: number }, i: number) =>
          `${i === 0 ? 'M' : 'L'} ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`
      )
      .join(' ');
    const firstX = points[0].x;
    const lastX = points[points.length - 1].x;
    const areaD = `${pathD} L ${lastX.toFixed(1)} ${chartHeight} L ${firstX.toFixed(1)} ${chartHeight} Z`;

    const peakPoint = points[peakIndex] ?? null;
    const troughPoint = points[troughIndex] ?? null;

    return { pathD, areaD, points, minVal, maxVal, peakPoint, troughPoint };
  });

  const zeroLineY = $derived.by(() => {
    const { minVal, maxVal } = chartCoords;
    if (minVal < 0 && maxVal > 0) {
      const rangeY = maxVal - minVal;
      return chartHeight - padY - ((0 - minVal) / rangeY) * (chartHeight - 2 * padY);
    }
    return null;
  });

  const hoveredCoord = $derived.by(() => {
    if (hoveredIndex === null || !chartCoords.points[hoveredIndex]) return null;
    return chartCoords.points[hoveredIndex];
  });

  function handlePointerMove(e: PointerEvent) {
    if (!chartSvgEl || chartCoords.points.length === 0) return;
    const rect = chartSvgEl.getBoundingClientRect();
    const clientX = e.clientX - rect.left;
    const ratio = Math.max(0, Math.min(1, clientX / rect.width));
    const idx = Math.round(ratio * (chartCoords.points.length - 1));
    hoveredIndex = idx;
  }

  function handlePointerLeave() {
    hoveredIndex = null;
  }

  // Financial health ratios and allocations
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
    if (!ledger.data) return [];
    const cmap = childrenMap;
    const isLeaf = (id: string) => !cmap.has(id);
    const totalAssets = bs.assets > 0 ? bs.assets : 1;
    return ledger.accounts
      .filter((a) => isLeaf(a.id) && a.type === 'ASSET' && !a.placeholder)
      .map((a) => {
        const raw = accountBalanceMinor(a.id, ledger.data!, cmap);
        const balIdr = a.currency === 'USD' ? convertMinor(raw, 'USD', 'IDR', ledger.fxRate) : raw;
        const percent = Math.max(0, Math.min(100, (balIdr / totalAssets) * 100));
        return { account: a, balIdr, percent: percent.toFixed(1) };
      })
      .filter((a) => a.balIdr > 0)
      .sort((a, b) => b.balIdr - a.balIdr)
      .slice(0, 5);
  });
</script>

<div class="flex min-h-0 flex-1 flex-col space-y-2 overflow-y-auto pr-1 font-mono">
  <!-- HERO CARD: ROBINHOOD SIGNATURE FINANCIAL LINE CHART -->
  <div class="sharp-card relative flex shrink-0 flex-col px-3 py-2.5">
    <!-- Header: Hero Metric Label & Large Balance -->
    <div class="flex items-start justify-between pb-1">
      <div>
        <div class="flex items-start gap-2">
          <p class="label-title text-text-strong leading-none">
            {trendsMetric === 'netWorth'
              ? i18n.t.metricNetWorth
              : trendsMetric === 'assets'
                ? i18n.t.metricTotalAssets
                : trendsMetric === 'liabilities'
                  ? i18n.t.totalLiabilities
                  : i18n.t.metricLiquidCash}
          </p>
          <span
            class="border-line bg-bg-app text-text-muted font-proto inline-flex items-center justify-center border px-1.5 py-0.5 text-[9px] leading-none tracking-wider uppercase transition-opacity {hoveredCoord
              ? 'opacity-100'
              : 'pointer-events-none opacity-0'}"
          >
            {activeDateFormatted}
          </span>
        </div>
        <div
          class="text-text-strong font-proto mt-1.5 text-[26px] leading-none font-bold tracking-tight tabular-nums"
        >
          {formatIDR(currentValue)}
        </div>
        <div class="mt-1.5 flex items-center gap-2 text-[11px]">
          <span
            class="font-proto font-bold tabular-nums {isHealthyChange
              ? 'text-income'
              : 'text-expense'}"
          >
            {deltaNominal >= 0 ? '▲' : '▼'}
            {deltaNominal >= 0 ? '+' : ''}{formatIDR(deltaNominal)}
            {#if isNewCapital}
              (NEW)
            {:else}
              ({deltaPercent >= 0 ? '+' : ''}{deltaPercent.toFixed(2)}%)
            {/if}
          </span>
          <span class="text-text-dim font-proto text-[10px] tracking-wide uppercase">
            {hoveredCoord
              ? `vs ${trendsDateRange.startStr}`
              : `${i18n.t.pastPeriodLabel} (${trendsPeriod})`}
          </span>
        </div>
      </div>

      <!-- High / Low indicator for period -->
      <div class="text-text-dim font-proto hidden space-y-0.5 text-right text-[10px] sm:block">
        <div>
          HIGH: <span class="text-text-base">{formatIDR(chartCoords.maxVal)}</span>
        </div>
        <div>
          LOW: <span class="text-text-base">{formatIDR(chartCoords.minVal)}</span>
        </div>
      </div>
    </div>

    <!-- Interactive SVG Chart -->
    <div
      role="region"
      aria-label="Interactive Financial Chart"
      class="relative mt-2 h-55 w-full cursor-crosshair select-none"
      onpointermove={handlePointerMove}
      onpointerleave={handlePointerLeave}
    >
      {#if historicalPoints.length === 0}
        <div
          class="text-text-dim border-line font-proto flex h-full w-full items-center justify-center border border-dashed text-[11px]"
        >
          {i18n.t.noHistoricalData}
        </div>
      {:else}
        <svg
          bind:this={chartSvgEl}
          viewBox="0 0 {chartWidth} {chartHeight}"
          preserveAspectRatio="none"
          class="h-full w-full overflow-visible"
        >
          <defs>
            <linearGradient id="chartFillGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color={neonColor} stop-opacity="0.16" />
              <stop offset="100%" stop-color={neonColor} stop-opacity="0.00" />
            </linearGradient>
          </defs>

          <!-- Zero baseline if values cross 0 -->
          {#if zeroLineY !== null}
            <line
              x1={padX}
              y1={zeroLineY}
              x2={chartWidth - padX}
              y2={zeroLineY}
              stroke="var(--color-line)"
              stroke-dasharray="3,3"
              opacity="0.6"
            />
          {/if}

          <!-- Subtle gradient area fill -->
          <path d={chartCoords.areaD} fill="url(#chartFillGrad)" />

          <!-- Main crisp trend line -->
          <path
            d={chartCoords.pathD}
            fill="none"
            stroke={neonColor}
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />

          <!-- High and Low subtle markers when NOT hovered -->
          {#if !hoveredCoord && chartCoords.points.length > 3 && chartCoords.peakPoint && chartCoords.troughPoint && chartCoords.peakPoint.val !== chartCoords.troughPoint.val}
            <!-- Peak Point Marker -->
            <circle
              cx={chartCoords.peakPoint.x}
              cy={chartCoords.peakPoint.y}
              r="3"
              fill="var(--color-income)"
              opacity="0.85"
            />
            <text
              x={chartCoords.peakPoint.x}
              y={Math.max(12, chartCoords.peakPoint.y - 6)}
              fill="var(--color-income)"
              font-size="8.5"
              font-family="var(--font-mono)"
              font-weight="bold"
              text-anchor="middle"
              opacity="0.85"
            >
              H
            </text>

            <!-- Trough Point Marker -->
            <circle
              cx={chartCoords.troughPoint.x}
              cy={chartCoords.troughPoint.y}
              r="3"
              fill="var(--color-expense)"
              opacity="0.85"
            />
            <text
              x={chartCoords.troughPoint.x}
              y={Math.min(chartHeight - 6, chartCoords.troughPoint.y + 12)}
              fill="var(--color-expense)"
              font-size="8.5"
              font-family="var(--font-mono)"
              font-weight="bold"
              text-anchor="middle"
              opacity="0.85"
            >
              L
            </text>
          {/if}

          <!-- Hover crosshair indicator & floating pill tooltip -->
          {#if hoveredCoord}
            <line
              x1={hoveredCoord.x}
              y1={0}
              x2={hoveredCoord.x}
              y2={chartHeight}
              stroke="var(--color-text-muted)"
              stroke-dasharray="2,2"
              stroke-width="1"
              opacity="0.6"
            />
            <circle cx={hoveredCoord.x} cy={hoveredCoord.y} r="7" fill={neonColor} opacity="0.25" />
            <circle
              cx={hoveredCoord.x}
              cy={hoveredCoord.y}
              r="3.5"
              fill={neonColor}
              stroke="var(--color-bg-base)"
              stroke-width="2"
            />

            <!-- Floating Tooltip Pill inside SVG -->
            {@const tipW = 105}
            {@const tipH = 22}
            {@const tipX = Math.max(
              padX,
              Math.min(chartWidth - padX - tipW, hoveredCoord.x - tipW / 2)
            )}
            {@const tipY = hoveredCoord.y > 36 ? hoveredCoord.y - tipH - 8 : hoveredCoord.y + 10}
            <g class="pointer-events-none">
              <rect
                x={tipX}
                y={tipY}
                width={tipW}
                height={tipH}
                fill="var(--color-bg-card)"
                stroke="var(--color-line)"
                stroke-width="1"
              />
              <text
                x={tipX + tipW / 2}
                y={tipY + 14}
                fill="var(--color-text-strong)"
                font-size="9"
                font-family="var(--font-mono)"
                font-weight="bold"
                text-anchor="middle"
              >
                {formatIDR(hoveredCoord.val)}
              </text>
            </g>
          {/if}
        </svg>
      {/if}
    </div>

    <!-- Period Selector Pills (1W, 1M, 3M, YTD, 1Y, ALL) -->
    <div class="border-line/60 mt-1 flex shrink-0 items-center justify-between border-t pt-2.5">
      <Tabs
        variant="outline"
        tabs={(['1W', '1M', '3M', 'YTD', '1Y', 'ALL'] as const).map((p) => ({
          id: p,
          label: p,
        }))}
        active={trendsPeriod}
        onSelect={(id) => (trendsPeriod = id as typeof trendsPeriod)}
      />
      <div class="text-text-dim font-proto hidden text-[10px] sm:block">
        {trendsDateRange.startStr} — {trendsDateRange.endStr} ({historicalPoints.length}
        {i18n.t.daysCountLabel})
      </div>
    </div>
  </div>

  <!-- BRUTALIST MONOSPACE BREAKDOWN (Interactive Metric Switchers) -->
  <div class="grid shrink-0 grid-cols-4 gap-3 text-[11px]">
    <!-- [01] ASSETS (Click to plot Total Assets) -->
    <button
      type="button"
      onclick={() => (trendsMetric = 'assets')}
      class="bg-bg-card cursor-pointer space-y-1 border px-3 py-2.5 text-left transition-all {trendsMetric ===
      'assets'
        ? 'border-teal/70 bg-bg-row-active/40'
        : 'border-line hover:border-line/80 hover:bg-bg-card/70'}"
    >
      <div class="text-text-dim font-proto flex items-center justify-between gap-1 text-[10px]">
        <span class="truncate {trendsMetric === 'assets' ? 'text-teal font-semibold' : ''}"
          >[01] {i18n.t.assetsTitle}</span
        >
        <span
          class="py-0.2 shrink-0 border px-1 text-[8.5px] whitespace-nowrap {trendsMetric ===
          'assets'
            ? 'border-teal/50 text-teal'
            : 'border-line text-text-dim'}">TOTAL</span
        >
      </div>
      <div class="text-text-strong font-proto text-[15px] font-bold tabular-nums">
        {formatIDR(bs.assets)}
      </div>
      <div class="text-text-muted font-proto truncate text-[9px]">
        {i18n.t.allAccountsDesc}
      </div>
    </button>

    <!-- [02] LIABILITIES (Click to plot Liabilities) -->
    <button
      type="button"
      onclick={() => (trendsMetric = 'liabilities')}
      class="bg-bg-card cursor-pointer space-y-1 border px-3 py-2.5 text-left transition-all {trendsMetric ===
      'liabilities'
        ? 'border-teal/70 bg-bg-row-active/40'
        : 'border-line hover:border-line/80 hover:bg-bg-card/70'}"
    >
      <div class="text-text-dim font-proto flex items-center justify-between gap-1 text-[10px]">
        <span class="truncate {trendsMetric === 'liabilities' ? 'text-teal font-semibold' : ''}"
          >[02] {i18n.t.liabilitiesShortLabel}</span
        >
        <span
          class="py-0.2 shrink-0 border px-1 text-[8.5px] whitespace-nowrap {trendsMetric ===
          'liabilities'
            ? 'border-teal/50 text-teal'
            : 'border-line text-text-dim'}">DEBT</span
        >
      </div>
      <div class="text-expense font-proto text-[15px] font-bold tabular-nums">
        {formatIDR(bs.liabilities)}
      </div>
      <div class="text-text-muted font-proto truncate text-[9px]">
        {i18n.t.liabilitiesDesc}
      </div>
    </button>

    <!-- [03] LIQUID CASH (Click to plot Liquid Cash) -->
    <button
      type="button"
      onclick={() => (trendsMetric = 'liquidCash')}
      class="bg-bg-card cursor-pointer space-y-1 border px-3 py-2.5 text-left transition-all {trendsMetric ===
      'liquidCash'
        ? 'border-teal/70 bg-bg-row-active/40'
        : 'border-line hover:border-line/80 hover:bg-bg-card/70'}"
    >
      <div class="text-text-dim font-proto flex items-center justify-between gap-1 text-[10px]">
        <span class="truncate {trendsMetric === 'liquidCash' ? 'text-teal font-semibold' : ''}"
          >[03] {i18n.t.metricLiquidCash}</span
        >
        <span
          class="py-0.2 shrink-0 border px-1 text-[8.5px] whitespace-nowrap {trendsMetric ===
          'liquidCash'
            ? 'border-teal/50 text-teal'
            : 'border-line text-text-dim'}">LIQUID</span
        >
      </div>
      <div class="text-text-strong font-proto text-[15px] font-bold tabular-nums">
        {formatIDR(activePoint ? activePoint.liquidCash : 0)}
      </div>
      <div class="text-text-muted font-proto truncate text-[9px]">
        {i18n.t.liquidCashDesc}
      </div>
    </button>

    <!-- [04] NET WORTH (Click to plot Net Worth) -->
    <button
      type="button"
      onclick={() => (trendsMetric = 'netWorth')}
      class="bg-bg-card cursor-pointer space-y-1 border px-3 py-2.5 text-left transition-all {trendsMetric ===
      'netWorth'
        ? 'border-teal/70 bg-bg-row-active/40'
        : 'border-line hover:border-line/80 hover:bg-bg-card/70'}"
    >
      <div class="text-text-dim font-proto flex items-center justify-between gap-1 text-[10px]">
        <span class="truncate {trendsMetric === 'netWorth' ? 'text-teal font-semibold' : ''}"
          >[04] {i18n.t.metricNetWorth}</span
        >
        <span
          class="py-0.2 shrink-0 border px-1 text-[8.5px] whitespace-nowrap {trendsMetric ===
          'netWorth'
            ? 'border-teal/50 text-teal'
            : 'border-line text-text-dim'}">NET</span
        >
      </div>
      <div class="text-text-strong font-proto text-[15px] font-bold tabular-nums">
        {formatIDR(activePoint ? activePoint.netWorth : bs.equity)}
      </div>
      <div class="text-text-muted font-proto truncate text-[9px]">
        Δ {deltaNominal >= 0 ? '+' : ''}{formatIDR(deltaNominal)} ({isNewCapital
          ? 'NEW'
          : `${deltaPercent >= 0 ? '+' : ''}${deltaPercent.toFixed(2)}%`})
      </div>
    </button>
  </div>

  <!-- BOTTOM ROW: TOP ASSET ALLOCATION & FINANCIAL HEALTH RATIOS -->
  <div class="grid grid-cols-12 gap-2 pb-2">
    <!-- Left: Top Asset Holdings (Col-span-7) -->
    <Card title={i18n.t.topHoldings} class="col-span-7">
      {#snippet header()}
        <span class="text-text-dim font-proto text-[10px] leading-none">TOP 5</span>
      {/snippet}

      <div class="space-y-2.5">
        {#each topAssetAccounts as item (item.account.id)}
          <div class="space-y-1">
            <div class="flex items-center justify-between text-[11px]">
              <div class="flex items-center gap-1.5 truncate">
                <span class="text-text-dim font-proto text-[10px]">{item.account.code}</span>
                <span class="text-text-strong truncate font-medium">{item.account.name}</span>
              </div>
              <div class="font-proto shrink-0 text-right">
                <span class="text-text-base">{formatIDR(item.balIdr)}</span>
                <span class="text-text-dim ml-1 text-[10px]">({item.percent}%)</span>
              </div>
            </div>
            <!-- Distribution Bar -->
            <div class="bg-bg-app border-line h-1.5 overflow-hidden border">
              <div class="bg-income h-full" style="width: {item.percent}%"></div>
            </div>
          </div>
        {:else}
          <div class="text-text-dim p-4 text-center text-[11px]">No asset holdings recorded</div>
        {/each}
      </div>
    </Card>

    <!-- Right: Financial Health Ratios (Col-span-5) -->
    <Card title={i18n.t.financialHealthRatios} class="col-span-5">
      {#snippet header()}
        <span class="text-text-dim font-proto text-[10px] leading-none">METRICS</span>
      {/snippet}

      <div class="space-y-3 text-[11px]">
        <!-- Debt Ratio -->
        <div class="sharp-card space-y-1 p-2.5">
          <div class="flex items-center justify-between">
            <span class="text-text-muted text-[10px]">{i18n.t.debtToAssetRatio}</span>
            <span
              class="border-line border px-1.5 py-0.5 text-[9px] font-bold uppercase
              {Number(debtRatio) < 30
                ? 'text-income border-income/40'
                : Number(debtRatio) < 60
                  ? 'text-text-base'
                  : 'text-expense border-expense/40'}"
            >
              {debtStatus}
            </span>
          </div>
          <div class="text-text-strong text-[16px] font-bold">
            {debtRatio}%
          </div>
          <div class="text-text-dim text-[9px]">Total Debt / Total Assets</div>
        </div>

        <!-- Cash Runway -->
        <div class="sharp-card space-y-1 p-2.5">
          <div class="flex items-center justify-between">
            <span class="text-text-muted text-[10px]">{i18n.t.cashRunwayMonths}</span>
            <span class="text-text-dim font-proto text-[9px] uppercase">RESERVES</span>
          </div>
          <div class="text-text-strong text-[16px] font-bold">
            {runwayMonths}
            <span class="text-text-muted text-[11px] font-normal">{i18n.t.months}</span>
          </div>
          <div class="text-text-dim text-[9px]">Liquid Cash / Monthly Expense</div>
        </div>
      </div>
    </Card>
  </div>
</div>
