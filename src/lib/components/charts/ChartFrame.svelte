<script lang="ts">
  import type { Snippet } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    width = 900,
    height = 240,
    count,
    isEmpty,
    emptyLabel,
    ariaLabel,
    body,
    overlay,
    onHoverChange,
  }: {
    width?: number;
    height?: number;
    count: number;
    isEmpty: boolean;
    emptyLabel?: string;
    ariaLabel?: string;
    body: Snippet;
    overlay?: Snippet;
    onHoverChange: (index: number | null) => void;
  } = $props();

  let svgEl = $state<SVGSVGElement | null>(null);

  function handlePointerMove(e: PointerEvent) {
    if (!svgEl || count <= 0) return;
    const rect = svgEl.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    onHoverChange(Math.round(ratio * (count - 1)));
  }
</script>

<div
  role="region"
  aria-label={ariaLabel ?? i18n.t.trendsChartAria}
  class="relative mt-2 h-56 w-full cursor-crosshair select-none"
  onpointermove={handlePointerMove}
  onpointerleave={() => onHoverChange(null)}
>
  {#if isEmpty}
    <div
      class="text-text-dim border-line font-proto text-small flex h-full w-full items-center justify-center border border-dashed"
    >
      {emptyLabel ?? i18n.t.noHistoricalData}
    </div>
  {:else}
    <svg
      bind:this={svgEl}
      viewBox="0 0 {width} {height}"
      preserveAspectRatio="none"
      class="h-full w-full overflow-visible"
    >
      {@render body()}
      {@render overlay?.()}
    </svg>
  {/if}
</div>
