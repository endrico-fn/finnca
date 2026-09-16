<script lang="ts">
  import { formatIDR } from '$lib/core/format/currency';
  import { Card, Badge } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import type { MonthCalculation } from '../state/budget.svelte';

  let {
    budgetData,
  }: {
    budgetData: MonthCalculation;
  } = $props();
</script>

<div class="grid shrink-0 grid-cols-1 gap-2 md:grid-cols-3">
  <!-- To Be Budgeted Card -->
  <Card title={i18n.t.budgetReadyToAssign}>
    {#snippet header()}
      <Badge
        size="m"
        tone={budgetData.toBeBudgeted === 0
          ? 'ok'
          : budgetData.toBeBudgeted > 0
            ? 'neutral'
            : 'err'}
      >
        {budgetData.toBeBudgeted === 0
          ? i18n.t.budgetStatusBalanced
          : budgetData.toBeBudgeted > 0
            ? i18n.t.budgetStatusUnassigned
            : i18n.t.budgetStatusDeficit}
      </Badge>
    {/snippet}

    <div class="mt-1 flex flex-col">
      <span
        class="font-proto text-large truncate font-bold tabular-nums {budgetData.toBeBudgeted === 0
          ? 'text-text-strong'
          : budgetData.toBeBudgeted > 0
            ? 'text-income'
            : 'text-expense'}"
        title={formatIDR(budgetData.toBeBudgeted)}
      >
        {formatIDR(budgetData.toBeBudgeted)}
      </span>
      <span class="text-text-dim font-proto text-smaller mt-1">
        {#if budgetData.toBeBudgeted > 0}
          {i18n.t.budgetGiveDollarJob}
        {:else if budgetData.toBeBudgeted < 0}
          {i18n.t.budgetAssignedMore}
        {:else}
          {i18n.t.budgetAllFundsAssigned}
        {/if}
      </span>
    </div>
  </Card>

  <!-- Total Assigned Card -->
  <Card title={i18n.t.budgetAssignedThisMonth}>
    {#snippet header()}
      <span class="font-proto text-text-dim text-smaller ml-auto tracking-widest">
        {i18n.t.budgetAllocatedLabel}
      </span>
    {/snippet}
    <div class="mt-1 flex flex-col">
      <span
        class="font-proto text-text-strong text-large truncate font-bold tabular-nums"
        title={formatIDR(budgetData.totalAssigned)}
      >
        {formatIDR(budgetData.totalAssigned)}
      </span>
      <span class="text-text-dim font-proto text-smaller mt-1">
        {budgetData.envelopes.filter((e) => e.assigned > 0).length}
        {i18n.t.envelopesLabel}
      </span>
    </div>
  </Card>

  <!-- Total Activity Card -->
  <Card title={i18n.t.budgetActivity}>
    {#snippet header()}
      <span class="font-proto text-text-dim text-smaller ml-auto tracking-widest">
        {i18n.t.budgetOutflowLabel}
      </span>
    {/snippet}
    <div class="mt-1 flex flex-col">
      <span
        class="font-proto text-expense text-large truncate font-bold tabular-nums"
        title={formatIDR(budgetData.totalActivity)}
      >
        {formatIDR(budgetData.totalActivity)}
      </span>
      <span class="text-text-dim font-proto text-smaller mt-1">
        {budgetData.totalAssigned > 0
          ? i18n.t.percentSpent.replace(
              '{pct}',
              String(Math.round((budgetData.totalActivity / budgetData.totalAssigned) * 100))
            )
          : i18n.t.percentSpent.replace('{pct}', '0')}
      </span>
    </div>
  </Card>
</div>
