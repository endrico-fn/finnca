<script lang="ts">
  import { KpiCard, AnimatedCounter } from '$lib/components/ui';
  import { DonutChart } from '$lib/components/charts';
  import { i18n } from '$lib/core/i18n.svelte';
  import type { MonthCalculation } from '../state/budget.svelte';

  let {
    budgetData,
    currency = 'IDR',
  }: {
    budgetData: MonthCalculation;
    currency?: string;
  } = $props();
</script>

<div class="grid shrink-0 grid-cols-1 gap-2 md:grid-cols-3">
  <KpiCard
    label={i18n.t.budgetReadyToAssign}
    badge={budgetData.toBeBudgeted === 0
      ? i18n.t.budgetStatusBalanced
      : budgetData.toBeBudgeted > 0
        ? i18n.t.budgetStatusUnassigned
        : i18n.t.budgetStatusDeficit}
    badgeTone={budgetData.toBeBudgeted === 0
      ? 'ok'
      : budgetData.toBeBudgeted > 0
        ? 'neutral'
        : 'err'}
    subValue={budgetData.toBeBudgeted > 0
      ? i18n.t.budgetGiveDollarJob
      : budgetData.toBeBudgeted < 0
        ? i18n.t.budgetAssignedMore
        : i18n.t.budgetAllFundsAssigned}
  >
    <AnimatedCounter
      value={budgetData.toBeBudgeted}
      currency={currency as 'IDR' | 'USD'}
      class="text-large block leading-tight font-bold {budgetData.toBeBudgeted === 0
        ? 'text-text-strong'
        : budgetData.toBeBudgeted > 0
          ? 'text-income'
          : 'text-expense'}"
    />
  </KpiCard>

  <KpiCard
    label={i18n.t.budgetAssignedThisMonth}
    subValue={`${budgetData.envelopes.filter((e) => e.assigned > 0).length} ${i18n.t.envelopesLabel}`}
  >
    <div class="flex items-center justify-between">
      <AnimatedCounter
        value={budgetData.totalAssigned}
        currency={currency as 'IDR' | 'USD'}
        class="text-large text-text-strong block leading-tight font-bold"
      />
      {#if budgetData.totalAssigned > 0}
        <DonutChart
          size={36}
          strokeWidth={6}
          data={[
            { id: 'assigned', label: 'Assigned', value: budgetData.totalAssigned, color: 'var(--color-teal)' },
            { id: 'unassigned', label: 'Unassigned', value: Math.max(0, budgetData.toBeBudgeted), color: 'var(--color-bg-row-active)' }
          ]}
        />
      {/if}
    </div>
  </KpiCard>

  <KpiCard
    label={i18n.t.budgetActivity}
    labelClass="text-expense"
    subValue={budgetData.totalAssigned > 0
      ? i18n.t.percentSpent.replace(
          '{pct}',
          String(Math.round((budgetData.totalActivity / budgetData.totalAssigned) * 100))
        )
      : i18n.t.percentSpent.replace('{pct}', '0')}
  >
    <div class="flex items-center justify-between">
      <AnimatedCounter
        value={budgetData.totalActivity}
        currency={currency as 'IDR' | 'USD'}
        class="text-large text-expense block leading-tight font-bold"
      />
      {#if budgetData.totalActivity > 0}
        <DonutChart
          size={36}
          strokeWidth={6}
          data={[
            { id: 'spent', label: 'Spent', value: budgetData.totalActivity, color: 'var(--color-expense)' },
            { id: 'remaining', label: 'Remaining', value: Math.max(0, budgetData.totalAssigned - budgetData.totalActivity), color: 'var(--color-bg-row-active)' }
          ]}
        />
      {/if}
    </div>
  </KpiCard>
</div>
