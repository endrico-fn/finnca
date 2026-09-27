<script lang="ts">
  import { SvelteDate } from 'svelte/reactivity';
  import { formatMinorGrouping, fromMinor, parseStringAmountToMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, ProgressBar, Tooltip } from '$lib/components/ui';
  import { budgetState, type MonthCalculation } from '../state/budget.svelte';

  let {
    budgetData,
    currency = 'IDR',
    onAssignBudget,
  }: {
    budgetData: MonthCalculation;
    currency?: string;
    onAssignBudget: (accountId: string, amountMinor: number) => Promise<void>;
  } = $props();

  const currencySymbol = $derived(currency === 'IDR' ? 'Rp' : currency);

  const totalAvailable = $derived(budgetData.envelopes.reduce((sum, e) => sum + e.available, 0));

  const monthPacing = $derived.by(() => {
    const now = new SvelteDate();
    const currentYearMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
    const selected = budgetState.selectedMonth;

    if (selected < currentYearMonth) {
      return { percent: 100, isCurrentMonth: false, label: '100%' };
    }
    if (selected > currentYearMonth) {
      return { percent: 0, isCurrentMonth: false, label: '0%' };
    }

    const currentDay = now.getDate();
    const daysInCurMonth = new SvelteDate(now.getFullYear(), now.getMonth() + 1, 0).getDate();
    const percent = Math.min(100, Math.round((currentDay / daysInCurMonth) * 100));
    return {
      percent,
      isCurrentMonth: true,
      currentDay,
      daysInCurMonth,
      label: `${currentDay}/${daysInCurMonth} (${percent}%)`,
    };
  });

  function getPacingStatus(
    assigned: number,
    activity: number,
    available: number
  ): {
    tone: 'ok' | 'warn' | 'err' | 'neutral';
    label: string;
  } {
    if (assigned <= 0) {
      return { tone: 'neutral', label: '—' };
    }
    if (available < 0) {
      return { tone: 'err', label: i18n.t.pacingExceeded };
    }
    const usagePercent = Math.round((activity / assigned) * 100);
    const diff = usagePercent - monthPacing.percent;

    if (diff > 12) {
      return { tone: 'warn', label: i18n.t.pacingAhead };
    }
    if (diff < -15) {
      return { tone: 'ok', label: i18n.t.pacingUnder };
    }
    return { tone: 'neutral', label: i18n.t.pacingOnTrack };
  }

  const totalPacing = $derived(
    getPacingStatus(budgetData.totalAssigned, budgetData.totalActivity, totalAvailable)
  );
</script>

<div class="sharp-card flex w-full flex-1 flex-col overflow-y-auto">
  <table class="sharp-table">
    <thead>
      <tr>
        <th class="text-left">
          <span>{i18n.t.budgetCategoryEnvelope}</span>
          {#if monthPacing.isCurrentMonth}
            <span class="text-text-dim ml-2 text-smaller font-normal tracking-normal lowercase">
              ({monthPacing.label}
              {i18n.t.monthProgressLabel})
            </span>
          {/if}
        </th>
        <th class="w-44 text-right">{i18n.t.budgetAssignedThisMonth}</th>
        <th class="w-32 text-right">{i18n.t.budgetActivity}</th>
        <th class="w-32 text-right">{i18n.t.budgetAvailable}</th>
        <th class="w-28 text-center">{i18n.t.pacingCol}</th>
      </tr>
    </thead>
    <tbody>
      {#each budgetData.envelopes as env (env.accountId)}
        {@const usagePercent =
          env.assigned > 0
            ? Math.min(100, Math.round((env.activity / env.assigned) * 100))
            : env.activity > 0
              ? 100
              : 0}
        {@const pacing = getPacingStatus(env.assigned, env.activity, env.available)}
        {@const tooltipContent = [
          `${i18n.t.budgetAssignedLabel ?? 'Assigned'}: ${formatMinorGrouping(env.assigned, currency)}`,
          `${i18n.t.expense}: ${formatMinorGrouping(env.activity, currency)}`,
          `${i18n.t.net}: ${formatMinorGrouping(env.available, currency)}`,
          `${usagePercent}%`
        ].join(' · ')}
        <tr class="hover:bg-bg-row-active transition-colors group">
          <td>
            <Tooltip content={tooltipContent} placement="right" delay={200} class="block w-full">
              <div class="flex flex-col gap-1 pr-2">
                <div class="flex items-center gap-2">
                  <span class="text-text-muted font-proto text-smaller">{env.accountCode || '—'}</span
                  >
                  <span class="text-text-strong group-hover:text-teal truncate font-medium transition-colors">{env.accountName}</span>
                </div>
                <ProgressBar
                  value={usagePercent}
                  tone={env.available < 0 ? 'expense' : usagePercent >= 90 ? 'warning' : 'teal'}
                  track="line"
                  size="xs"
                />
              </div>
            </Tooltip>

          </td>
          <td class="text-right">
            <div
              class="border-line bg-bg-app focus-within:border-teal relative ml-auto flex w-40 items-center border transition-colors"
            >
              <span class="text-text-dim font-proto text-smaller pl-2 select-none">{currencySymbol}</span>
              <input
                type="text"
                inputmode="decimal"
                value={env.assigned > 0 ? String(fromMinor(currency, env.assigned)) : ''}
                placeholder="0"
                onchange={(e) => {
                  const val = e.currentTarget.value;
                  const minor = parseStringAmountToMinor(val, currency);
                  onAssignBudget(env.accountId, minor);
                }}
                class="text-text-strong font-proto text-small w-full bg-transparent px-2 py-1 text-right font-bold tabular-nums focus:outline-none"
              />
            </div>
          </td>
          <td class="text-text-muted font-proto text-right tabular-nums">
            {formatMinorGrouping(env.activity, currency)}
          </td>
          <td class="font-proto text-right font-bold tabular-nums">
            <Badge
              size="m"
              tone={env.available > 0 ? 'ok' : env.available === 0 ? 'neutral' : 'err'}
            >
              {formatMinorGrouping(env.available, currency)}
            </Badge>
          </td>
          <td class="font-proto text-center">
            {#if env.assigned > 0}
              <div class="flex flex-col items-center gap-0.5">
                <Badge size="s" tone={pacing.tone}>
                  {pacing.label}
                </Badge>
                <span class="text-text-dim text-smaller tabular-nums">
                  {usagePercent}% / {monthPacing.percent}%
                </span>
              </div>
            {:else}
              <span class="text-text-muted text-smaller">—</span>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
    <tfoot>
      <tr>
        <td class="text-text-strong text-right font-bold tracking-widest uppercase">
          {i18n.t.totals}
        </td>
        <td class="text-teal font-proto text-right font-bold tabular-nums">
          {formatMinorGrouping(budgetData.totalAssigned, currency)}
        </td>
        <td class="text-expense font-proto text-right font-bold tabular-nums">
          {formatMinorGrouping(budgetData.totalActivity, currency)}
        </td>
        <td class="font-proto text-right font-bold tabular-nums">
          <Badge
            size="m"
            tone={totalAvailable > 0 ? 'ok' : totalAvailable === 0 ? 'neutral' : 'err'}
          >
            {formatMinorGrouping(totalAvailable, currency)}
          </Badge>
        </td>
        <td class="font-proto text-center">
          <Badge size="s" tone={totalPacing.tone}>
            {totalPacing.label}
          </Badge>
        </td>
      </tr>
    </tfoot>
  </table>
</div>
