<script lang="ts">
  import { reportState } from '../state/report.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatIDR } from '$lib/core/format/currency';
  import { KpiCard, AnimatedCounter, Card } from '$lib/components/ui';
  import { BarGroupChart, type BarGroupPoint } from '$lib/components/charts';

  let { from, to }: { from?: string; to?: string } = $props();

  $effect(() => {
    reportState.loadProfitLoss(from || undefined, to || undefined);
    reportState.loadMonthlyCashflow(from || undefined, to || undefined);
  });

  const barData = $derived.by<BarGroupPoint[]>(() => {
    const monthly = reportState.monthlyCashflow ?? [];
    if (monthly.length > 0) {
      return monthly.map((m) => ({
        label: m.month.slice(0, 7), // 'YYYY-MM'
        income: m.income,
        expense: m.expense,
      }));
    }
    return [];
  });

  const pnl = $derived(reportState.profitLoss);
  const totalIncome = $derived(pnl?.total_income ?? 0);
  const totalExpenses = $derived(pnl?.total_expenses ?? 0);
  const netIncome = $derived(pnl?.net_income ?? 0);
  const savingsRate = $derived(
    totalIncome > 0 ? Math.max(0, (netIncome / totalIncome) * 100).toFixed(1) : '0.0'
  );
</script>

<div class="flex min-h-0 flex-1 flex-col gap-4">
  <div class="grid shrink-0 grid-cols-2 gap-4 sm:grid-cols-4">
    <KpiCard label={i18n.t.revenues} labelClass="text-income">
      <AnimatedCounter
        value={totalIncome}
        currency="IDR"
        class="text-medium text-income block leading-tight font-bold"
      />
    </KpiCard>
    <KpiCard label={i18n.t.operationalExpenses} labelClass="text-expense">
      <AnimatedCounter
        value={totalExpenses}
        currency="IDR"
        class="text-medium text-expense block leading-tight font-bold"
      />
    </KpiCard>
    <KpiCard
      label={i18n.t.netIncome}
      labelClass={netIncome >= 0 ? 'text-text-white' : 'text-expense'}
      badge={netIncome >= 0 ? i18n.t.statusOk : i18n.t.statusErr}
      badgeTone={netIncome >= 0 ? 'ok' : 'err'}
    >
      <AnimatedCounter
        value={netIncome ?? 0}
        currency="IDR"
        class="text-medium block leading-tight font-bold {netIncome < 0
          ? 'text-expense'
          : 'text-text-white'}"
      />
    </KpiCard>
    <KpiCard label={i18n.t.savingsRateLabel || 'SAVINGS RATE'}>
      <span class="text-medium text-teal font-proto block leading-tight font-bold tabular-nums">
        {savingsRate}%
      </span>
    </KpiCard>
  </div>

  <Card title={i18n.t.incomeExpChartTitle} class="min-h-0 flex-1">
    {#if barData.length === 0}
      <div
        class="text-text-dim font-proto text-small border-line flex h-full min-h-50 w-full items-center justify-center border border-dashed"
      >
        {i18n.t.noHistoricalData}
      </div>
    {:else}
      <BarGroupChart
        data={barData}
        formatFn={(v) => formatIDR(v)}
        class="h-full w-full pt-4 pb-2"
      />
    {/if}
  </Card>
</div>
