<script lang="ts">
  import { reportState } from '../state/report.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import {
    Card,
    Badge,
    EmptyState,
    KpiCard,
    AnimatedCounter,
    ProgressBar,
    Button,
  } from '$lib/components/ui';
  import { DonutChart, type DonutSlice } from '$lib/components/charts';

  let { from, to }: { from: string; to: string } = $props();

  let selectedCategoryId = $state<string | null>(null);

  $effect(() => {
    reportState.loadProfitLoss(from || undefined, to || undefined);
  });

  const pnl = $derived(reportState.profitLoss);
  const totalExpense = $derived(pnl?.total_expenses ?? 0);
  const expenseRows = $derived(
    (pnl?.expense_rows ?? []).filter((r) => r.amount > 0).sort((a, b) => b.amount - a.amount)
  );

  const topCategory = $derived(expenseRows.length > 0 ? expenseRows[0] : null);

  const daysCount = $derived.by(() => {
    if (!from || !to) return 30;
    const d1 = new Date(from).getTime();
    const d2 = new Date(to).getTime();
    const diff = Math.round(Math.abs((d2 - d1) / (1000 * 60 * 60 * 24))) + 1;
    return Math.max(1, diff);
  });

  const dailyBurn = $derived(totalExpense > 0 ? Math.round(totalExpense / daysCount) : 0);
  const avgPerCat = $derived(
    totalExpense > 0 && expenseRows.length > 0 ? Math.round(totalExpense / expenseRows.length) : 0
  );

  const DONUT_COLORS = [
    'var(--color-teal)',
    'var(--color-income)',
    'var(--color-expense)',
    'var(--color-warning)',
    'var(--color-text-dim)',
    'var(--color-text-muted)',
  ];

  function getCategoryColor(idx: number): string {
    return DONUT_COLORS[idx % DONUT_COLORS.length];
  }

  const donutSlices = $derived.by<DonutSlice[]>(() => {
    if (totalExpense <= 0) return [];
    return expenseRows.slice(0, 6).map((row, idx) => ({
      id: row.account_id,
      label: row.name,
      value: row.amount,
      color: getCategoryColor(idx),
    }));
  });

  const focusedCategory = $derived.by(() => {
    if (!selectedCategoryId) return null;
    return expenseRows.find((r) => r.account_id === selectedCategoryId) ?? null;
  });

  const focusedPct = $derived.by(() => {
    if (!focusedCategory || totalExpense <= 0) return 0;
    return Math.round((focusedCategory.amount / totalExpense) * 100);
  });

  const topPct = $derived.by(() => {
    if (!topCategory || totalExpense <= 0) return 0;
    return Math.round((topCategory.amount / totalExpense) * 100);
  });

  function toggleCategory(id: string) {
    selectedCategoryId = selectedCategoryId === id ? null : id;
  }
</script>

