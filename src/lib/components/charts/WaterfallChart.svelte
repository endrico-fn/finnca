<script lang="ts">
  import {
    calculateWaterfallLayout,
    type WaterfallStepInput,
    type WaterfallBarComputed,
  } from './chartMath';

  let {
    steps,
    width = 800,
    height = 280,
    class: className = '',
  }: {
    steps: WaterfallStepInput[];
    width?: number;
    height?: number;
    class?: string;
  } = $props();

  let hoveredBar = $state<WaterfallBarComputed | null>(null);

  const layout = $derived(calculateWaterfallLayout(steps, width, height, 32, 32, 44, 52));
</script>

<div class="relative w-full select-none {className}" style="min-height: 240px;">
  <svg
    viewBox="0 0 {layout.width} {layout.height}"
    class="h-full w-full overflow-visible"
    preserveAspectRatio="xMidYMid meet"
  >
    <!-- Zero baseline -->
    <line
      x1="20"
      y1={layout.zeroY}
      x2={layout.width - 20}
      y2={layout.zeroY}
      stroke="var(--color-line)"
      stroke-width="1"
      stroke-dasharray="4 4"
    />

    <!-- Step connectors -->
    {#each layout.connectors as conn, idx (idx)}
      <line
        x1={conn.x1}
        y1={conn.y1}
        x2={conn.x2}
        y2={conn.y2}
        stroke="var(--color-text-dim)"
        stroke-width="1"
        stroke-dasharray="2 2"
        opacity="0.6"
      />
    {/each}

    <!-- Waterfall bars -->
    {#each layout.bars as bar (bar.id)}
      {@const isHovered = hoveredBar?.id === bar.id}
      {@const isPositive = bar.amount >= 0}
      <!-- Value label Y position with bounds clamping to avoid clipping -->
      {@const labelY = Math.max(22, bar.y - 8)}
      {@const percentY = Math.max(10, bar.y - 20)}
      <!-- Bottom label Y position -->
      {@const catLabelY = layout.height - 18}

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <g
        class="cursor-pointer transition-opacity"
        onmouseenter={() => (hoveredBar = bar)}
        onmouseleave={() => (hoveredBar = null)}
      >
        <title
          >{bar.label}: {bar.formattedAmount}{bar.percentage ? ` (${bar.percentage})` : ''}</title
        >
        <!-- Bar rect -->
        <rect
          x={bar.x}
          y={bar.y}
          width={bar.width}
          height={bar.height}
          fill={bar.color}
          opacity={isHovered ? 1 : 0.85}
          class="transition-all duration-150"
        />

        <!-- Sharp border highlight on hover -->
        {#if isHovered}
          <rect
            x={bar.x - 1}
            y={bar.y - 1}
            width={bar.width + 2}
            height={bar.height + 2}
            fill="none"
            stroke="var(--color-text-white)"
            stroke-width="1.5"
          />
        {/if}

        <!-- Percentage label above bar -->
        {#if bar.percentage}
          <text
            x={bar.x + bar.width / 2}
            y={percentY}
            text-anchor="middle"
            fill="var(--color-text-dim)"
            class="font-proto text-smaller tabular-nums"
          >
            {bar.percentage}
          </text>
        {/if}

        <!-- Amount text above bar -->
        <text
          x={bar.x + bar.width / 2}
          y={labelY}
          text-anchor="middle"
          fill={bar.tone === 'expense'
            ? 'var(--color-expense)'
            : bar.tone === 'income'
              ? 'var(--color-income)'
              : 'var(--color-text-white)'}
          class="font-proto text-smaller font-bold tabular-nums"
        >
          {bar.formattedAmount}
        </text>

        <!-- Category label below -->
        <text
          x={bar.x + bar.width / 2}
          y={catLabelY}
          text-anchor="middle"
          fill={isHovered ? 'var(--color-text-white)' : 'var(--color-text-base)'}
          class="font-proto text-smaller tracking-wider uppercase"
        >
          {bar.label}
        </text>

        <!-- Direction indicator (+ / -) for intermediate bars when tall enough -->
        {#if !bar.isTotal && bar.height >= 18}
          <text
            x={bar.x + bar.width / 2}
            y={bar.y + bar.height / 2 + 4}
            text-anchor="middle"
            fill="var(--color-bg-app)"
            class="font-proto text-smaller font-bold select-none"
          >
            {isPositive ? '+' : '−'}
          </text>
        {/if}
      </g>
    {/each}
  </svg>
</div>
