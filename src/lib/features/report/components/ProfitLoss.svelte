<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { Card, KpiCard, AnimatedCounter } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number, c = 'IDR') => formatMinorToDisplay(n, c);

  let { from = '', to = '' }: { from?: string; to?: string } = $props();

  $effect(() => {
    reportState.loadProfitLoss(from || undefined, to || undefined);
  });

  const pnl = $derived(reportState.profitLoss);
  const incomeRows = $derived(pnl?.income_rows ?? []);
  const expenseRows = $derived(pnl?.expense_rows ?? []);

  const totalIncome = $derived(pnl?.total_income ?? 0);
  const totalExpenses = $derived(pnl?.total_expenses ?? 0);
  const netIncome = $derived(pnl?.net_income ?? 0);

  const profitMargin = $derived(totalIncome > 0 ? Math.round((netIncome / totalIncome) * 100) : 0);
  const expenseRatio = $derived(
    totalIncome > 0 ? Math.round((totalExpenses / totalIncome) * 100) : 0
  );
</script>

<div class="flex min-h-0 w-full flex-1 flex-col gap-3">
  <div class="grid shrink-0 grid-cols-2 gap-2.5 sm:grid-cols-4">
    <KpiCard label={i18n.t.revenues} labelClass="text-income">
      <AnimatedCounter
        value={totalIncome}
        currency="IDR"
        class="text-medium text-income block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {incomeRows.length}
        {i18n.t.accountsTitle}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.operationalExpenses} labelClass="text-expense">
      <AnimatedCounter
        value={totalExpenses}
        currency="IDR"
        class="text-medium text-expense block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {expenseRows.length}
        {i18n.t.accountsTitle}
      </span>
    </KpiCard>

    <KpiCard
      label={i18n.t.netIncome}
      labelClass={netIncome >= 0 ? 'text-text-white' : 'text-expense'}
      badge={netIncome >= 0 ? i18n.t.statusOk : i18n.t.statusErr}
      badgeTone={netIncome >= 0 ? 'ok' : 'err'}
    >
      <AnimatedCounter
        value={netIncome}
        currency="IDR"
        class="text-medium block leading-tight font-bold {netIncome < 0
          ? 'text-expense'
          : 'text-text-white'}"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {i18n.t.netIncome}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.financialEfficiency}>
      <span
        class="text-medium text-text-white font-proto block leading-tight font-bold tabular-nums"
      >
        {profitMargin}%
      </span>
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {i18n.t.profitMargin}
      </span>
    </KpiCard>
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-1 gap-3 lg:grid-cols-2">
    <Card divided title={i18n.t.revenues} class="flex min-h-0 flex-1 flex-col">
      {#snippet header()}
        <span class="font-proto text-text-white text-small font-bold tabular-nums">
          {fmt(totalIncome)}
        </span>
      {/snippet}

      {#if incomeRows.length === 0}
        <div
          class="text-text-dim font-aux text-small flex flex-1 items-center justify-center py-8 text-center"
        >
          {i18n.t.noIncomeAccounts}
        </div>
      {:else}
        <div class="flex-1 overflow-y-auto">
          <table class="sharp-table w-full">
            <thead>
              <tr>
                <th class="w-24 pl-3">{i18n.t.colCode}</th>
                <th class="px-3">{i18n.t.name}</th>
                <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
              </tr>
            </thead>
            <tbody>
              {#each incomeRows as row (row.account_id)}
                <tr>
                  <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
                    {row.code}
                  </td>
                  <td class="font-aux text-text-white text-small px-3">
                    <div class="flex items-center gap-2">
                      <span class="bg-income size-1.5 shrink-0"></span>
                      <span class="truncate">{row.name}</span>
                    </div>
                  </td>
                  <td
                    class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums"
                  >
                    {fmt(row.amount, row.currency)}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </Card>

    <Card divided title={i18n.t.operationalExpenses} class="flex min-h-0 flex-1 flex-col">
      {#snippet header()}
        <span class="font-proto text-text-white text-small font-bold tabular-nums">
          {fmt(totalExpenses)}
        </span>
      {/snippet}

      {#if expenseRows.length === 0}
        <div
          class="text-text-dim font-aux text-small flex flex-1 items-center justify-center py-8 text-center"
        >
          {i18n.t.noExpenseAccounts}
        </div>
      {:else}
        <div class="flex-1 overflow-y-auto">
          <table class="sharp-table w-full">
            <thead>
              <tr>
                <th class="w-24 pl-3">{i18n.t.colCode}</th>
                <th class="px-3">{i18n.t.name}</th>
                <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
              </tr>
            </thead>
            <tbody>
              {#each expenseRows as row (row.account_id)}
                <tr>
                  <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
                    {row.code}
                  </td>
                  <td class="font-aux text-text-white text-small px-3">
                    <div class="flex items-center gap-2">
                      <span class="bg-expense size-1.5 shrink-0"></span>
                      <span class="truncate">{row.name}</span>
                    </div>
                  </td>
                  <td
                    class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums"
                  >
                    {fmt(row.amount, row.currency)}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </Card>
  </div>

  <Card class="border-line bg-bg-card/60 shrink-0 p-2.5">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="font-proto text-smaller text-text-dim flex items-center gap-2">
        <span class="text-text-base font-bold">{i18n.t.netIncome}:</span>
        <span>{fmt(totalIncome)} - {fmt(totalExpenses)} = </span>
        <strong class="font-bold {netIncome < 0 ? 'text-expense' : 'text-text-white'}">
          {fmt(netIncome)}
        </strong>
      </div>
      <div class="font-proto text-smaller text-text-dim flex items-center gap-4">
        <span>{i18n.t.expenseRatio}: <strong class="text-text-white">{expenseRatio}%</strong></span>
        <span>{i18n.t.profitMargin}: <strong class="text-text-white">{profitMargin}%</strong></span>
      </div>
    </div>
  </Card>
</div>
