<script lang="ts">
  import { reportState } from '../state/report.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatIDR } from '$lib/core/format/currency';
  import { EmptyState, Card } from '$lib/components/ui';

  let { from, to } = $props<{ from: string; to: string }>();

  $effect(() => {
    reportState.loadProfitLoss(from || undefined, to || undefined);
  });

  interface SankeyNode {
    id: string;
    label: string;
    value: number;
    color: string;
    column: number;
    y?: number;
    height?: number;
  }

  interface SankeyLink {
    source: string;
    target: string;
    value: number;
    color: string;
    path?: string;
  }

  const sankeyData = $derived.by(() => {
    const pnl = reportState.profitLoss;
    if (!pnl) return { nodes: [], links: [] };

    let nodes: SankeyNode[] = [];
    let links: SankeyLink[] = [];

    const totalIncome = pnl.total_income;
    const totalExpense = pnl.total_expenses;

    for (const row of pnl.income_rows) {
      if (row.amount > 0) {
        nodes.push({
          id: row.account_id,
          label: row.name,
          value: row.amount,
          color: 'var(--color-income)',
          column: 0,
        });
        links.push({
          source: row.account_id,
          target: 'CENTER',
          value: row.amount,
          color: 'var(--color-income)',
        });
      }
    }

    for (const row of pnl.expense_rows) {
      if (row.amount > 0) {
        nodes.push({
          id: row.account_id,
          label: row.name,
          value: row.amount,
          color: 'var(--color-expense)',
          column: 2,
        });
        links.push({
          source: 'CENTER',
          target: row.account_id,
          value: row.amount,
          color: 'var(--color-expense)',
        });
      }
    }

    if (totalIncome === 0 && totalExpense === 0) {
      return { nodes: [], links: [] };
    }

    const net = totalIncome - totalExpense;
    const centerVal = Math.max(totalIncome, totalExpense);

    nodes.push({
      id: 'CENTER',
      label: i18n.t.cashflowCenterNode,
      value: centerVal,
      color: 'var(--color-text-strong)',
      column: 1,
    });

    if (net > 0) {
      nodes.push({
        id: 'NET',
        label: i18n.t.cashflowNetSavings,
        value: net,
        color: 'var(--color-teal)',
        column: 2,
      });
      links.push({
        source: 'CENTER',
        target: 'NET',
        value: net,
        color: 'var(--color-teal)',
      });
    } else if (net < 0) {
      nodes.push({
        id: 'NET',
        label: i18n.t.cashflowNetDeficit,
        value: Math.abs(net),
        color: 'var(--color-expense)',
        column: 0,
      });
      links.push({
        source: 'NET',
        target: 'CENTER',
        value: Math.abs(net),
        color: 'var(--color-expense)',
      });
    }

    const width = 800;
    const height = 500;
    const colWidth = 200;
    const padding = 15;

    const cols = [[], [], []] as SankeyNode[][];
    nodes.forEach((n) => {
      cols[n.column].push(n);
    });

    const colTotals = cols.map((col) => col.reduce((sum, n) => sum + n.value, 0));
    const maxTotal = Math.max(...colTotals);

    const maxNodesInCol = Math.max(...cols.map((c) => c.length));
    const minRequiredHeight = maxNodesInCol * 20 + (maxNodesInCol - 1) * padding;
    const actualHeight = Math.max(height, minRequiredHeight);

    let scale = (actualHeight - (maxNodesInCol - 1) * padding) / maxTotal;
    if (!isFinite(scale) || scale <= 0) scale = 1;

    cols.forEach((colNodes, colIdx) => {
      colNodes.sort((a, b) => b.value - a.value);

      let yOffset = 0;
      if (colIdx === 1 && colNodes.length === 1) {
        const h = colNodes[0].value * scale;
        yOffset = (actualHeight - h) / 2;
      } else {
        const colTotalHeight =
          colNodes.reduce((sum, n) => sum + n.value * scale, 0) + (colNodes.length - 1) * padding;
        yOffset = (actualHeight - colTotalHeight) / 2;
      }

      colNodes.forEach((node) => {
        node.height = node.value * scale;
        node.y = yOffset;
        yOffset += node.height + padding;
      });
    });

    const nodeOffsets: Record<string, { left: number; right: number }> = {};
    nodes.forEach((n) => {
      nodeOffsets[n.id] = { left: n.y!, right: n.y! };
    });

    links.forEach((link) => {
      const sourceNode = nodes.find((n) => n.id === link.source)!;
      const targetNode = nodes.find((n) => n.id === link.target)!;

      const linkHeight = link.value * scale;

      const startX = sourceNode.column === 0 ? colWidth : width / 2 + colWidth / 2;
      const startY = nodeOffsets[sourceNode.id].right;
      nodeOffsets[sourceNode.id].right += linkHeight;

      const endX = targetNode.column === 2 ? width - colWidth : width / 2 - colWidth / 2;
      const endY = nodeOffsets[targetNode.id].left;
      nodeOffsets[targetNode.id].left += linkHeight;

      const curvature = 0.5;
      const xOffset = (endX - startX) * curvature;

      link.path = `M ${startX} ${startY} C ${startX + xOffset} ${startY}, ${endX - xOffset} ${endY}, ${endX} ${endY} L ${endX} ${endY + linkHeight} C ${endX - xOffset} ${endY + linkHeight}, ${startX + xOffset} ${startY + linkHeight}, ${startX} ${startY + linkHeight} Z`;
    });

    return { nodes, links, actualHeight, width, colWidth };
  });
