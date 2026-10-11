<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { formatDateToDisplay } from '$lib/core/format/date';
  import { reportState } from '../state/report.svelte';
  import {
    runCashForecastSimulation,
    computeForecastGeometry,
    type HorizonDays,
  } from '../state/forecastEngine';
  import {
    listAccountsCmd,
    listPlansWithProgressCmd,
    type AccountBalanceView,
    type PlanProgressView,
  } from '$lib/core/ipc/bindings';
  import { Card, Badge, EmptyState, KpiCard, AnimatedCounter, Icon } from '$lib/components/ui';

  let selectedHorizon = $state<HorizonDays>(60);
  let accounts = $state<AccountBalanceView[]>([]);
  let planItems = $state<PlanProgressView[]>([]);
  let hoveredPoint = $state<{
    date: string;
    balance: number;
    delta: number;
    x: number;
    y: number;
  } | null>(null);

  onMount(async () => {
    try {
      const [accs, plans] = await Promise.all([
        listAccountsCmd().catch(() => []),
        listPlansWithProgressCmd().catch(() => []),
      ]);
      accounts = accs;
      planItems = plans;
    } catch {
      accounts = [];
      planItems = [];
    }
  });

  const liquidCash = $derived.by(() => {
    const assetAccounts = accounts.filter(
      (a) =>
        a.account.account_type === 'ASSET' &&
        (a.account.code.startsWith('10') ||
          a.account.code.startsWith('11') ||
          a.account.code.startsWith('1'))
    );
    if (assetAccounts.length > 0) {
      return assetAccounts.reduce(
        (sum, a) => sum + (a.recursive_balance ?? a.direct_balance ?? 0),
        0
      );
    }
    const bsAssets = reportState.balanceSheet?.total_assets ?? 0;
    if (bsAssets > 0) return bsAssets;
    const hist = reportState.historicalPoints;
    return hist.length > 0 ? hist[hist.length - 1].liquidCash : 0;
  });

  const dailyBaselineBurn = $derived.by(() => {
    const pnl = reportState.profitLoss;
    if (pnl && pnl.total_expenses > 0) {
      return Math.round(pnl.total_expenses / 30);
    }
    const bsLiab = reportState.balanceSheet?.total_liabilities ?? 0;
    if (bsLiab > 0) {
      return Math.round(bsLiab / 90);
    }
    return 0;
  });

  const simulation = $derived(
    runCashForecastSimulation(liquidCash, dailyBaselineBurn, planItems, selectedHorizon)
  );

  const chartGeometry = $derived(computeForecastGeometry(simulation.points));

  function handleChartMouseMove(e: MouseEvent) {
    if (!chartGeometry || chartGeometry.coords.length === 0) return;
    const svg = e.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const mouseX = ((e.clientX - rect.left) / rect.width) * chartGeometry.width;

    let closest = chartGeometry.coords[0];
    let minDist = Infinity;
    for (const c of chartGeometry.coords) {
      const dist = Math.abs(c.x - mouseX);
      if (dist < minDist) {
        minDist = dist;
        closest = c;
      }
    }

    hoveredPoint = {
      date: closest.date,
      balance: closest.balance,
      delta: closest.balance - liquidCash,
      x: closest.x,
      y: closest.y,
    };
  }

  function handleChartMouseLeave() {
    hoveredPoint = null;
  }

  const horizonOptions: HorizonDays[] = [30, 60, 90, 180];
</script>

