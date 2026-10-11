<script lang="ts">
  export interface DonutSlice {
    id: string;
    label: string;
    value: number;
    color?: string;
  }

  let {
    data = [],
    size = 140,
    strokeWidth = 20,
    centerLabel = '',
    centerValue = '',
    interactive = false,
    animate = true,
    onSliceHover,
    onSliceClick,
    class: className = '',
    title = '',
  }: {
    data: DonutSlice[];
    size?: number;
    strokeWidth?: number;
    centerLabel?: string;
    centerValue?: string;
    interactive?: boolean;
    animate?: boolean;
    onSliceHover?: (slice: DonutSlice | null) => void;
    onSliceClick?: (slice: DonutSlice) => void;
    class?: string;
    title?: string;
  } = $props();

  const DEFAULT_COLORS = [
    'var(--color-teal)',
    'var(--color-income)',
    'var(--color-expense)',
    'var(--color-warning)',
    'var(--color-text-dim)',
    'var(--color-text-muted)',
  ];

  const total = $derived.by(() => {
    return data.reduce((sum, d) => sum + Math.max(0, d.value), 0);
  });

  const cx = $derived(size / 2);
  const cy = $derived(size / 2);
  const radius = $derived((size - strokeWidth) / 2);
  const circumference = $derived(2 * Math.PI * radius);

  const slices = $derived.by(() => {
    if (total <= 0) return [];
    let accumulated = 0;
    return data
      .filter((d) => d.value > 0)
      .map((d, idx) => {
        const ratio = d.value / total;
        const strokeLength = ratio * circumference;
        const dashOffset = -accumulated;
        const startAngle = (accumulated / circumference) * 2 * Math.PI - Math.PI / 2;
        accumulated += strokeLength;
        const endAngle = (accumulated / circumference) * 2 * Math.PI - Math.PI / 2;
        return {
          ...d,
          color: d.color || DEFAULT_COLORS[idx % DEFAULT_COLORS.length],
          dashArray: `${strokeLength.toFixed(1)} ${(circumference - strokeLength).toFixed(1)}`,
          dashOffset: dashOffset.toFixed(1),
          percent: Math.round(ratio * 100),
          startAngle,
          endAngle,
          animDelay: idx * 80,
        };
      });
  });

  function sectorPath(startAngle: number, endAngle: number): string {
    const r = radius + strokeWidth / 2;
    const x1 = cx + r * Math.cos(startAngle);
    const y1 = cy + r * Math.sin(startAngle);
    const x2 = cx + r * Math.cos(endAngle);
    const y2 = cy + r * Math.sin(endAngle);
    const largeArc = endAngle - startAngle > Math.PI ? 1 : 0;
    const ri = radius - strokeWidth / 2;
    const xi1 = cx + ri * Math.cos(endAngle);
    const yi1 = cy + ri * Math.sin(endAngle);
    const xi2 = cx + ri * Math.cos(startAngle);
    const yi2 = cy + ri * Math.sin(startAngle);
    return [
      `M ${x1.toFixed(2)} ${y1.toFixed(2)}`,
      `A ${r.toFixed(2)} ${r.toFixed(2)} 0 ${largeArc} 1 ${x2.toFixed(2)} ${y2.toFixed(2)}`,
      `L ${xi1.toFixed(2)} ${yi1.toFixed(2)}`,
      `A ${ri.toFixed(2)} ${ri.toFixed(2)} 0 ${largeArc} 0 ${xi2.toFixed(2)} ${yi2.toFixed(2)}`,
      'Z',
    ].join(' ');
  }

  let hoveredSliceId = $state<string | null>(null);

  function handleSliceEnter(slice: DonutSlice) {
    if (!interactive) return;
    hoveredSliceId = slice.id;
    onSliceHover?.(slice);
  }

  function handleSliceLeave() {
    if (!interactive) return;
    hoveredSliceId = null;
    onSliceHover?.(null);
  }

  function handleSliceClick(slice: DonutSlice) {
    if (!interactive) return;
    onSliceClick?.(slice);
  }
  const chartAriaLabel = $derived(title || centerLabel || 'Donut chart allocation breakdown');
</script>

<div
  class="relative inline-flex items-center justify-center {className}"
  style="width: {size}px; height: {size}px"
>
  <svg
    role="img"
    aria-label={chartAriaLabel}
    width={size}
    height={size}
    viewBox="0 0 {size} {size}"
    class="-rotate-90"
  >
    <title>{chartAriaLabel}</title>
    <circle
      {cx}
      {cy}
      r={radius}
      fill="none"
      stroke="var(--color-line)"
      stroke-width={strokeWidth}
      opacity="0.3"
    />

    {#each slices as slice (slice.id)}
      {@const isHovered = hoveredSliceId === slice.id}
      <circle
        {cx}
        {cy}
        r={radius}
        fill="none"
        stroke={slice.color}
        stroke-width={isHovered ? strokeWidth + 4 : strokeWidth}
        stroke-dasharray={slice.dashArray}
        stroke-dashoffset={slice.dashOffset}
        stroke-linecap="butt"
        class={animate ? 'anim-donut-draw' : ''}
        style={animate ? `animation-delay: ${slice.animDelay}ms` : ''}
        opacity={interactive && hoveredSliceId !== null && !isHovered ? 0.5 : 1}
      >
        <title>{slice.label}: {slice.percent}%</title>
      </circle>
    {/each}

    {#if interactive}
      {#each slices as slice (slice.id + '-hit')}
        <path
          d={sectorPath(slice.startAngle, slice.endAngle)}
          fill="transparent"
          stroke="none"
          class="cursor-pointer"
          onmouseenter={() => handleSliceEnter(slice)}
          onmouseleave={handleSliceLeave}
          onclick={() => handleSliceClick(slice)}
          role="button"
          aria-label="{slice.label}: {slice.percent}%"
          tabindex="0"
          onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSliceClick(slice)}
        />
      {/each}
    {/if}
  </svg>

  {#if centerValue || centerLabel}
    <div class="pointer-events-none absolute flex flex-col items-center justify-center text-center">
      {#if centerValue}
        <span class="font-proto text-smaller text-text-strong font-bold tabular-nums">
          {centerValue}
        </span>
      {/if}
      {#if centerLabel}
        <span class="font-proto text-smaller text-text-dim tracking-wider uppercase">
          {centerLabel}
        </span>
      {/if}
    </div>
  {/if}
</div>
