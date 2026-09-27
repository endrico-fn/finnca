<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import {
    Badge,
    Card,
    KpiCard,
    AnimatedCounter,
    Tabs,
    SearchBar,
    EmptyState,
    Button,
    Icon,
  } from '$lib/components/ui';
  import { SankeyDiagram, type SankeyNodeInput, type SankeyLinkInput } from '$lib/components/charts';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number) => formatMinorToDisplay(n, 'IDR');

  function formatSigned(n: number): string {
    if (n > 0) return `+${fmt(n)}`;
    if (n < 0) return `-${fmt(Math.abs(n))}`;
    return fmt(0);
  }

  let { from = '', to = '' }: { from?: string; to?: string } = $props();

  let activeActivityTab = $state<'all' | 'operating' | 'investing' | 'financing'>('all');
  let searchQuery = $state('');
  let directionFilter = $state<'all' | 'inflow' | 'outflow'>('all');
  let showTrend = $state(false);
  let viewMode = $state<'table' | 'sankey'>('table');

  $effect(() => {
    reportState.loadCashFlow(from || undefined, to || undefined);
  });

  const cf = $derived(reportState.cashFlow);
  const monthlyPoints = $derived(reportState.monthlyCashflow);

  const startingCash = $derived(cf?.starting_cash ?? 0);
  const operatingCash = $derived(cf?.operating_cash_flow ?? 0);
  const investingCash = $derived(cf?.investing_cash_flow ?? 0);
  const financingCash = $derived(cf?.financing_cash_flow ?? 0);
  const netCashChange = $derived(cf?.net_cash_change ?? 0);
  const endingCash = $derived(cf?.ending_cash ?? 0);

  const operatingRows = $derived(cf?.operating_rows ?? []);
  const investingRows = $derived(cf?.investing_rows ?? []);
  const financingRows = $derived(cf?.financing_rows ?? []);

  const allRows = $derived.by(() => {
    const combined = [
      ...operatingRows.map((r) => ({ ...r, section: 'operating' as const })),
      ...investingRows.map((r) => ({ ...r, section: 'investing' as const })),
      ...financingRows.map((r) => ({ ...r, section: 'financing' as const })),
    ];
    return combined.sort((a, b) => b.date.localeCompare(a.date));
  });

  const countAll = $derived(allRows.length);
  const countOperating = $derived(operatingRows.length);
  const countInvesting = $derived(investingRows.length);
  const countFinancing = $derived(financingRows.length);

  const displayedRows = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return allRows.filter((r) => {
      if (activeActivityTab !== 'all' && r.section !== activeActivityTab) {
        return false;
      }
      if (directionFilter === 'inflow' && r.amount < 0) {
        return false;
      }
      if (directionFilter === 'outflow' && r.amount >= 0) {
        return false;
      }
      if (q) {
        const matchDesc = r.description.toLowerCase().includes(q);
        const matchAcc = r.account_name.toLowerCase().includes(q);
        const matchCat = r.category.toLowerCase().includes(q);
        const matchDate = r.date.includes(q);
        if (!matchDesc && !matchAcc && !matchCat && !matchDate) {
          return false;
        }
      }
      return true;
    });
  });

  const maxMonthlyAbs = $derived.by(() => {
    if (monthlyPoints.length === 0) return 1;
    const max = Math.max(
      ...monthlyPoints.map((p) => Math.max(Math.abs(p.net), p.income, p.expense))
    );
    return max > 0 ? max : 1;
  });

  function getCategoryLabel(category: string): string {
    switch (category) {
      case 'OPERATING_INCOME':
      case 'INVESTING_INFLOW':
      case 'FINANCING_INFLOW':
        return i18n.t.cashIn;
      case 'OPERATING_EXPENSE':
      case 'INVESTING_OUTFLOW':
      case 'FINANCING_OUTFLOW':
        return i18n.t.cashOut;
      default:
        return category;
    }
  }

  function getBadgeTone(section: 'operating' | 'investing' | 'financing', amount: number) {
    if (amount >= 0) return 'ok';
    if (section === 'investing') return 'warn';
    return 'err';
  }

  const sankeyData = $derived.by(() => {
    if (allRows.length === 0) return { nodes: [] as SankeyNodeInput[], links: [] as SankeyLinkInput[] };

    const inflowsByCategory = new SvelteMap<string, number>();
    const outflowsByCategory = new SvelteMap<string, number>();
    const cashByAccount = new SvelteMap<string, { in: number; out: number }>();

    for (const r of allRows) {
      const acc = r.account_name || 'Cash';
      if (!cashByAccount.has(acc)) cashByAccount.set(acc, { in: 0, out: 0 });

      if (r.amount > 0) {
        const cat = getCategoryLabel(r.category);
        inflowsByCategory.set(cat, (inflowsByCategory.get(cat) ?? 0) + r.amount);
        cashByAccount.get(acc)!.in += r.amount;
      } else if (r.amount < 0) {
        const cat = getCategoryLabel(r.category);
        const absAmt = Math.abs(r.amount);
        outflowsByCategory.set(cat, (outflowsByCategory.get(cat) ?? 0) + absAmt);
        cashByAccount.get(acc)!.out += absAmt;
      }
    }

    const nodes: SankeyNodeInput[] = [];
    const links: SankeyLinkInput[] = [];

    for (const [cat, val] of inflowsByCategory.entries()) {
      nodes.push({ id: `in_${cat}`, label: cat, value: val, color: 'var(--color-income)', column: 0 });
    }

    for (const [acc, flow] of cashByAccount.entries()) {
      const totalFlow = flow.in + flow.out;
      if (totalFlow > 0) {
        nodes.push({ id: `acc_${acc}`, label: acc, value: totalFlow, color: 'var(--color-teal)', column: 1 });
      }
    }

    for (const [cat, val] of outflowsByCategory.entries()) {
      nodes.push({ id: `out_${cat}`, label: cat, value: val, color: 'var(--color-expense)', column: 2 });
    }

    const totalIn = Array.from(inflowsByCategory.values()).reduce((a, b) => a + b, 0);
    const totalOut = Array.from(outflowsByCategory.values()).reduce((a, b) => a + b, 0);

    for (const [cat, inVal] of inflowsByCategory.entries()) {
      for (const [acc, flow] of cashByAccount.entries()) {
        if (totalIn > 0 && flow.in > 0) {
          const linkVal = Math.round((inVal * flow.in) / totalIn);
          if (linkVal > 0) {
            links.push({
              source: `in_${cat}`,
              target: `acc_${acc}`,
              value: linkVal,
              color: 'var(--color-income)',
            });
          }
        }
      }
    }

    for (const [acc, flow] of cashByAccount.entries()) {
      for (const [cat, outVal] of outflowsByCategory.entries()) {
        if (totalOut > 0 && flow.out > 0) {
          const linkVal = Math.round((outVal * flow.out) / totalOut);
          if (linkVal > 0) {
            links.push({
              source: `acc_${acc}`,
              target: `out_${cat}`,
              value: linkVal,
              color: 'var(--color-expense)',
            });
          }
        }
      }
    }

    return { nodes, links };
  });
