<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { Badge, Card, Button } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number) => formatMinorToDisplay(n, 'IDR');

  let selectedMonth = $state(new Date().toISOString().slice(0, 7));

  $effect(() => {
    reportState.loadBudgetSummary(selectedMonth);
  });

  const summary = $derived(reportState.budgetSummary);
  const envelopes = $derived(summary?.envelopes ?? []);
  const totalAssigned = $derived(summary?.total_assigned ?? 0);
  const totalActivity = $derived(summary?.total_activity ?? 0);
  const totalAvailable = $derived(totalAssigned - totalActivity);
  const usagePercent = $derived(
    totalAssigned > 0 ? Math.round((totalActivity / totalAssigned) * 100) : 0
  );

  function prevMonth() {
    const [y, m] = selectedMonth.split('-').map(Number);
    const d = new Date(y, m - 2, 1);
    selectedMonth = d.toISOString().slice(0, 7);
  }

  function nextMonth() {
    const [y, m] = selectedMonth.split('-').map(Number);
    const d = new Date(y, m, 1);
    selectedMonth = d.toISOString().slice(0, 7);
  }
</script>

<div class="space-y-4">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="sm" onclick={prevMonth}>◀</Button>
      <span class="font-proto text-text-white text-small px-2 font-bold tracking-wider">
        {selectedMonth}
      </span>
      <Button variant="ghost" size="sm" onclick={nextMonth}>▶</Button>
    </div>

    <div class="font-proto text-smaller text-text-dim flex items-center gap-4">
      <span>{i18n.t.colUsagePercent}: <strong class="{usagePercent > 100 ? 'text-expense' : 'text-income'} font-bold">{usagePercent}%</strong></span>
    </div>
  </div>

  <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.colAssigned}
      </div>
      <div class="font-proto text-teal text-lg font-bold tabular-nums">
        {fmt(totalAssigned)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.colActivity}
      </div>
      <div class="font-proto text-expense text-lg font-bold tabular-nums">
        {fmt(totalActivity)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.colAvailable}
      </div>
      <div class="font-proto text-lg font-bold tabular-nums {totalAvailable < 0 ? 'text-expense' : 'text-income'}">
        {fmt(totalAvailable)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="flex items-center justify-between">
        <span class="font-proto text-text-dim text-smaller uppercase tracking-wider">
          {i18n.t.status}
        </span>
        <Badge size="s" tone={usagePercent > 100 ? 'err' : usagePercent >= 85 ? 'warn' : 'ok'}>
          {usagePercent > 100 ? i18n.t.budgetOverrun : `${usagePercent}%`}
        </Badge>
      </div>
      <div class="mt-2 h-2 w-full bg-bg-app border border-line">
        <div
          class="h-full {usagePercent > 100 ? 'bg-expense' : usagePercent >= 85 ? 'bg-warning' : 'bg-income'} transition-all"
          style="width: {Math.min(usagePercent, 100)}%"
        ></div>
      </div>
    </Card>
  </div>

  <Card divided title={i18n.t.budgetVsActualTitle}>
    {#if envelopes.length === 0}
      <div class="text-text-dim font-aux py-8 text-center text-small">
        {i18n.t.noTbRecords}
      </div>
    {:else}
      <table class="sharp-table">
        <thead>
          <tr>
            <th class="w-24 pl-3">{i18n.t.colCode}</th>
            <th class="px-3">{i18n.t.name}</th>
            <th class="numeric w-32 px-3">{i18n.t.colAssigned}</th>
            <th class="numeric w-32 px-3">{i18n.t.colActivity}</th>
            <th class="numeric w-32 px-3">{i18n.t.colAvailable}</th>
            <th class="numeric w-28 pr-3">{i18n.t.colUsagePercent}</th>
          </tr>
        </thead>
        <tbody>
          {#each envelopes as env (env.account_id)}
            {@const pct = env.assigned > 0 ? Math.round((env.activity / env.assigned) * 100) : 0}
            {@const isOver = env.activity > env.assigned && env.assigned > 0}
            <tr>
              <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
                {env.account_code}
              </td>
              <td class="font-aux text-text-white text-small px-3">
                <span class="truncate">{env.account_name}</span>
              </td>
              <td class="numeric font-proto text-smaller w-32 px-3 whitespace-nowrap tabular-nums">
                {fmt(env.assigned)}
              </td>
              <td class="numeric font-proto text-expense text-smaller w-32 px-3 whitespace-nowrap tabular-nums">
                {fmt(env.activity)}
              </td>
              <td class="numeric font-proto text-smaller w-32 px-3 whitespace-nowrap tabular-nums {env.available < 0 ? 'text-expense font-bold' : 'text-income'}">
                {fmt(env.available)}
              </td>
              <td class="numeric font-proto text-smaller w-28 pr-3 whitespace-nowrap tabular-nums">
                <span class="{isOver ? 'text-expense font-bold' : pct >= 85 ? 'text-warning' : 'text-text-dim'}">
                  {pct}%
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Card>
</div>
