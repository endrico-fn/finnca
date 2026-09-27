<script lang="ts">
  import {
    listPlansWithProgressCmd,
    listAccountsCmd,
    type PlanProgressViewDto,
    type AccountBalanceView,
  } from '$lib/core/ipc/bindings';
  import { onMount } from 'svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import DebtSimulator from './DebtSimulator.svelte';
  import {
    KpiCard,
    Card,
    Badge,
    EmptyState,
    ProgressBar,
    AnimatedCounter,
  } from '$lib/components/ui';

  let { asOf = '' }: { asOf?: string } = $props();

  let plans = $state<PlanProgressViewDto[]>([]);
  let accounts = $state<AccountBalanceView[]>([]);

  onMount(async () => {
    const [p, a] = await Promise.all([
      listPlansWithProgressCmd().catch(() => []),
      listAccountsCmd().catch(() => []),
    ]);
    plans = p;
    accounts = a;
  });

  const fmt = (n: number) => formatMinorToDisplay(n, 'IDR');

  function formatNetSigned(n: number): string {
    const prefix = n >= 0 ? '+' : '';
    return `${prefix}${fmt(n)}`;
  }

  const debtReportStats = $derived.by(() => {
    const plansWithProgress = asOf ? plans.filter((p) => p.plan.start_date <= asOf) : plans;
    const receivableTotal = plansWithProgress
      .filter((p) => p.plan.plan_type === 'RECEIVABLE')
      .reduce((a, c) => a + c.plan.total_amount, 0);
    const payableTotal = plansWithProgress
      .filter((p) => p.plan.plan_type === 'PAYABLE')
      .reduce((a, c) => a + c.plan.total_amount, 0);
    const receivableRemaining = plansWithProgress
      .filter((p) => p.plan.plan_type === 'RECEIVABLE')
      .reduce((a, c) => a + c.remaining_amount, 0);
    const payableRemaining = plansWithProgress
      .filter((p) => p.plan.plan_type === 'PAYABLE')
      .reduce((a, c) => a + c.remaining_amount, 0);
    const settledCount = plansWithProgress.filter((p) => p.is_settled).length;
    const activeCount = plansWithProgress.filter((p) => !p.is_settled).length;

    return {
      plans: plansWithProgress,
      receivableTotal,
      payableTotal,
      receivableRemaining,
      payableRemaining,
      settledCount,
      activeCount,
      netCommitment: receivableRemaining - payableRemaining,
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
  <div class="grid shrink-0 grid-cols-4 gap-2">
    <KpiCard
      label={i18n.t.totalScheduledReceivables}
      labelClass="text-income"
      subValue={`${i18n.t.remainingReceivables}: ${fmt(debtReportStats.receivableRemaining)}`}
    >
      <AnimatedCounter
        value={debtReportStats.receivableTotal}
        currency="IDR"
        class="text-medium text-income block leading-tight font-bold"
      />
    </KpiCard>

    <KpiCard
      label={i18n.t.totalDebtPayables}
      labelClass="text-expense"
      subValue={`${i18n.t.remainingPayables}: ${fmt(debtReportStats.payableRemaining)}`}
    >
      <AnimatedCounter
        value={debtReportStats.payableTotal}
        currency="IDR"
        class="text-medium text-expense block leading-tight font-bold"
      />
    </KpiCard>

    <KpiCard label={i18n.t.commitmentStatus}>
      <span class="font-proto flex items-baseline gap-2 leading-tight font-bold tabular-nums">
        <span class="text-text-strong text-medium">{debtReportStats.activeCount}</span>
        <span class="text-text-dim text-medium">/</span>
        <span class="text-income text-medium">{debtReportStats.settledCount}</span>
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
      subValue={debtReportStats.netCommitment >= 0
        ? i18n.t.netReceivableSurplus
        : i18n.t.netPayableDeficit}
    >
      <AnimatedCounter
        value={debtReportStats.netCommitment}
        currency="IDR"
        formatFn={formatNetSigned}
        class="text-medium block leading-tight font-bold {debtReportStats.netCommitment >= 0
          ? 'text-income'
          : 'text-expense'}"
      />
    </KpiCard>
  </div>

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
            class="bg-bg-app border-line space-y-2 border px-3 pt-2 pb-2.5 {item.is_settled
              ? 'opacity-65'
              : ''}"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Badge size="m" tone={p.plan_type === 'RECEIVABLE' ? 'ok' : 'err'}>
                  {planTypeLabel(p.plan_type)} • {planFreqLabel(p.frequency)}
                </Badge>
                <span class="font-proto text-text-strong text-medium font-bold">{p.title}</span>
              </div>
              {#if item.is_settled}
                <Badge size="m" tone="ok">
                  {i18n.t.settled100Percent}
                </Badge>
              {:else}
                <span
                  class="font-proto text-small font-bold tabular-nums {p.plan_type === 'RECEIVABLE'
                    ? 'text-income'
                    : 'text-expense'}"
                >
                  {i18n.t.remainingLabel}: {fmt(item.remaining_amount)}
                </span>
              {/if}
            </div>

            <div class="space-y-1">
              <div class="text-text-dim text-smaller flex justify-between">
                <span>{i18n.t.paidLabel}: {fmt(item.paid_amount)} / {fmt(p.total_amount)}</span>
                <span
                  >{item.progress_percent}% ({i18n.t.planCountShort.replace(
                    '{n}',
                    String(item.installments_paid_count)
                  )}
                  {i18n.t.installmentsCountLabel})</span
                >
              </div>
              <ProgressBar
                value={item.progress_percent}
                tone={p.plan_type === 'RECEIVABLE' ? 'income' : 'expense'}
                track="card"
                size="m"
              />
            </div>

            <div
              class="text-text-muted border-line/40 text-smaller flex items-center justify-between border-t pt-1"
            >
              <span>
                {i18n.t.planTargetPer}:
                <strong class="text-text-strong">{fmt(p.installment_amount)}</strong>
              </span>
              {#if p.due_date}
                <span class="text-text-dim">
                  {i18n.t.date}: {p.due_date}
                </span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </Card>

  <div class="shrink-0">
    <DebtSimulator {accounts} />
  </div>
</div>