</script>

<div class="flex min-h-0 w-full flex-1 flex-col gap-3">
  <div class="grid shrink-0 grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-5">
    <KpiCard label={i18n.t.startingCash} subValue={i18n.t.startingCashDesc}>
      <AnimatedCounter
        value={startingCash}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
    </KpiCard>

    <KpiCard label={i18n.t.operatingCashFlow} subValue={i18n.t.operatingActivities}>
      <AnimatedCounter
        value={operatingCash}
        currency="IDR"
        formatFn={formatSigned}
        class="text-medium block leading-tight font-bold {operatingCash > 0
          ? 'text-income'
          : operatingCash < 0
            ? 'text-expense'
            : 'text-text-white'}"
      />
    </KpiCard>

    <KpiCard label={i18n.t.investingCashFlow} subValue={i18n.t.investingActivities}>
      <AnimatedCounter
        value={investingCash}
        currency="IDR"
        formatFn={formatSigned}
        class="text-medium block leading-tight font-bold {investingCash > 0
          ? 'text-income'
          : investingCash < 0
            ? 'text-expense'
            : 'text-text-white'}"
      />
    </KpiCard>

    <KpiCard label={i18n.t.financingCashFlow} subValue={i18n.t.financingActivities}>
      <AnimatedCounter
        value={financingCash}
        currency="IDR"
        formatFn={formatSigned}
        class="text-medium block leading-tight font-bold {financingCash > 0
          ? 'text-income'
          : financingCash < 0
            ? 'text-expense'
            : 'text-text-white'}"
      />
    </KpiCard>

    <KpiCard
      label={i18n.t.endingCash}
      subValue={`${i18n.t.netChange}: ${formatSigned(netCashChange)}`}
    >
      <AnimatedCounter
        value={endingCash}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
    </KpiCard>
  </div>

  {#if monthlyPoints.length > 0 && showTrend}
    <Card title={i18n.t.monthlyCashflowTrend} class="shrink-0 p-3">
      {#snippet actions()}
        <Button variant="ghost" size="sm" onclick={() => (showTrend = false)}>
          {i18n.t.hideTrend}
        </Button>
      {/snippet}
      <div class="flex flex-wrap gap-2.5 pt-1">
        {#each monthlyPoints as pt (pt.month)}
          <div
            class="border-line bg-bg-btn flex max-w-sm min-w-65 flex-1 flex-col justify-between border p-2.5"
          >
            <div class="flex items-center justify-between gap-2">
              <span class="font-proto text-text-white text-small font-bold">{pt.month}</span>
              <span
                class="font-proto text-small font-bold tabular-nums {pt.net > 0
                  ? 'text-income'
                  : pt.net < 0
                    ? 'text-expense'
                    : 'text-text-white'}"
              >
                {formatSigned(pt.net)}
              </span>
            </div>

            <div class="bg-bg-card border-line/50 my-2 flex h-1.5 w-full items-center border">
              {#if pt.net >= 0}
                <div
                  class="bg-income h-full transition-all duration-300"
                  style="width: {Math.max(
                    4,
                    Math.min(100, Math.round((pt.net / maxMonthlyAbs) * 100))
                  )}%"
                ></div>
              {:else}
                <div
                  class="bg-expense h-full transition-all duration-300"
                  style="width: {Math.max(
                    4,
                    Math.min(100, Math.round((Math.abs(pt.net) / maxMonthlyAbs) * 100))
                  )}%"
                ></div>
              {/if}
            </div>

            <div
              class="font-proto text-smaller text-text-dim flex items-center justify-between tabular-nums"
            >
              <span class="flex items-center gap-1">
                <span class="text-text-muted text-smaller tracking-wider uppercase"
                  >{i18n.t.cashIn}:</span
                >
                <span class="text-income font-medium">+{fmt(pt.income)}</span>
              </span>
              <span class="flex items-center gap-1">
                <span class="text-text-muted text-smaller tracking-wider uppercase"
                  >{i18n.t.cashOut}:</span
                >
                <span class="text-expense font-medium">-{fmt(pt.expense)}</span>
              </span>
            </div>
          </div>
        {/each}
      </div>
    </Card>
  {/if}

  <Card
    padding={false}
    divided={false}
    title={i18n.t.cashFlowTitle}
    count={displayedRows.length}
    class="flex min-h-0 flex-1 flex-col"
  >
    {#snippet header()}
      <div class="flex items-center gap-2">
        <Tabs
          variant="outline"
          tabs={[
            { id: 'all', label: `${i18n.t.allActivities} (${countAll})` },
            { id: 'operating', label: `${i18n.t.operatingActivities} (${countOperating})` },
            { id: 'investing', label: `${i18n.t.investingActivities} (${countInvesting})` },
            { id: 'financing', label: `${i18n.t.financingActivities} (${countFinancing})` },
          ]}
          active={activeActivityTab}
          onSelect={(id) =>
            (activeActivityTab = id as 'all' | 'operating' | 'investing' | 'financing')}
        />
        {#if monthlyPoints.length > 0}
          <Button
            variant={showTrend ? 'secondary' : 'outline'}
            size="sm"
            onclick={() => (showTrend = !showTrend)}
            class="flex shrink-0 items-center gap-1.5"
          >
            <Icon name="chart" size={14} />
            {showTrend ? i18n.t.hideTrend : i18n.t.showTrend}
          </Button>
        {/if}
        <Button
          variant={viewMode === 'sankey' ? 'secondary' : 'outline'}
          size="sm"
          onclick={() => (viewMode = viewMode === 'sankey' ? 'table' : 'sankey')}
          class="flex shrink-0 items-center gap-1.5"
        >
          <Icon name="chart" size={14} />
          {viewMode === 'sankey' ? i18n.t.cashFlowTable : i18n.t.cashFlowDiagram}
        </Button>
      </div>
    {/snippet}

    {#if viewMode === 'sankey'}
      <div class="flex-1 overflow-auto p-4">
        {#if sankeyData.nodes.length === 0}
          <div class="flex h-72 items-center justify-center p-8">
            <EmptyState title={i18n.t.noCashFlowRecords} hint={i18n.t.adjustFilterHint} icon="chart" />
          </div>
        {:else}
          <SankeyDiagram
            nodes={sankeyData.nodes}
            links={sankeyData.links}
            formatValue={(v) => fmt(v)}
            class="w-full"
          />
        {/if}
      </div>
    {:else}
      <div
        class="border-line bg-bg-card flex shrink-0 items-center justify-between gap-3 border-b px-3 py-2"
      >
        <div class="flex max-w-md min-w-48 flex-1 items-center">
          <SearchBar bind:value={searchQuery} placeholder={i18n.t.searchPlaceholder} class="w-full" />
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <Tabs
            variant="segmented"
            tabs={[
              { id: 'all', label: i18n.t.filterAllLabel },
              { id: 'inflow', label: `+ ${i18n.t.cashIn}` },
              { id: 'outflow', label: `- ${i18n.t.cashOut}` },
            ]}
            active={directionFilter}
            onSelect={(id) => (directionFilter = id as 'all' | 'inflow' | 'outflow')}
          />
        </div>
      </div>

      {#if displayedRows.length === 0}
        <div class="flex flex-1 items-center justify-center p-8">
          <EmptyState title={i18n.t.noCashFlowRecords} hint={i18n.t.adjustFilterHint} icon="chart" />
        </div>
    {:else}
      <div class="min-h-0 flex-1 overflow-y-auto">
        <table class="sharp-table w-full">
          <thead>
            <tr>
              <th class="w-28 pl-3">{i18n.t.colDate}</th>
              <th class="w-32 px-2">{i18n.t.category}</th>
              <th class="min-w-50 px-3">{i18n.t.description}</th>
              <th class="w-56 px-3">{i18n.t.contraAccount}</th>
              <th class="numeric w-40 pr-3">{i18n.t.netChange}</th>
            </tr>
          </thead>
          <tbody>
            {#each displayedRows as row (row.date + row.description + row.amount + row.account_name)}
              <tr>
                <td class="font-proto text-text-muted text-smaller w-28 pl-3 whitespace-nowrap">
                  {row.date}
                </td>
                <td class="w-32 px-2 whitespace-nowrap">
                  <Badge size="s" tone={getBadgeTone(row.section, row.amount)}>
                    {getCategoryLabel(row.category)}
                  </Badge>
                </td>
                <td class="font-aux text-text-white text-small px-3">
                  {row.description}
                </td>
                <td class="font-aux text-text-dim text-small w-56 truncate px-3">
                  {row.account_name || '—'}
                </td>
                <td
                  class="font-proto text-small numeric w-40 pr-3 font-bold whitespace-nowrap tabular-nums {row.amount >
                  0
                    ? 'text-income'
                    : row.amount < 0
                      ? 'text-expense'
                      : 'text-text-white'}"
                >
                  {formatSigned(row.amount)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</Card>
</div>
