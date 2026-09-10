<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { calculatePlanProgress } from '$lib/accounting/finance';
  import DebtSimulator from './DebtSimulator.svelte';
  import { todayString } from '$lib/accounting/core/date';
  import { generateGhostTransactions } from '$lib/accounting/features/planning';
  import { KpiCard, ReportCard } from '$lib/components/ui';

  let { asOf = '' }: { asOf?: string } = $props();

  const filteredVault = $derived.by(() => {
    if (!ledger.data) return null;
    let ghostTxs: import('$lib/accounting/types').Transaction[] = [];
    const today = todayString();
    if (asOf && asOf > today) {
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 1);
      const tomorrowStr = todayString(tomorrow);
      ghostTxs = generateGhostTransactions(ledger.data, tomorrowStr, asOf);
    }
    return asOf
      ? { ...ledger.data, transactions: [...ledger.data.transactions.filter(t => t.date <= asOf), ...ghostTxs] }
      : ledger.data;
  });

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
</script>

<div class="flex min-h-0 flex-1 flex-col space-y-2 font-mono">
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

    <div class="sharp-card space-y-1 px-3 pt-2.5 pb-2.5">
      <span class="font-proto text-text-muted block text-[9px] tracking-wider uppercase leading-none">
        {i18n.t.commitmentStatus}
      </span>
      <div class="font-proto flex items-baseline gap-2 leading-tight">
        <span class="text-text-strong text-[18px] font-bold">{debtReportStats.activeCount}</span>
        <span class="text-text-dim text-[14px]">/</span>
        <span class="text-income text-[18px] font-bold">{debtReportStats.settledCount}</span>
      </div>
      <div class="font-proto text-text-dim flex items-center gap-1.5 text-[10px] uppercase leading-tight">
        <span>{i18n.t.activeStatus}</span>
        <span>/</span>
        <span>{i18n.t.settledStatus}</span>
      </div>
    </div>

    <KpiCard
      label={i18n.t.netCommitmentBalance}
      labelClass="text-teal"
      value={`${debtReportStats.receivableRemaining >= debtReportStats.payableRemaining ? '+' : ''}Rp ${fmt(debtReportStats.receivableRemaining - debtReportStats.payableRemaining)}`}
      valueClass={debtReportStats.receivableRemaining >= debtReportStats.payableRemaining ? 'text-income' : 'text-expense'}
      subValue={debtReportStats.receivableRemaining >= debtReportStats.payableRemaining ? i18n.t.netReceivableSurplus : i18n.t.netPayableDeficit}
    />
  </div>

  <!-- DETAILED AMORTIZATION & COMMITMENT LIST -->
  <ReportCard
    title={i18n.t.amortizationSchedule}
    class="flex-1 font-mono"
  >
    {#snippet headerRight()}
      <span class="font-proto text-text-muted text-[10px] uppercase">
        {debtReportStats.plans.length} {i18n.t.registeredCommitments}
      </span>
    {/snippet}

    <div class="flex-1 space-y-2 pr-1">
      {#if debtReportStats.plans.length === 0}
        <div class="text-text-dim border-line border border-dashed p-8 text-center text-[11px]">
          {i18n.t.noCommitmentsRecorded}
        </div>
      {:else}
        {#each debtReportStats.plans as item (item.plan.id)}
          {@const p = item.plan}
          <div
            class="bg-bg-app border-line space-y-2 border p-3 {item.isSettled ? 'opacity-65' : ''}"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span
                  class="px-1.5 py-0.5 text-[9px] font-bold uppercase {p.type === 'RECEIVABLE'
                    ? 'bg-bg-row-active text-income'
                    : 'bg-expense/10 text-expense'}"
                >
                  {p.type} • {p.frequency}
                </span>
                <span class="text-text-strong text-[13px] font-bold">{p.title}</span>
              </div>
              {#if item.isSettled}
                <span class="badge-ok px-2 py-0.5 text-[9px] font-bold">
                  {i18n.t.settled100Percent}
                </span>
              {:else}
                <span
                  class="text-[11px] font-bold {p.type === 'RECEIVABLE'
                    ? 'text-income'
                    : 'text-expense'}"
                >
                  {i18n.t.remainingLabel}: Rp {fmt(item.remainingAmount)}
                </span>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="space-y-1">
              <div class="text-text-dim flex justify-between text-[10px]">
                <span>{i18n.t.paidLabel}: Rp {fmt(item.paidAmount)} / Rp {fmt(p.totalAmount)}</span>
                <span
                  >{item.progressPercent}% ({item.installmentsPaidCount}x {i18n.t
                    .installmentsCountLabel})</span
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
              class="text-text-muted border-line/40 flex items-center justify-between border-t pt-1 text-[10px]"
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
  </ReportCard>

  <div class="shrink-0">
    <DebtSimulator />
  </div>
</div>
