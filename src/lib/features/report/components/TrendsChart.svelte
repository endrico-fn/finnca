<script lang="ts">
  import { formatIDR } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { ChartFrame } from '$lib/components/charts';
  import type { ChartCoordsResult, ChartPoint } from '../state/trendsChartUtils';

  let {
    chartCoords,
    chartWidth = 900,
    chartHeight = 240,
    padX = 16,
    neonColor,
    zeroLineY = null,
    hoveredCoord = null,
    hasHistoricalData = false,
    onHoverChange,
  }: {
    chartCoords: ChartCoordsResult;
    chartWidth?: number;
    chartHeight?: number;
    padX?: number;
    neonColor: string;
    zeroLineY?: number | null;
    hoveredCoord?: ChartPoint | null;
    hasHistoricalData: boolean;
    onHoverChange: (index: number | null) => void;
  } = $props();
</script>

<ChartFrame
  width={chartWidth}
  height={chartHeight}
  count={chartCoords.points.length}
  isEmpty={!hasHistoricalData}
  {onHoverChange}
>
  {#snippet body()}
    <defs>
      <linearGradient id="chartFillGrad" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color={neonColor} stop-opacity="0.16" />
        <stop offset="100%" stop-color={neonColor} stop-opacity="0.00" />
      </linearGradient>
    </defs>

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

    <path d={chartCoords.areaD} fill="url(#chartFillGrad)" />

    <path
      d={chartCoords.pathD}
      fill="none"
      stroke={neonColor}
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    {#if !hoveredCoord && chartCoords.points.length > 3 && chartCoords.peakPoint && chartCoords.troughPoint && chartCoords.peakPoint.val !== chartCoords.troughPoint.val}
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
        font-size="10"
        font-family="var(--font-proto)"
        font-weight="bold"
        text-anchor="middle"
        opacity="0.85"
      >
        {i18n.t.trendsPeakMarker}
      </text>

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
        font-size="10"
        font-family="var(--font-proto)"
        font-weight="bold"
        text-anchor="middle"
        opacity="0.85"
      >
        {i18n.t.trendsTroughMarker}
      </text>
    {/if}
  {/snippet}

  {#snippet overlay()}
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

      {@const tipW = 105}
      {@const tipH = 22}
      {@const tipX = Math.max(padX, Math.min(chartWidth - padX - tipW, hoveredCoord.x - tipW / 2))}
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
          font-size="10"
          font-family="var(--font-proto)"
          font-weight="bold"
          text-anchor="middle"
        >
          {formatIDR(hoveredCoord.val)}
        </text>
      </g>
    {/if}
  {/snippet}
</ChartFrame>
