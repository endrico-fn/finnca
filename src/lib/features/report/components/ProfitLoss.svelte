<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { Card } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number) => formatMinorToDisplay(n, 'IDR');

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

  const profitMargin = $derived(
    totalIncome > 0 ? Math.round((netIncome / totalIncome) * 100) : 0
  );
  const expenseRatio = $derived(
    totalIncome > 0 ? Math.round((totalExpenses / totalIncome) * 100) : 0
  );
</script>

<div class="space-y-4">
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.revenues}
      </div>
      <div class="font-proto text-income text-lg font-bold tabular-nums">
        {fmt(totalIncome)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.operationalExpenses}
      </div>
      <div class="font-proto text-expense text-lg font-bold tabular-nums">
        {fmt(totalExpenses)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.netIncome}
      </div>
      <div class="font-proto text-lg font-bold tabular-nums {netIncome >= 0 ? 'text-income' : 'text-expense'}">
        {fmt(netIncome)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.financialEfficiency}
      </div>
      <div class="font-proto text-text-base text-lg font-bold tabular-nums">
        {profitMargin}% <span class="text-text-dim text-smaller font-normal">({i18n.t.profitMargin})</span>
      </div>
    </Card>
  </div>

  <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
    <Card divided title={i18n.t.revenues}>
      {#snippet header()}
        <span class="font-proto text-income text-small font-bold tabular-nums">
          {fmt(totalIncome)}
        </span>
      {/snippet}

      {#if incomeRows.length === 0}
        <div class="text-text-dim font-aux py-8 text-center text-small">
          {i18n.t.noIncomeAccounts}
        </div>
      {:else}
        <table class="sharp-table">
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
                <td class="numeric font-proto text-income text-smaller w-36 pr-3 whitespace-nowrap tabular-nums">
                  {fmt(row.amount)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </Card>

    <Card divided title={i18n.t.operationalExpenses}>
      {#snippet header()}
        <span class="font-proto text-expense text-small font-bold tabular-nums">
          {fmt(totalExpenses)}
        </span>
      {/snippet}

      {#if expenseRows.length === 0}
        <div class="text-text-dim font-aux py-8 text-center text-small">
          {i18n.t.noExpenseAccounts}
        </div>
      {:else}
        <table class="sharp-table">
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
                <td class="numeric font-proto text-expense text-smaller w-36 pr-3 whitespace-nowrap tabular-nums">
                  {fmt(row.amount)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </Card>
  </div>

  <Card class="border-line bg-bg-card/60 p-3">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="font-proto text-smaller text-text-dim flex items-center gap-2">
        <span class="text-text-base font-bold">{i18n.t.netIncome}:</span>
        <span>{fmt(totalIncome)} - {fmt(totalExpenses)} = </span>
        <strong class="font-bold {netIncome >= 0 ? 'text-income' : 'text-expense'}">
          {fmt(netIncome)}
        </strong>
      </div>
      <div class="font-proto text-smaller text-text-dim flex items-center gap-4">
        <span>{i18n.t.expenseRatio}: <strong class="text-text-base">{expenseRatio}%</strong></span>
        <span>{i18n.t.profitMargin}: <strong class="text-text-base">{profitMargin}%</strong></span>
      </div>
    </div>
  </Card>
</div>
