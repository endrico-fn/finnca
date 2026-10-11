<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteDate } from 'svelte/reactivity';
  import { getHistoricalTrendsReportCmd, type DailyTrendPoint } from '$lib/core/ipc/bindings';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { fxState } from '$lib/core/state/fx.svelte';
  import { todayString } from '$lib/core/format/date';
  import { i18n } from '$lib/core/i18n.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';

  let {
    class: extraClass = '',
  }: {
    class?: string;
  } = $props();

  let loading = $state(true);
  let points = $state<DailyTrendPoint[]>([]);
  let selectedMetric = $state<'net_worth' | 'assets' | 'liquid_cash'>('net_worth');
  let hoveredIndex = $state<number | null>(null);

  function getPastDate(days: number): string {
    const d = new SvelteDate();
    d.setDate(d.getDate() - days);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  async function loadTrendData() {
    loading = true;
    try {
      const fxRate = fxState.rate;
      const toDate = todayString();
      const fromDate = getPastDate(30);
      const report = await getHistoricalTrendsReportCmd(fromDate, toDate, fxRate);
      points = report.points ?? [];
    } catch {
      points = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadTrendData();
    const unsubTx = eventBus.on('transaction:posted', () => loadTrendData());
    const unsubAcc = eventBus.on('accounts:changed', () => loadTrendData());
    return () => {
      unsubTx();
      unsubAcc();
    };
  });

  const chartWidth = 720;
  const chartHeight = 220;
  const padX = 24;
  const padY = 24;

  const vals = $derived(
    points.map((p) =>
      selectedMetric === 'assets'
        ? p.assets
        : selectedMetric === 'liquid_cash'
          ? p.liquid_cash
          : p.net_worth
    )
  );

  const minVal = $derived(vals.length > 0 ? Math.min(...vals) : 0);
  const maxVal = $derived(vals.length > 0 ? Math.max(...vals) : 0);
  const rangeY = $derived.by(() => {
    if (vals.length === 0) return 1;
    const diff = maxVal - minVal;
    return diff === 0 ? Math.max(1, Math.abs(maxVal) * 0.2) : diff * 1.15;
  });

  const baseMinY = $derived.by(() => {
    if (vals.length === 0) return 0;
    const diff = maxVal - minVal;
    return diff === 0 ? minVal - Math.abs(minVal) * 0.1 : minVal - diff * 0.08;
  });

  const chartPoints = $derived(
    points.map((p, i) => {
      const val = vals[i];
      const x =
        points.length > 1
          ? padX + (i / (points.length - 1)) * (chartWidth - 2 * padX)
          : chartWidth / 2;
      const y = chartHeight - padY - ((val - baseMinY) / rangeY) * (chartHeight - 2 * padY);
      return { x, y, val, date: p.date };
    })
  );

  const pathD = $derived.by(() => {
    if (chartPoints.length === 0) return '';
    return chartPoints
      .map((pt, i) => `${i === 0 ? 'M' : 'L'} ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`)
      .join(' ');
  });

  const areaD = $derived.by(() => {
    if (chartPoints.length === 0) return '';
    const firstX = chartPoints[0].x;
    const lastX = chartPoints[chartPoints.length - 1].x;
    return `${pathD} L ${lastX.toFixed(1)} ${chartHeight - padY / 2} L ${firstX.toFixed(1)} ${chartHeight - padY / 2} Z`;
  });

  const latestVal = $derived(vals.length > 0 ? vals[vals.length - 1] : 0);
  const startVal = $derived(vals.length > 0 ? vals[0] : 0);
  const deltaVal = $derived(latestVal - startVal);
  const activeHoverPoint = $derived(
    hoveredIndex !== null && chartPoints[hoveredIndex] ? chartPoints[hoveredIndex] : null
  );

  let chartSvgEl = $state<SVGSVGElement | null>(null);

  function handlePointerMove(e: PointerEvent) {
    if (!chartSvgEl || chartPoints.length === 0) return;
    const rect = chartSvgEl.getBoundingClientRect();
    const clientX = e.clientX - rect.left;
    const ratio = Math.max(0, Math.min(1, clientX / rect.width));
    hoveredIndex = Math.round(ratio * (chartPoints.length - 1));
  }

  function handlePointerLeave() {
    hoveredIndex = null;
  }
</script>

<div class="flex h-full w-full flex-col {extraClass}">
  <!-- Metric sub-header -->
  <div class="border-line/60 flex shrink-0 items-center justify-between border-b px-1 pt-1 pb-2">
    <div class="flex items-center gap-1.5">
      <button
        type="button"
        onclick={() => (selectedMetric = 'net_worth')}
        class="font-proto text-smaller px-2 py-0.5 font-bold uppercase transition-colors {selectedMetric ===
        'net_worth'
          ? 'bg-bg-btn text-teal border-teal/40 border'
          : 'text-text-muted hover:text-text-base border border-transparent'}"
      >
        {i18n.t.netWorth}
      </button>
      <button
        type="button"
        onclick={() => (selectedMetric = 'assets')}
        class="font-proto text-smaller px-2 py-0.5 font-bold uppercase transition-colors {selectedMetric ===
        'assets'
          ? 'bg-bg-btn text-teal border-teal/40 border'
          : 'text-text-muted hover:text-text-base border border-transparent'}"
      >
        {i18n.t.totalAssetsLabel}
      </button>
      <button
        type="button"
        onclick={() => (selectedMetric = 'liquid_cash')}
        class="font-proto text-smaller px-2 py-0.5 font-bold uppercase transition-colors {selectedMetric ===
        'liquid_cash'
          ? 'bg-bg-btn text-teal border-teal/40 border'
          : 'text-text-muted hover:text-text-base border border-transparent'}"
      >
        {i18n.t.liquidCash}
      </button>
    </div>

    <!-- Active Hover or Latest Value -->
    <div class="flex items-baseline gap-2">
      {#if activeHoverPoint}
        <span class="text-text-muted font-proto text-smaller tabular-nums">
          {activeHoverPoint.date}
        </span>
        <span class="text-text-white font-proto text-small font-bold tabular-nums">
          {formatMinorToDisplay(activeHoverPoint.val, 'IDR')}
        </span>
      {:else}
        <span class="text-text-muted font-proto text-smaller">30D</span>
        <span class="text-text-white font-proto text-small font-bold tabular-nums">
          {formatMinorToDisplay(latestVal, 'IDR')}
        </span>
        <span
          class="font-proto text-smaller font-bold tabular-nums {deltaVal >= 0
            ? 'text-income'
            : 'text-expense'}"
        >
          {deltaVal >= 0 ? '▲ +' : '▼ '}
          {formatMinorToDisplay(Math.abs(deltaVal), 'IDR')}
        </span>
      {/if}
    </div>
  </div>

  <!-- SVG Chart Body -->
  <div
    role="region"
    aria-label={i18n.t.trendChartAria}
    class="relative min-h-0 w-full flex-1 cursor-crosshair pt-2 select-none"
    onpointermove={handlePointerMove}
    onpointerleave={handlePointerLeave}
  >
    {#if loading}
      <div class="flex h-full w-full items-center justify-center">
        <span class="spinner-sm"></span>
      </div>
    {:else if points.length === 0}
      <div class="text-text-muted font-proto text-smaller flex h-full items-center justify-center">
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
          <linearGradient id="dashboardTrendGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--color-teal)" stop-opacity="0.18" />
            <stop offset="100%" stop-color="var(--color-teal)" stop-opacity="0.00" />
          </linearGradient>
        </defs>

        <!-- Horizontal Grid Lines -->
        <line
          x1={padX}
          y1={padY}
          x2={chartWidth - padX}
          y2={padY}
          stroke="var(--color-line)"
          stroke-dasharray="3,3"
          opacity="0.4"
        />
        <line
          x1={padX}
          y1={chartHeight / 2}
          x2={chartWidth - padX}
          y2={chartHeight / 2}
          stroke="var(--color-line)"
          stroke-dasharray="3,3"
          opacity="0.4"
        />
        <line
          x1={padX}
          y1={chartHeight - padY}
          x2={chartWidth - padX}
          y2={chartHeight - padY}
          stroke="var(--color-line)"
          stroke-dasharray="3,3"
          opacity="0.4"
        />

        <!-- Area Fill -->
        {#if areaD}
          {#key selectedMetric}
            <path d={areaD} fill="url(#dashboardTrendGradient)" class="animate-area-fade" />
          {/key}
        {/if}

        <!-- Trend Line -->
        {#if pathD}
          {#key selectedMetric}
            <path
              d={pathD}
              pathLength="100"
              fill="none"
              stroke="var(--color-teal)"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="animate-line-sweep"
            />
          {/key}
        {/if}

        <!-- Hover Indicator -->
        {#if activeHoverPoint}
          <line
            x1={activeHoverPoint.x}
            y1={padY}
            x2={activeHoverPoint.x}
            y2={chartHeight - padY / 2}
            stroke="var(--color-teal)"
            stroke-width="1"
            stroke-dasharray="2,2"
            opacity="0.8"
          />
          <circle
            cx={activeHoverPoint.x}
            cy={activeHoverPoint.y}
            r="4.5"
            fill="var(--color-teal)"
            stroke="var(--color-bg-app)"
            stroke-width="2"
          />
        {/if}
      </svg>
    {/if}
  </div>

  <!-- Footer Dates -->
  <div
    class="border-line/40 text-text-muted font-proto text-smaller flex shrink-0 items-center justify-between border-t pt-1 tabular-nums"
  >
    <span>{points[0]?.date ?? ''}</span>
    <span>{i18n.t.trendTrajectory30d}</span>
    <span>{points[points.length - 1]?.date ?? ''}</span>
  </div>
</div>
