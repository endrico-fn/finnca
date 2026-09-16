<script lang="ts">
  import type { Transaction } from '$lib/core/types';
  import type { Account } from '$lib/core/ipc/bindings';
  import { formatIDR } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Tabs, Card } from '$lib/components/ui';

  let {
    transactions = [],
    accountsById = new Map(),
  }: {
    transactions?: Transaction[];
    accountsById?: Map<string, Account>;
  } = $props();

  let cashflowPeriod = $state<'month' | 'all'>('month');

  function computePnl(txs: Transaction[], fromDate?: string, toDate?: string) {
    let income = 0;
    let expense = 0;
    for (const t of txs) {
      if (fromDate && t.date < fromDate) continue;
      if (toDate && t.date > toDate) continue;
      for (const s of t.splits) {
        const acc = accountsById.get(s.accountId);
        if (!acc) continue;
        if (acc.account_type === 'INCOME') {
          income += -s.amount;
        } else if (acc.account_type === 'EXPENSE') {
          expense += s.amount;
        }
      }
    }
    return { income, expense, net: income - expense };
  }

  const activePnl = $derived.by(() => {
    if (cashflowPeriod === 'month') {
      const now = new Date();
      const currentMonthStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
      return computePnl(transactions, currentMonthStart, undefined);
    }
    return computePnl(transactions);
  });

  const savingsRate = $derived(
    activePnl.income > 0 ? Math.max(0, (activePnl.net / activePnl.income) * 100).toFixed(1) : '0.0'
  );
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

  <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 w-full pt-1">
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
</Card>
