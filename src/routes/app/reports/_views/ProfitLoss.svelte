<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    incomeStatement,
    fromMinor,
    formatIDR,
    formatUSD,
    accountBalanceMinorFiltered,
    buildChildrenMap,
  } from '$lib/accounting/finance';

  let { from, to } = $props<{ from: string; to: string }>();

  const fmt = (n: number) => n.toLocaleString('en-US');

  const pnl = $derived(
    ledger.data
      ? incomeStatement(ledger.data, from || undefined, to || undefined)
      : { income: 0, expense: 0, net: 0 }
  );

  const childrenMap = $derived(ledger.data ? buildChildrenMap(ledger.data.accounts) : new Map());

  const incomeAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'INCOME' && !a.placeholder)
  );
  const expenseAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'EXPENSE' && !a.placeholder)
  );

  const totalFlow = $derived(pnl.income + pnl.expense);
  const incomePercent = $derived(totalFlow > 0 ? (pnl.income / totalFlow) * 100 : 50);
  const expensePercent = $derived(totalFlow > 0 ? (pnl.expense / totalFlow) * 100 : 50);
  const netMargin = $derived(pnl.income > 0 ? ((pnl.net / pnl.income) * 100).toFixed(1) : '0.0');
  import { ReportCard, AccountRow, Card } from '$lib/components/ui';
</script>

<div class="grid min-h-0 flex-1 grid-cols-12 gap-2">
  <!-- P&L Summary Breakdown (Left 8 cols) -->
  <ReportCard
    title={i18n.t.pnlTitle}
    description={i18n.t.pnlNetIncomeDesc}
    class="col-span-8 flex-1"
  >
    {#snippet headerRight()}
      <div class="text-right">
        <span class="text-text-muted block text-[9px] uppercase">{i18n.t.netIncome}</span>
        <span
          class="font-proto text-[14px] font-bold {pnl.net >= 0 ? 'text-income' : 'text-expense'}"
        >
          Rp {fmt(fromMinor('IDR', pnl.net))}
        </span>
      </div>
    {/snippet}

    <div class="min-h-0 flex-1 space-y-2 pr-1 text-[12px]">
      <!-- Revenues / Income Card -->
      <div class="sharp-card">
        <div class="flex items-center justify-between px-3 py-2">
          <span class="text-income flex items-center gap-2 text-[11px] font-bold tracking-wider">
            <span class="bg-income size-1.5"></span>
            {i18n.t.revenues}
          </span>
          <span class="text-income font-proto text-[11px] font-bold"
            >Rp {fmt(fromMinor('IDR', pnl.income))}</span
          >
        </div>
        <div class="divide-line/30 divide-y">
          {#each incomeAccounts as acc (acc.id)}
            {@const bal = ledger.data
              ? accountBalanceMinorFiltered(
                  acc.id,
                  ledger.data,
                  from || undefined,
                  to || undefined,
                  childrenMap
                )
              : 0}
            <AccountRow account={acc} balance={bal} valueClass="text-income" />
          {:else}
            <p class="text-text-dim p-3 text-[11px]">
              {i18n.t.noIncomeAccounts}
            </p>
          {/each}
        </div>
      </div>

      <!-- Expenses Card -->
      <div class="sharp-card">
        <div class="flex items-center justify-between px-3 py-2">
          <span class="text-expense flex items-center gap-2 text-[11px] font-bold tracking-wider">
            <span class="bg-expense size-1.5"></span>
            {i18n.t.expense}
          </span>
          <span class="text-expense font-proto text-[11px] font-bold"
            >Rp {fmt(fromMinor('IDR', pnl.expense))}</span
          >
        </div>
        <div class="divide-line/30 divide-y">
          {#each expenseAccounts as acc (acc.id)}
            {@const bal = ledger.data
              ? accountBalanceMinorFiltered(
                  acc.id,
                  ledger.data,
                  from || undefined,
                  to || undefined,
                  childrenMap
                )
              : 0}
            {@const pct = pnl.expense > 0 ? ((Math.abs(bal) / pnl.expense) * 100).toFixed(1) + '%' : '0.0%'}
            <AccountRow account={acc} balance={bal} valueClass="text-expense" percentage={pct} />
          {:else}
            <p class="text-text-dim p-3 text-[11px]">
              {i18n.t.noExpenseAccounts}
            </p>
          {/each}
        </div>
      </div>
    </div>
  </ReportCard>

  <!-- P&L Visuals (Right 4 cols) -->
  <div class="col-span-4 flex min-h-0 flex-col gap-2">
    <Card title={i18n.t.financialEfficiency}>
      <div class="flex flex-col gap-2.5 text-[12px]">
        <div class="flex justify-between">
          <span class="text-text-icon">{i18n.t.revenues}</span>
          <span class="text-income font-proto font-bold"
            >Rp {fmt(fromMinor('IDR', pnl.income))}</span
          >
        </div>
        <div class="flex justify-between">
          <span class="text-text-icon">{i18n.t.expense}</span>
          <span class="text-expense font-proto font-bold"
            >Rp {fmt(fromMinor('IDR', pnl.expense))}</span
          >
        </div>

        <!-- Visual Flow Ratio Progress Bar -->
        <div class="pt-1">
          <div class="text-text-dim mb-1 flex justify-between text-[9px]">
            <span>INCOME {incomePercent}%</span>
            <span>EXPENSE {expensePercent}%</span>
          </div>
          <div class="bg-bg-app border-line flex h-2.5 w-full overflow-hidden border">
            {#if totalFlow > 0}
              <div class="bg-income transition-all" style="width: {incomePercent}%"></div>
              <div class="bg-expense transition-all" style="width: {expensePercent}%"></div>
            {:else}
              <div class="bg-line h-full w-full"></div>
            {/if}
          </div>
        </div>

        <div class="border-line flex justify-between border-t pt-2 font-bold">
          <span class="text-text-strong">{i18n.t.profitMargin}</span>
          <span class="text-[13px] {pnl.net >= 0 ? 'text-income' : 'text-expense'} font-proto">
            {netMargin}%
          </span>
        </div>
      </div>
    </Card>

    <Card title={i18n.t.usdBenchmark} class="flex-1">
      <div class="flex flex-col gap-2 text-[12px]">
        <div class="flex justify-between">
          <span class="text-text-icon">{i18n.t.activeFxRate}</span>
          <span class="text-text-base font-proto">$1 = Rp {ledger.fxRate.toLocaleString()}</span>
        </div>
        <div class="border-line/50 flex justify-between border-t pt-2">
          <span class="text-text-icon">{i18n.t.netIncomeUsd}</span>
          <span class="{pnl.net >= 0 ? 'text-income' : 'text-expense'} font-proto font-bold">
            $ {(fromMinor('IDR', pnl.net) / ledger.fxRate).toLocaleString('en-US', {
              maximumFractionDigits: 2,
              minimumFractionDigits: 2,
            })}
          </span>
        </div>
      </div>
    </Card>
  </div>
</div>