<div class="flex min-h-0 w-full flex-1 flex-col gap-2">
  <div class="grid shrink-0 grid-cols-1 gap-2 sm:grid-cols-3">
    <KpiCard label={i18n.t.expense} labelClass="text-expense">
      <AnimatedCounter
        value={totalExpense}
        currency="IDR"
        class="text-medium text-expense block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight uppercase">
        {expenseRows.length}
        {i18n.t.accountsTitle}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.topCategories}>
      <span class="text-medium text-text-white font-proto block truncate leading-tight font-bold">
        {topCategory ? topCategory.name : '—'}
      </span>
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight uppercase">
        {#if topCategory && totalExpense > 0}
          {topPct}% {i18n.t.totals}
        {:else}
          —
        {/if}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.dailyBurnVelocity}>
      <AnimatedCounter
        value={dailyBurn}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight uppercase">
        {daysCount}
        {i18n.t.daysCountLabel}
      </span>
    </KpiCard>
  </div>

  {#if expenseRows.length === 0}
    <div class="border-line bg-bg-app flex min-h-0 flex-1 items-center justify-center border">
      <EmptyState title={i18n.t.noSpendingData} hint={i18n.t.adjustFilterHint} icon="chart" />
    </div>
  {:else}
    <div class="grid min-h-0 flex-1 grid-cols-1 gap-2 lg:grid-cols-12">
      <Card
        title={i18n.t.categoryDistribution}
        class="flex min-h-0 flex-col justify-between p-4 lg:col-span-4"
      >
        <div class="flex flex-1 flex-col items-center justify-center py-4">
          <DonutChart
            data={donutSlices}
            size={160}
            strokeWidth={20}
            centerLabel={focusedCategory
              ? focusedCategory.name
              : `${expenseRows.length} ${i18n.t.accountsTitle}`}
            centerValue={focusedCategory
              ? `${focusedPct}%`
              : formatMinorToDisplay(totalExpense, 'IDR')}
          />
        </div>

        <div class="border-line/40 font-proto text-smaller flex flex-col gap-2 border-t pt-4">
          {#if focusedCategory}
            <div class="flex items-center justify-between">
              <span class="text-text-dim uppercase">{i18n.t.selectedCategory}</span>
              <span class="text-teal font-bold">{focusedCategory.code}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-text-white truncate font-medium">{focusedCategory.name}</span>
              <span class="text-expense font-bold tabular-nums">
                {formatMinorToDisplay(focusedCategory.amount, focusedCategory.currency || 'IDR')}
              </span>
            </div>
            <Button
              variant="ghost"
              onclick={() => (selectedCategoryId = null)}
              class="text-smaller font-proto mt-1 w-full uppercase"
            >
              {i18n.t.clearFocus}
            </Button>
          {:else}
            <div class="flex items-center justify-between">
              <span class="text-text-dim uppercase">{i18n.t.averagePerCategory}</span>
              <span class="text-text-white font-bold tabular-nums">
                {formatMinorToDisplay(avgPerCat, 'IDR')}
              </span>
            </div>
            {#if topCategory}
              <div class="flex items-center justify-between">
                <span class="text-text-dim uppercase">{i18n.t.topCategories}</span>
                <span class="text-text-white truncate font-medium"
                  >{topCategory.name} ({topPct}%)</span
                >
              </div>
            {/if}
          {/if}
        </div>
      </Card>

      <div class="sharp-card flex min-h-0 flex-1 flex-col overflow-hidden lg:col-span-8">
        <!-- Proportional distribution stacked bar across all categories -->
        <div class="border-line/60 bg-bg-card/40 border-b px-4 py-3">
          <div class="flex items-center justify-between pb-1.5">
            <span class="font-proto text-text-dim text-smaller font-bold tracking-wider uppercase">
              {i18n.t.spendingDistribution}
            </span>
            <span class="font-proto text-text-muted text-smaller tabular-nums"> 100% </span>
          </div>
          <div class="border-line/60 bg-bg-app flex h-3 w-full overflow-hidden border">
            {#each expenseRows as row, idx (row.account_id)}
              {@const share = totalExpense > 0 ? (row.amount / totalExpense) * 100 : 0}
              {@const isSel = selectedCategoryId === row.account_id}
              <button
                type="button"
                onclick={() => toggleCategory(row.account_id)}
                class="h-full cursor-pointer transition-opacity {isSel
                  ? 'ring-1 ring-white ring-inset'
                  : 'hover:opacity-80'}"
                style="width: {share}%; background-color: {getCategoryColor(idx)};"
                title="{row.name}: {share.toFixed(1)}% ({formatMinorToDisplay(
                  row.amount,
                  row.currency || 'IDR'
                )})"
                aria-label="{row.name}: {share.toFixed(1)}%"
              ></button>
            {/each}
          </div>
        </div>

        <div class="flex-1 overflow-y-auto">
          <table class="sharp-table">
            <thead class="sticky top-0 z-10">
              <tr>
                <th class="w-20 pl-3">{i18n.t.code}</th>
                <th class="px-3">{i18n.t.account}</th>
                <th class="w-48 px-3 text-right">{i18n.t.colPctExpense}</th>
                <th class="w-36 pr-3 text-right">{i18n.t.amount}</th>
              </tr>
            </thead>
            <tbody>
              {#each expenseRows as row, idx (row.account_id)}
                {@const pct = totalExpense > 0 ? (row.amount / totalExpense) * 100 : 0}
                {@const isSelected = selectedCategoryId === row.account_id}
                {@const rowColor = getCategoryColor(idx)}
                <tr
                  class="hover:bg-bg-card cursor-pointer transition-colors {isSelected
                    ? 'selected border-teal bg-bg-card border-l-2'
                    : ''}"
                  onclick={() => toggleCategory(row.account_id)}
                >
                  <td
                    class="text-text-muted font-proto text-smaller w-20 pl-3 font-bold whitespace-nowrap"
                  >
                    {row.code}
                  </td>
                  <td class="px-3">
                    <div class="flex items-center gap-2">
                      <span class="size-1.5 shrink-0" style="background-color: {rowColor}"></span>
                      <span class="text-text-white font-proto text-smaller truncate font-medium">
                        {row.name}
                      </span>
                      {#if isSelected}
                        <Badge size="s" tone="teal">{i18n.t.activeStatus}</Badge>
                      {/if}
                    </div>
                  </td>
                  <td class="w-48 px-3 text-right">
                    <div class="flex items-center justify-end gap-2">
                      <ProgressBar value={pct} tone="teal" size="xs" class="w-24 shrink-0" />
                      <span
                        class="text-text-muted font-proto text-smaller w-12 text-right tabular-nums"
                      >
                        {pct.toFixed(1)}%
                      </span>
                    </div>
                  </td>
                  <td
                    class="text-expense font-proto text-smaller w-36 pr-3 text-right font-bold whitespace-nowrap tabular-nums"
                  >
                    {formatMinorToDisplay(row.amount, row.currency || 'IDR')}
                  </td>
                </tr>
              {/each}
            </tbody>
            <tfoot>
              <tr>
                <td
                  colspan="2"
                  class="text-text-strong font-proto text-smaller pl-3 font-bold tracking-widest uppercase"
                >
                  {i18n.t.totals} ({expenseRows.length}
                  {i18n.t.accountsTitle})
                </td>
                <td
                  class="text-text-muted font-proto text-smaller px-3 text-right font-bold tabular-nums"
                >
                  100.0%
                </td>
                <td
                  class="text-expense font-proto text-smaller pr-3 text-right font-bold tabular-nums"
                >
                  {formatMinorToDisplay(totalExpense, 'IDR')}
                </td>
              </tr>
            </tfoot>
          </table>
        </div>
      </div>
    </div>
  {/if}
</div>
