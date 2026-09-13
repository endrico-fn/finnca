<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { calculatePlanProgress } from '$lib/accounting/finance';
  import DebtSimulator from './DebtSimulator.svelte';
  import { buildAsOfVault } from '$lib/accounting/reports/statements';
  import { KpiCard, Card, Badge, EmptyState } from '$lib/components/ui';

  let { asOf = '' }: { asOf?: string } = $props();

  const filteredVault = $derived(ledger.data ? buildAsOfVault(ledger.data, asOf) : null);

  const fmt = (n: number) => n.toLocaleString('en-US');

  // Debt & Receivables Advanced Report Analysis
  const debtReportStats = $derived.by(() => {
    if (!filteredVault) {
      return {
        plans: [],
        receivableTotal: 0,
        payableTotal: 0,
        receivableRemaining: 0,
        payableRemaining: 0,
        settledCount: 0,
        activeCount: 0,
      };
    }

    const plansWithProgress = ledger.plans.map((p) => calculatePlanProgress(p, filteredVault));
    const receivableTotal = plansWithProgress
      .filter((p) => p.plan.type === 'RECEIVABLE')
      .reduce((a, c) => a + c.plan.totalAmount, 0);
    const payableTotal = plansWithProgress
      .filter((p) => p.plan.type === 'PAYABLE')
      .reduce((a, c) => a + c.plan.totalAmount, 0);
    const receivableRemaining = plansWithProgress
      .filter((p) => p.plan.type === 'RECEIVABLE')
      .reduce((a, c) => a + c.remainingAmount, 0);
    const payableRemaining = plansWithProgress
      .filter((p) => p.plan.type === 'PAYABLE')
      .reduce((a, c) => a + c.remainingAmount, 0);
    const settledCount = plansWithProgress.filter((p) => p.isSettled).length;
    const activeCount = plansWithProgress.filter((p) => !p.isSettled).length;

    return {
      plans: plansWithProgress,
      receivableTotal,
      payableTotal,
      receivableRemaining,
      payableRemaining,
      settledCount,
      activeCount,
    };
  });

  function planTypeLabel(t: string): string {
    return t === 'RECEIVABLE' ? i18n.t.planFilterReceivable : i18n.t.planFilterPayable;
  }

  function planFreqLabel(f: string): string {
    if (f === 'DAILY') return i18n.t.planFreqDailyOpt;
    if (f === 'WEEKLY') return i18n.t.planFreqWeeklyOpt;
    return i18n.t.planFreqMonthlyOpt;
  }
</script>

<div class="flex min-h-0 flex-1 flex-col space-y-2">
  <!-- TOP SUMMARY KPI CARDS -->
  <div class="grid shrink-0 grid-cols-4 gap-2">
    <KpiCard
      label={i18n.t.totalScheduledReceivables}
      labelClass="text-income"
      value={`Rp ${fmt(debtReportStats.receivableTotal)}`}
      valueClass="text-income"
      subValue={`${i18n.t.remainingReceivables}: Rp ${fmt(debtReportStats.receivableRemaining)}`}
    />

    <KpiCard
      label={i18n.t.totalDebtPayables}
      labelClass="text-expense"
      value={`Rp ${fmt(debtReportStats.payableTotal)}`}
      valueClass="text-expense"
      subValue={`${i18n.t.remainingPayables}: Rp ${fmt(debtReportStats.payableRemaining)}`}
    />

    <KpiCard label={i18n.t.commitmentStatus}>
      <span class="font-proto flex items-baseline gap-2 leading-tight font-bold tabular-nums">
        <span class="text-text-strong text-large">{debtReportStats.activeCount}</span>
        <span class="text-text-dim text-medium">/</span>
        <span class="text-income text-large">{debtReportStats.settledCount}</span>
      </span>
      <span
        class="font-proto text-text-dim text-smaller flex items-center gap-1.5 leading-tight uppercase"
      >
        <span>{i18n.t.activeStatus}</span>
        <span>/</span>
        <span>{i18n.t.settledStatus}</span>
      </span>
    </KpiCard>

    <KpiCard
      label={i18n.t.netCommitmentBalance}
      labelClass="text-teal"
      value={`${debtReportStats.receivableRemaining >= debtReportStats.payableRemaining ? '+' : ''}Rp ${fmt(debtReportStats.receivableRemaining - debtReportStats.payableRemaining)}`}
      valueClass={debtReportStats.receivableRemaining >= debtReportStats.payableRemaining
        ? 'text-income'
        : 'text-expense'}
      subValue={debtReportStats.receivableRemaining >= debtReportStats.payableRemaining
        ? i18n.t.netReceivableSurplus
        : i18n.t.netPayableDeficit}
    />
  </div>

  <!-- DETAILED AMORTIZATION & COMMITMENT LIST -->
  <Card divided title={i18n.t.amortizationSchedule} class="flex-1">
    {#snippet header()}
      <span class="font-proto text-text-muted text-smaller uppercase">
        {debtReportStats.plans.length}
        {i18n.t.registeredCommitments}
      </span>
    {/snippet}

    <div class="flex-1 space-y-2 pr-1">
      {#if debtReportStats.plans.length === 0}
        <EmptyState title={i18n.t.noCommitmentsRecorded} />
      {:else}
        {#each debtReportStats.plans as item (item.plan.id)}
          {@const p = item.plan}
          <div
            class="bg-bg-app border-line space-y-2 border px-3 pt-2 pb-2.5 {item.isSettled
              ? 'opacity-65'
              : ''}"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Badge tone={p.type === 'RECEIVABLE' ? 'ok' : 'err'}>
                  {planTypeLabel(p.type)} • {planFreqLabel(p.frequency)}
                </Badge>
                <span class="font-proto text-text-strong text-medium font-bold">{p.title}</span>
              </div>
              {#if item.isSettled}
                <Badge tone="ok">
                  {i18n.t.settled100Percent}
                </Badge>
              {:else}
                <span
                  class="font-proto text-small font-bold tabular-nums {p.type === 'RECEIVABLE'
                    ? 'text-income'
                    : 'text-expense'}"
                >
                  {i18n.t.remainingLabel}: Rp {fmt(item.remainingAmount)}
                </span>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="space-y-1">
              <div class="text-text-dim text-smaller flex justify-between">
                <span>{i18n.t.paidLabel}: Rp {fmt(item.paidAmount)} / Rp {fmt(p.totalAmount)}</span>
                <span
                  >{item.progressPercent}% ({i18n.t.planCountShort.replace(
                    '{n}',
                    String(item.installmentsPaidCount)
                  )}
                  {i18n.t.installmentsCountLabel})</span
                >
              </div>
              <div class="bg-bg-card border-line h-2 w-full overflow-hidden border">
                <div
                  class="h-full transition-all {p.type === 'RECEIVABLE'
                    ? 'bg-income'
                    : 'bg-expense'}"
                  style="width: {item.progressPercent}%"
                ></div>
              </div>
            </div>

            <div
              class="text-text-muted border-line/40 text-smaller flex items-center justify-between border-t pt-1"
            >
              <span>
                {i18n.t.planTargetPer}:
                <strong class="text-text-strong">Rp {fmt(p.installmentAmount)}</strong>
              </span>
              {#if p.dueDate}
                <span class="text-text-dim">
                  {i18n.t.date}: {p.dueDate}
                </span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </Card>

  <div class="shrink-0">
    <DebtSimulator />
  </div>
</div>