</script>

<Card divided title={i18n.t.cashflowChartTitle} class="flex-1">
  {#if sankeyData.nodes.length === 0}
    <div class="border-line bg-bg-app flex min-h-0 flex-1 items-center justify-center border">
      <EmptyState
        title={i18n.t.noCashflowDataTitle}
        hint={i18n.t.noCashflowDataHint}
        icon="chart"
      />
    </div>
  {:else}
    <div class="border-line bg-bg-app relative min-h-0 flex-1 overflow-auto border">
      <svg
        width="100%"
        height="100%"
        viewBox="0 0 {sankeyData.width!} {sankeyData.actualHeight!}"
        preserveAspectRatio="xMidYMid meet"
        class="min-h-125"
      >
        <!-- Links -->
        {#each sankeyData.links as link (link.source + '-' + link.target)}
          <path
            d={link.path}
            fill={link.color}
            opacity="0.15"
            class="transition-opacity hover:opacity-40"
          >
            <title>{formatIDR(link.value)}</title>
          </path>
        {/each}

        <!-- Nodes -->
        {#each sankeyData.nodes as node (node.id || node)}
          {@const x =
            node.column === 0
              ? 0
              : node.column === 1
                ? sankeyData.width! / 2 - sankeyData.colWidth! / 2
                : sankeyData.width! - sankeyData.colWidth!}
          <g transform="translate({x}, {node.y})">
            <!-- Node Bar -->
            <rect
              x={node.column === 0 ? sankeyData.colWidth! - 4 : 0}
              y="0"
              width="4"
              height={node.height}
              fill={node.color}
            />

            <!-- Node Label Area -->
            <rect
              x={node.column === 0 ? 0 : 4}
              y="0"
              width={sankeyData.colWidth! - 4}
              height={node.height}
              fill="transparent"
            />

            <text
              x={node.column === 0 ? sankeyData.colWidth! - 12 : 12}
              y={Math.max(14, (node.height ?? 0) / 2 + 4)}
              fill="var(--color-text-strong)"
              font-size="10"
              font-family="var(--font-proto)"
              text-anchor={node.column === 0 ? 'end' : 'start'}
              class="pointer-events-none"
            >
              <tspan class="font-bold"
                >{node.label.length > 20 ? node.label.substring(0, 20) + '...' : node.label}</tspan
              >
            </text>
            <text
              x={node.column === 0 ? sankeyData.colWidth! - 12 : 12}
              y={Math.max(14, (node.height ?? 0) / 2 + 4) + 14}
              fill="var(--color-text-muted)"
              font-size="10"
              font-family="var(--font-proto)"
              text-anchor={node.column === 0 ? 'end' : 'start'}
              class="pointer-events-none"
            >
              <tspan>{formatIDR(node.value)}</tspan>
            </text>
          </g>
        {/each}
      </svg>
    </div>
  {/if}
</Card>
