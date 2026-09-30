<script lang="ts">
  import { formatIDR } from '$lib/core/format/currency';
  import { ChartFrame } from '$lib/components/charts';

  export interface FlowBar {
    date: string;
    value: number;
  }

  let {
    bars,
    chartWidth = 900,
    chartHeight = 240,
    padX = 16,
    padY = 24,
    hoveredIndex = null,
    hasHistoricalData = false,
    onHoverChange,
  }: {
    bars: FlowBar[];
    chartWidth?: number;
    chartHeight?: number;
    padX?: number;
    padY?: number;
    hoveredIndex?: number | null;
    hasHistoricalData: boolean;
    onHoverChange: (index: number | null) => void;
  } = $props();

  const maxAbs = $derived(Math.max(1, ...bars.map((b) => Math.abs(b.value))));
  const plotH = $derived(chartHeight - padY * 2);
  const zeroY = $derived.by(() => {
    const max = maxAbs;
    const min = -maxAbs;
    const ratio = (0 - min) / (max - min || 1);
    return padY + (1 - ratio) * plotH;
  });

  function barGeom(i: number) {
    const slotW = (chartWidth - padX * 2) / Math.max(1, bars.length);
    const w = Math.max(2, slotW - 2);
    const x = padX + i * slotW + (slotW - w) / 2;
    const h =
      bars[i].value === 0 ? 0 : Math.max(2, (Math.abs(bars[i].value) / maxAbs) * (plotH / 2 - 2));
    const y = bars[i].value >= 0 ? zeroY - h : zeroY;
    return { x, y, w, h };
  }
</script>

<ChartFrame
  width={chartWidth}
  height={chartHeight}
  count={bars.length}
  isEmpty={!hasHistoricalData}
  {onHoverChange}
>
  {#snippet body()}
    <line
      x1={padX}
      y1={zeroY}
      x2={chartWidth - padX}
      y2={zeroY}
      stroke="var(--color-line)"
      stroke-width="1"
    />

    {#each bars as bar, i (bar.date)}
      {@const g = barGeom(i)}
      {@const hovered = hoveredIndex === i}
      {#if hovered}
        <!-- Column illumination band -->
        <rect
          x={g.x - 2}
          y={padY}
          width={g.w + 4}
          height={chartHeight - padY * 2}
          fill="var(--color-teal)"
          opacity="0.06"
          class="pointer-events-none"
        />
        <!-- Vertical guide line -->
        <line
          x1={g.x + g.w / 2}
          y1={padY}
          x2={g.x + g.w / 2}
          y2={chartHeight - padY}
          stroke="var(--color-teal)"
          stroke-width="1"
          stroke-dasharray="2,2"
          opacity="0.7"
          class="pointer-events-none"
        />
      {/if}

      <g opacity={hoveredIndex === null ? 0.85 : hovered ? 1 : 0.3} class="transition-opacity">
        <title>{bar.date} — {formatIDR(bar.value)}</title>
        <rect
          x={g.x}
          y={g.y}
          width={g.w}
          height={Math.max(bar.value === 0 ? 1 : 3, g.h)}
          fill={bar.value >= 0 ? 'var(--color-income)' : 'var(--color-expense)'}
          stroke={hovered ? 'var(--color-text-white)' : 'none'}
          stroke-width={hovered ? 1.5 : 0}
        />
      </g>
    {/each}
  {/snippet}

  {#snippet overlay()}
    {#if hoveredIndex !== null && bars[hoveredIndex]}
      {@const hb = bars[hoveredIndex]}
      {@const hg = barGeom(hoveredIndex)}
      {@const tipW = 148}
      {@const tipH = 36}
      {@const rawTipX = hg.x + hg.w / 2 - tipW / 2}
      {@const tipX = Math.max(padX, Math.min(chartWidth - padX - tipW, rawTipX))}
      {@const anchorY = hb.value >= 0 ? hg.y : zeroY + Math.max(3, hg.h)}
      {@const tipY =
        hb.value >= 0
          ? anchorY >= padY + tipH + 10
            ? anchorY - tipH - 8
            : anchorY + hg.h + 10
          : anchorY + tipH + 8 <= chartHeight - padY
            ? anchorY + 8
            : zeroY - tipH - 10}

      <!-- Tip anchor pip at the bar edge -->
      <circle
        cx={hg.x + hg.w / 2}
        cy={anchorY}
        r="3"
        fill={hb.value >= 0 ? 'var(--color-income)' : 'var(--color-expense)'}
        stroke="var(--color-bg-app)"
        stroke-width="1.5"
        class="pointer-events-none"
      />

      <!-- Floating HUD box -->
      <g class="pointer-events-none select-none">
        <rect
          x={tipX}
          y={tipY}
          width={tipW}
          height={tipH}
          fill="var(--color-bg-card)"
          stroke="var(--color-line)"
          stroke-width="1"
        />
        <!-- Top accent indicator -->
        <line
          x1={tipX}
          y1={tipY}
          x2={tipX + tipW}
          y2={tipY}
          stroke={hb.value >= 0 ? 'var(--color-income)' : 'var(--color-expense)'}
          stroke-width="2"
        />
        <text
          x={tipX + tipW / 2}
          y={tipY + 13}
          fill="var(--color-text-dim)"
          font-size="9"
          font-family="var(--font-proto)"
          font-weight="normal"
          letter-spacing="0.05em"
          text-anchor="middle"
        >
          {hb.date}
        </text>
        <text
          x={tipX + tipW / 2}
          y={tipY + 27}
          fill={hb.value >= 0 ? 'var(--color-income)' : 'var(--color-expense)'}
          font-size="11"
          font-family="var(--font-proto)"
          font-weight="bold"
          text-anchor="middle"
        >
          {hb.value > 0 ? '+' : ''}{formatIDR(hb.value)}
        </text>
      </g>
    {/if}
  {/snippet}
</ChartFrame>
