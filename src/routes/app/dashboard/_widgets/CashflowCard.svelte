<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { incomeStatement, formatIDR } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import { Tabs, Card } from '$lib/components/ui';

  let cashflowPeriod = $state<'month' | 'all'>('month');

  const activePnl = $derived.by(() => {
    if (!ledger.data) return { income: 0, expense: 0, net: 0 };
    if (cashflowPeriod === 'month') {
      const now = new Date();
      const currentMonthStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
      return incomeStatement(ledger.data, currentMonthStart, undefined);
    }
    return incomeStatement(ledger.data);
  });

  const savingsRate = $derived(
    activePnl.income > 0 ? Math.max(0, (activePnl.net / activePnl.income) * 100).toFixed(1) : '0.0'
  );

  const last6Months = $derived.by(() => {
    if (!ledger.data) return [];
    const months = [];
    const d = new Date();
    for (let i = 5; i >= 0; i--) {
      const target = new Date(d.getFullYear(), d.getMonth() - i, 1);
      const y = target.getFullYear();
      const m = String(target.getMonth() + 1).padStart(2, '0');
      const label = target.toLocaleString('en-US', { month: 'short' }).toUpperCase();
      const start = `${y}-${m}-01`;
      const lastDay = new Date(y, target.getMonth() + 1, 0).getDate();
      const end = `${y}-${m}-${String(lastDay).padStart(2, '0')}`;

      const pnl = incomeStatement(ledger.data, start, end);
      months.push({
        label,
        income: pnl.income,
        expense: pnl.expense,
        net: pnl.net,
      });
    }
    const maxVal = Math.max(1, ...months.map((m) => Math.max(m.income, m.expense)));
    return months.map((m) => ({
      ...m,
      incH: Math.max(2, Math.round((m.income / maxVal) * 28)),
      expH: Math.max(2, Math.round((m.expense / maxVal) * 28)),
    }));
  });
</script>

<Card title={i18n.t.pnlTitle}>
  {#snippet header()}
    <Tabs
      variant="outline"
      tabs={[
        { id: 'month', label: i18n.t.thisMonth },
        { id: 'all', label: i18n.t.allTime },
      ]}
      active={cashflowPeriod}
      onSelect={(id) => (cashflowPeriod = id as 'month' | 'all')}
    />
  {/snippet}

  <div class="mt-1 flex w-full items-center justify-between">
    <div class="flex flex-1 items-center justify-between pr-8">
      <div>
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.income}
        </p>
        <p class="text-income font-proto text-medium mt-1 font-bold tabular-nums">
          {formatIDR(activePnl.income)}
        </p>
      </div>
      <div>
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.expense}
        </p>
        <p class="text-expense font-proto text-medium mt-1 font-bold tabular-nums">
          {formatIDR(activePnl.expense)}
        </p>
      </div>
      <div>
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.net}
        </p>
        <p
          class="{activePnl.net >= 0
            ? 'text-income'
            : 'text-expense'} font-proto text-medium mt-1 font-bold tabular-nums"
        >
          {formatIDR(activePnl.net)}
        </p>
      </div>
      <div>
        <p class="text-text-base font-proto text-smaller tracking-widest uppercase">
          {i18n.t.savingsRate}
        </p>
        <p class="text-teal font-proto text-medium mt-1 font-bold tabular-nums">{savingsRate}%</p>
      </div>
    </div>

    <div class="border-line/40 flex h-10 w-64 shrink-0 items-end justify-between border-l pl-6">
      {#each last6Months as m (m.label)}
        <div
          class="group flex cursor-crosshair flex-col items-center gap-0.5"
          title="{m.label}: {i18n.t.flowIn} {formatIDR(m.income)} / {i18n.t.flowOut} {formatIDR(
            m.expense
          )}"
        >
          <div class="flex h-7 items-end gap-[1px]">
            <div
              class="bg-income/80 group-hover:bg-income w-2 transition-colors"
              style="height: {m.incH}px"
            ></div>
            <div
              class="bg-expense/80 group-hover:bg-expense w-2 transition-colors"
              style="height: {m.expH}px"
            ></div>
          </div>
          <span class="font-proto text-text-dim group-hover:text-text-strong text-smaller"
            >{m.label}</span
          >
        </div>
      {/each}
    </div>
  </div>
</Card>
