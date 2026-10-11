<script lang="ts">
  import { buildMonotonePath } from './chartMath';

  let {
    data = [],
    color = 'var(--color-teal)',
    fill = true,
    width = 120,
    height = 36,
    class: className = '',
  }: {
    data: number[];
    color?: string;
    fill?: boolean;
    width?: number;
    height?: number;
    class?: string;
  } = $props();

  const PAD_Y = 4;

  const gradientId = $derived('spark-' + color.replace(/[^a-zA-Z0-9]/g, '').slice(-12));

  const points = $derived.by(() => {
    if (data.length < 2) {
      return [
        { x: 0, y: height / 2 },
        { x: width, y: height / 2 },
      ];
    }
    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min || 1;
    const usableH = height - PAD_Y * 2;
    return data.map((v, i) => ({
      x: (i / (data.length - 1)) * width,
      y: PAD_Y + ((max - v) / range) * usableH,
    }));
  });

  const linePath = $derived(buildMonotonePath(points));

  const fillPath = $derived(
    linePath +
      ` L ${points[points.length - 1].x.toFixed(1)} ${height} L ${points[0].x.toFixed(1)} ${height} Z`
  );
</script>

<div class="inline-block {className}" style="width:{width}px;height:{height}px">
  <svg
    viewBox="0 0 {width} {height}"
    preserveAspectRatio="none"
    {width}
    {height}
    aria-hidden="true"
  >
    {#if fill}
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color={color} stop-opacity="0.22" />
          <stop offset="100%" stop-color={color} stop-opacity="0" />
        </linearGradient>
      </defs>
      <path d={fillPath} fill="url(#{gradientId})" stroke="none" />
    {/if}
    <path
      d={linePath}
      fill="none"
      stroke={color}
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
  </svg>
</div>
