<script lang="ts">
  import { calculateSankeyLayout, type SankeyNodeInput, type SankeyLinkInput } from './chartMath';

  let {
    nodes = [],
    links = [],
    width = 800,
    height = 460,
    colWidth = 180,
    padding = 12,
    formatValue = (v: number) => String(v),
    class: className = '',
  }: {
    nodes: SankeyNodeInput[];
    links: SankeyLinkInput[];
    width?: number;
    height?: number;
    colWidth?: number;
    padding?: number;
    formatValue?: (v: number) => string;
    class?: string;
  } = $props();

  const layout = $derived(calculateSankeyLayout(nodes, links, width, height, colWidth, padding));

  const maxLinkValue = $derived(Math.max(1, ...links.map((l) => l.value)));
</script>

<div class="relative w-full overflow-auto {className}">
  {#if layout.nodes.length === 0}
    <div
      class="border-line bg-bg-app text-text-dim font-proto text-small flex h-72 w-full items-center justify-center border border-dashed"
    >
      —
    </div>
  {:else}
    <svg
      width="100%"
      height="100%"
      viewBox="0 0 {layout.width} {layout.actualHeight}"
      preserveAspectRatio="xMidYMid meet"
      class="min-h-110 overflow-visible"
    >
      {#each layout.links as link (link.source + '-' + link.target)}
        <path
          d={link.path}
          fill={link.color}
          opacity={(0.1 + (0.3 * link.value) / maxLinkValue).toFixed(2)}
          class="transition-opacity hover:opacity-50"
        >
          <title>{formatValue(link.value)}</title>
        </path>
      {/each}

      {#each layout.nodes as node (node.id)}
        {@const x =
          node.column === 0
            ? 0
            : node.column === 1
              ? layout.width / 2 - layout.colWidth / 2
              : layout.width - layout.colWidth}
        <g transform="translate({x}, {node.y})">
          <title>{node.label} — {formatValue(node.value)}</title>
          <rect
            x={node.column === 0 ? layout.colWidth - 6 : 0}
            y="0"
            width="6"
            height={node.height}
            fill={node.color}
          />

          <text
            x={node.column === 0
              ? layout.colWidth - 12
              : node.column === 1
                ? layout.colWidth / 2
                : 10}
            y={Math.max(12, node.height / 2 + 4)}
            text-anchor={node.column === 0 ? 'end' : node.column === 1 ? 'middle' : 'start'}
            fill="var(--color-text-strong)"
            font-size="11"
            font-family="var(--font-proto)"
            font-weight="bold"
            class="pointer-events-none select-none"
          >
            {node.label}
          </text>

          {#if node.height > 20}
            <text
              x={node.column === 0
                ? layout.colWidth - 12
                : node.column === 1
                  ? layout.colWidth / 2
                  : 10}
              y={Math.max(24, node.height / 2 + 16)}
              text-anchor={node.column === 0 ? 'end' : node.column === 1 ? 'middle' : 'start'}
              fill="var(--color-text-dim)"
              font-size="9"
              font-family="var(--font-proto)"
              class="pointer-events-none select-none"
            >
              {formatValue(node.value)}
            </text>
          {/if}
        </g>
      {/each}
    </svg>
  {/if}
</div>
