<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    fromMinor,
    formatIDR,
    convertMinor,
    buildChildrenMap,
    accountBalanceMinorFiltered,
  } from '$lib/accounting/finance';
  import { EmptyState, Card } from '$lib/components/ui';

  let { from, to } = $props<{ from: string; to: string }>();

  // Removed unused pnl derived

  const childrenMap = $derived(ledger.data ? buildChildrenMap(ledger.data.accounts) : new Map());

  const incomeAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'INCOME' && !a.placeholder)
  );

  const expenseAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'EXPENSE' && !a.placeholder)
  );

  // Build nodes and links for Sankey
  interface SankeyNode {
    id: string;
    label: string;
    value: number;
    color: string;
    column: number; // 0 = left, 1 = middle, 2 = right
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
    if (!ledger.data) return { nodes: [], links: [] };

    let nodes: SankeyNode[] = [];
    let links: SankeyLink[] = [];

    let totalIncome = 0;
    let totalExpense = 0;

    // Income Nodes (Column 0)
    for (const a of incomeAccounts) {
      const bal = accountBalanceMinorFiltered(
        a.id,
        ledger.data,
        from || undefined,
        to || undefined,
        childrenMap
      );
      if (bal !== 0) {
        const val = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        totalIncome += val;
        nodes.push({
          id: a.id,
          label: a.name,
          value: val,
          color: 'var(--color-income)',
          column: 0,
        });
        links.push({
          source: a.id,
          target: 'CENTER',
          value: val,
          color: 'var(--color-income)',
        });
      }
    }

    // Expense Nodes (Column 2)
    for (const a of expenseAccounts) {
      const bal = accountBalanceMinorFiltered(
        a.id,
        ledger.data,
        from || undefined,
        to || undefined,
        childrenMap
      );
      if (bal !== 0) {
        const val = convertMinor(Math.abs(bal), a.currency, 'IDR', ledger.fxRate);
        totalExpense += val;
        nodes.push({
          id: a.id,
          label: a.name,
          value: val,
          color: 'var(--color-expense)',
          column: 2,
        });
        links.push({
          source: 'CENTER',
          target: a.id,
          value: val,
          color: 'var(--color-expense)',
        });
      }
    }

    if (totalIncome === 0 && totalExpense === 0) {
      return { nodes: [], links: [] };
    }

    const net = totalIncome - totalExpense;

    // Center Node (Column 1)
    const centerVal = Math.max(totalIncome, totalExpense);
    nodes.push({
      id: 'CENTER',
      label: i18n.t.cashflowCenterNode,
      value: centerVal,
      color: 'var(--color-text-strong)',
      column: 1,
    });

    // Balancing Nodes
    if (net > 0) {
      // Net Savings (Column 2)
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
      // Net Deficit (Column 0)
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

    // --- Layout Algorithm ---
    const width = 800;
    const height = 500;
    const colWidth = 200;
    const padding = 15;

    // Group nodes by column
    const cols = [[], [], []] as SankeyNode[][];
    nodes.forEach((n) => {
      cols[n.column].push(n);
    });

    // Find max column total to calculate scale
    const colTotals = cols.map((col) => col.reduce((sum, n) => sum + n.value, 0));
    const maxTotal = Math.max(...colTotals);

    // Available height = total height - (padding * (num_nodes - 1))
    // We must ensure we don't get negative available height if there are many nodes.
    // Let's cap minimum height and scale up SVG viewBox if needed.
    const maxNodesInCol = Math.max(...cols.map((c) => c.length));
    const minRequiredHeight = maxNodesInCol * 20 + (maxNodesInCol - 1) * padding;
    const actualHeight = Math.max(height, minRequiredHeight);

    // Recalculate scale per column so they fill the height appropriately,
    // or just use a global scale. A standard Sankey uses a global scale.
    // Global scale: pixels per 1 unit of value
    let scale = (actualHeight - (maxNodesInCol - 1) * padding) / maxTotal;
    // Prevent Infinity if maxTotal is 0
    if (!isFinite(scale) || scale <= 0) scale = 1;

    // Position Nodes
    cols.forEach((colNodes, colIdx) => {
      // Sort nodes by value descending for better flow
      colNodes.sort((a, b) => b.value - a.value);

      let yOffset = 0;
      // If it's the center node, center it vertically
      if (colIdx === 1 && colNodes.length === 1) {
        const h = colNodes[0].value * scale;
        yOffset = (actualHeight - h) / 2;
      } else {
        // Center the whole column block vertically
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

    // Position Links (Bezier curves)
    // We need to track the current y-offset on the left and right side of each node
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
            <title>{formatIDR(fromMinor('IDR', link.value))}</title>
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
              <tspan>{formatIDR(fromMinor('IDR', node.value))}</tspan>
            </text>
          </g>
        {/each}
      </svg>
    </div>
  {/if}
</Card>
