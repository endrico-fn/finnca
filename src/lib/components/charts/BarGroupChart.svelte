<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { linearScale } from './chartMath';

  export interface BarGroupPoint {
    label: string;
    income: number;
    expense: number;
  }

  let {
    data = [],
    chartWidth = 900,
    chartHeight = 220,
    padX = 16,
    padY = 28,
    showSavingsLine = true,
    formatFn = (v: number) => v.toLocaleString(),
    class: className = '',
  }: {
    data: BarGroupPoint[];
    chartWidth?: number;
    chartHeight?: number;
    padX?: number;
    padY?: number;
    showSavingsLine?: boolean;
    formatFn?: (v: number) => string;
    class?: string;
  } = $props();

  let wrapperEl = $state<HTMLDivElement | undefined>(undefined);
  let svgEl = $state<SVGSVGElement | undefined>(undefined);
  let hoveredIndex = $state<number | null>(null);
  let tooltipX = $state(0);
  let tooltipY = $state(0);

  const plotW = $derived(chartWidth - padX * 2);
  const plotH = $derived(chartHeight - padY);
  const zeroY = $derived(chartHeight - padY);

  const hasData = $derived(data.length > 0 && data.some((d) => d.income > 0 || d.expense > 0));

  const maxVal = $derived.by(() => {
    if (!hasData) return 1;
    return Math.max(...data.map((d) => Math.max(d.income, d.expense)), 1);
  });

  const groupW = $derived(plotW / Math.max(data.length, 1));
  const barW = $derived(Math.max(2, (groupW - 4) / 2 - 1));

  const groups = $derived.by(() => {
    return data.map((d, i) => {
      const groupX = padX + i * groupW;
      const incomeH = linearScale(d.income, 0, maxVal, 0, plotH);
      const expenseH = linearScale(d.expense, 0, maxVal, 0, plotH);
      const savingsRate =
        d.income > 0 ? Math.max(0, Math.min(100, ((d.income - d.expense) / d.income) * 100)) : 0;
      return {
        ...d,
        index: i,
        groupX,
        centerX: groupX + groupW / 2,
        incomeX: groupX + groupW / 2 - barW - 1,
        expenseX: groupX + groupW / 2 + 1,
        incomeY: zeroY - incomeH,
        expenseY: zeroY - expenseH,
        incomeH,
        expenseH,
        net: d.income - d.expense,
        savingsRate,
      };
    });
  });

  const savingsLinePoints = $derived.by(() => {
    if (!showSavingsLine || groups.length < 2) return '';
    return groups
      .map((g, i) => {
        const y = linearScale(g.savingsRate, 0, 100, zeroY, padY / 2);
        return `${i === 0 ? 'M' : 'L'} ${g.centerX.toFixed(1)} ${y.toFixed(1)}`;
      })
      .join(' ');
  });

  function getSvgX(clientX: number): number {
    if (!svgEl) return 0;
    const rect = svgEl.getBoundingClientRect();
    return ((clientX - rect.left) / rect.width) * chartWidth;
  }

  function onpointermove(e: PointerEvent) {
    if (!wrapperEl || !data.length) return;
    const svgX = getSvgX(e.clientX);
    const idx = Math.floor((svgX - padX) / groupW);
    hoveredIndex = Math.max(0, Math.min(data.length - 1, idx));
    const wrapRect = wrapperEl.getBoundingClientRect();
    tooltipX = e.clientX - wrapRect.left + 12;
    tooltipY = e.clientY - wrapRect.top - 8;
  }

  function onpointerleave() {
    hoveredIndex = null;
  }
</script>

<div
  class="relative overflow-visible {className}"
  bind:this={wrapperEl}
  {onpointermove}
  {onpointerleave}
  role="img"
  aria-label="Bar group chart"
>
  {#if !hasData}
    <svg
      viewBox="0 0 {chartWidth} {chartHeight}"
      preserveAspectRatio="none"
      width="100%"
      height={chartHeight}
    >
      <rect
        x={padX}
        y={padY / 2}
        width={plotW}
        height={plotH}
        fill="none"
        stroke="var(--color-line)"
        stroke-dasharray="6,4"
        opacity="0.5"
      />
      <text
        x={chartWidth / 2}
        y={chartHeight / 2}
        text-anchor="middle"
        dominant-baseline="middle"
        fill="var(--color-text-muted)"
        font-family="var(--font-proto)"
        font-size="13">{i18n.t.noSpendingData}</text
      >
    </svg>
  {:else}
    <svg
      bind:this={svgEl}
      viewBox="0 0 {chartWidth} {chartHeight}"
      preserveAspectRatio="none"
      width="100%"
      height={chartHeight}
    >
      <line
        x1={padX}
        y1={zeroY}
        x2={chartWidth - padX}
        y2={zeroY}
        stroke="var(--color-line)"
        stroke-width="1"
        opacity="0.5"
      />

      {#each groups as g, i (g.label + i)}
        {@const isHovered = hoveredIndex === i}
        {@const dimmed = hoveredIndex !== null && !isHovered}

        <rect
          x={g.incomeX}
          y={g.incomeY}
          width={barW}
          height={g.incomeH}
          fill="var(--color-income)"
          opacity={dimmed ? 0.55 : 1}
          class="anim-bar-grow"
          style="animation-delay: {i * 30}ms; transform-origin: {g.incomeX + barW / 2}px {zeroY}px"
        />
        <rect
          x={g.expenseX}
          y={g.expenseY}
          width={barW}
          height={g.expenseH}
          fill="var(--color-expense)"
          opacity={dimmed ? 0.55 : 1}
          class="anim-bar-grow"
          style="animation-delay: {i * 30 + 15}ms; transform-origin: {g.expenseX +
            barW / 2}px {zeroY}px"
        />

        <text
          x={g.centerX}
          y={chartHeight - 6}
          text-anchor="middle"
          fill="var(--color-text-muted)"
          font-family="var(--font-proto)"
          font-size="10">{g.label}</text
        >
      {/each}

      {#if showSavingsLine && savingsLinePoints}
        <path
          d={savingsLinePoints}
          fill="none"
          stroke="var(--color-teal)"
          stroke-width="1.5"
          stroke-dasharray="3,3"
          opacity="0.85"
        />
      {/if}
    </svg>

    {#if hoveredIndex !== null}
      {@const g = groups[hoveredIndex]}
      <div
        class="pointer-events-none absolute z-[var(--z-tooltip)]"
        style="left:{tooltipX}px;top:{tooltipY}px"
      >
        <div class="font-proto border-line bg-bg-card text-smaller min-w-36 border px-2.5 py-2">
          <div class="text-text-dim mb-1 tracking-wide uppercase">{g.label}</div>
          <div class="text-income">+{formatFn(g.income)}</div>
          <div class="text-expense">-{formatFn(g.expense)}</div>
          <div class="border-line text-teal mt-1 border-t pt-1">
            {i18n.t.net}: {g.net >= 0 ? '+' : ''}{formatFn(g.net)}
          </div>
          <div class="text-text-muted">
            {i18n.t.savingsRate}: {g.savingsRate.toFixed(1)}%
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>