<div class="flex flex-col gap-4 p-4">
  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
    <KpiCard label={i18n.t.currentLiquidCash} labelClass="text-teal">
      <AnimatedCounter
        value={liquidCash}
        currency="IDR"
        class="text-medium text-text-white font-proto block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate uppercase">
        {i18n.t.rollupBalance}
      </span>
    </KpiCard>

    <KpiCard
      label={i18n.t.projectedRunway}
      labelClass={simulation.depletionDay !== null ? 'text-expense' : 'text-income'}
    >
      {#if simulation.depletionDay !== null}
        <span class="text-medium text-expense font-proto block leading-tight font-bold">
          {simulation.depletionDay} DAYS
        </span>
        <span class="text-expense/80 font-proto text-smaller block truncate uppercase">
          {i18n.t.runwayDepletionDate}: {simulation.depletionDate
            ? formatDateToDisplay(simulation.depletionDate)
            : '—'}
        </span>
      {:else}
        <span class="text-medium text-income font-proto block leading-tight font-bold">
          {i18n.t.sustainableRunway}
        </span>
        <span class="text-text-dim font-proto text-smaller block truncate uppercase">
          &gt; {selectedHorizon} DAYS RESERVE
        </span>
      {/if}
    </KpiCard>

    <KpiCard label={i18n.t.expectedInflows} labelClass="text-income">
      <AnimatedCounter
        value={simulation.totalInflow}
        currency="IDR"
        class="text-medium text-income font-proto block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate uppercase">
        {simulation.commitments.filter((c) => c.isInflow).length}
        {i18n.t.upcomingCommitments}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.expectedOutflows} labelClass="text-expense">
      <AnimatedCounter
        value={simulation.totalOutflow}
        currency="IDR"
        class="text-medium text-expense font-proto block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate uppercase">
        {formatMinorToDisplay(dailyBaselineBurn, 'IDR')} / DAY
      </span>
    </KpiCard>
  </div>

  {#if simulation.depletionDay !== null}
    <div
      class="border-expense bg-expense/10 text-expense flex items-center justify-between border p-3"
    >
      <div class="flex items-center gap-2">
        <span class="text-expense shrink-0">
          <Icon name="alert" size={16} />
        </span>
        <span class="font-proto text-smaller font-bold tracking-wider">
          {i18n.t.criticalDepletionWarning}:
        </span>
        <span class="font-aux text-smaller">
          {i18n.t.runwayDepletionDate}
          {simulation.depletionDate ? formatDateToDisplay(simulation.depletionDate) : ''} ({i18n.t.daysRemainingLabel.replace(
            '{days}',
            String(simulation.depletionDay)
          )}).
        </span>
      </div>
      <Badge tone="err" size="s">{i18n.t.badgeUrgent}</Badge>
    </div>
  {/if}

  <Card class="flex flex-col gap-3 p-4">
    <div class="border-line flex flex-wrap items-center justify-between gap-2 border-b pb-3">
      <div class="flex items-center gap-2">
        <span class="text-text-white font-proto text-small font-bold tracking-wider">
          {i18n.t.cashForecastTitle}
        </span>
        <span class="text-text-muted font-proto text-smaller">
          {selectedHorizon}D
        </span>
      </div>

      <div class="flex items-center gap-3">
        <div
          class="font-proto text-smaller bg-bg-card-subtle border-line flex items-center gap-2 border px-3 py-1"
        >
          {#if hoveredPoint}
            <span class="text-text-dim uppercase">{i18n.t.date}:</span>
            <span class="text-text-white font-bold">{formatDateToDisplay(hoveredPoint.date)}</span>
            <span class="text-line">|</span>
            <span class="text-text-dim uppercase">{i18n.t.colBalance}:</span>
            <span class="font-bold {hoveredPoint.balance >= 0 ? 'text-teal' : 'text-expense'}">
              {formatMinorToDisplay(hoveredPoint.balance, 'IDR')}
            </span>
            <span class="text-line">|</span>
            <span class="text-text-dim uppercase">Δ:</span>
            <span class="font-bold {hoveredPoint.delta >= 0 ? 'text-income' : 'text-expense'}">
              {hoveredPoint.delta >= 0 ? '+' : ''}{formatMinorToDisplay(hoveredPoint.delta, 'IDR')}
            </span>
          {:else}
            <span class="text-text-dim uppercase">{i18n.t.totals}:</span>
            <span
              class="font-bold {simulation.endingBalance >= liquidCash
                ? 'text-income'
                : 'text-expense'}"
            >
              {simulation.endingBalance >= liquidCash ? '+' : ''}{formatMinorToDisplay(
                simulation.endingBalance - liquidCash,
                'IDR'
              )}
            </span>
          {/if}
        </div>

        <div class="border-line flex border">
          {#each horizonOptions as h (h)}
            <button
              type="button"
              onclick={() => (selectedHorizon = h)}
              class="font-proto text-smaller px-3 py-1 transition-colors {selectedHorizon === h
                ? 'bg-teal text-bg-app font-bold'
                : 'text-text-dim hover:text-text-white hover:bg-bg-btn'}"
            >
              {h}D
            </button>
          {/each}
        </div>
      </div>
    </div>

    {#if chartGeometry}
      <div class="relative w-full overflow-hidden">
        <svg
          aria-hidden="true"
          viewBox={`0 0 ${chartGeometry.width} ${chartGeometry.height}`}
          class="h-56 w-full cursor-crosshair select-none"
          onmousemove={handleChartMouseMove}
          onmouseleave={handleChartMouseLeave}
        >
          <defs>
            <linearGradient id="forecastTealGradient" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" stop-color="var(--color-teal)" stop-opacity="0.25" />
              <stop offset="100%" stop-color="var(--color-teal)" stop-opacity="0.0" />
            </linearGradient>
          </defs>

          <line
            x1={chartGeometry.padding.left}
            y1={chartGeometry.padding.top}
            x2={chartGeometry.width - chartGeometry.padding.right}
            y2={chartGeometry.padding.top}
            stroke="var(--color-line)"
            stroke-width="1"
            stroke-dasharray="2,2"
          />
          <line
            x1={chartGeometry.padding.left}
            y1={chartGeometry.height - chartGeometry.padding.bottom}
            x2={chartGeometry.width - chartGeometry.padding.right}
            y2={chartGeometry.height - chartGeometry.padding.bottom}
            stroke="var(--color-line)"
            stroke-width="1"
          />

          {#if chartGeometry.hasZeroLine}
            <line
              x1={chartGeometry.padding.left}
              y1={chartGeometry.zeroY}
              x2={chartGeometry.width - chartGeometry.padding.right}
              y2={chartGeometry.zeroY}
              stroke="var(--color-expense)"
              stroke-width="1.5"
              stroke-dasharray="4,4"
            />
            <text
              x={chartGeometry.width - chartGeometry.padding.right + 4}
              y={chartGeometry.zeroY + 3}
              fill="var(--color-expense)"
              class="font-proto text-smaller"
            >
              0
            </text>
          {/if}

          {#if chartGeometry.areaPath}
            <path d={chartGeometry.areaPath} fill="url(#forecastTealGradient)" />
          {/if}

          <path
            d={chartGeometry.pathData}
            fill="none"
            stroke="var(--color-teal)"
            stroke-width="2"
            stroke-linejoin="miter"
            stroke-linecap="square"
          />

          {#if hoveredPoint}
            <line
              x1={hoveredPoint.x}
              y1={chartGeometry.padding.top}
              x2={hoveredPoint.x}
              y2={chartGeometry.height - chartGeometry.padding.bottom}
              stroke="var(--color-text-dim)"
              stroke-width="1"
              stroke-dasharray="3,3"
            />
            <circle
              cx={hoveredPoint.x}
              cy={hoveredPoint.y}
              r="4"
              fill="var(--color-teal)"
              stroke="var(--color-bg-app)"
              stroke-width="2"
            />
          {/if}

          <text
            x={chartGeometry.padding.left}
            y={chartGeometry.height - 8}
            fill="var(--color-text-dim)"
            class="font-proto text-smaller"
          >
            {formatDateToDisplay(chartGeometry.coords[0].date)}
          </text>
          <text
            x={chartGeometry.width - chartGeometry.padding.right}
            y={chartGeometry.height - 8}
            text-anchor="end"
            fill="var(--color-text-dim)"
            class="font-proto text-smaller"
          >
            {formatDateToDisplay(chartGeometry.coords[chartGeometry.coords.length - 1].date)}
          </text>
        </svg>
      </div>
    {/if}
  </Card>

  <Card class="flex flex-col p-4">
    <div class="border-line flex items-center justify-between border-b pb-3">
      <div class="flex items-center gap-2">
        <span class="text-text-white font-proto text-small font-bold tracking-wider">
          {i18n.t.upcomingCommitments}
        </span>
        <Badge tone="neutral" size="s">
          {simulation.commitments.length}
        </Badge>
      </div>
      <span class="text-text-dim font-proto text-smaller">
        {i18n.t.horizonSelector}: {selectedHorizon}D
      </span>
    </div>

    {#if simulation.commitments.length === 0}
      <div class="py-8">
        <EmptyState title={i18n.t.noScheduledCommitments} icon="calendar" />
      </div>
    {:else}
      <div class="divide-line divide-y overflow-x-auto">
        <table class="font-proto text-smaller w-full text-left">
          <thead>
            <tr class="text-text-dim uppercase">
              <th class="py-2 pr-4">{i18n.t.date}</th>
              <th class="py-2 pr-4">{i18n.t.description}</th>
              <th class="py-2 pr-4">{i18n.t.type}</th>
              <th class="py-2 text-right">{i18n.t.colBalance}</th>
            </tr>
          </thead>
          <tbody class="divide-line/60 divide-y">
            {#each simulation.commitments as c, idx (`${c.date}-${c.title}-${idx}`)}
              <tr class="hover:bg-bg-btn/30 transition-colors">
                <td class="text-text-dim py-2 pr-4">{formatDateToDisplay(c.date)}</td>
                <td class="text-text-white font-aux py-2 pr-4">{c.title}</td>
                <td class="py-2 pr-4">
                  <Badge tone={c.isInflow ? 'ok' : 'neutral'} size="s">
                    {c.planType}
                  </Badge>
                </td>
                <td class="py-2 text-right font-bold {c.isInflow ? 'text-income' : 'text-expense'}">
                  {c.isInflow ? '+' : '-'}{formatMinorToDisplay(c.amount, 'IDR')}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </Card>
</div>
