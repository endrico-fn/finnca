<script lang="ts">
  import { SvelteDate } from 'svelte/reactivity';
  import {
    formatMinorGrouping,
    fromMinor,
    parseStringAmountToMinor,
  } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, Tooltip } from '$lib/components/ui';
  import { budgetState, type EnvelopeData, type MonthCalculation } from '../state/budget.svelte';

  let {
    budgetData,
    envelopes: propEnvelopes,
    totalAssigned: propTotalAssigned,
    totalActivity: propTotalActivity,
    totalAvailable: propTotalAvailable,
    totalCount: propTotalCount,
    currency = 'IDR',
    onAssignBudget,
  }: {
    budgetData?: MonthCalculation;
    envelopes?: EnvelopeData[];
    totalAssigned?: number;
    totalActivity?: number;
    totalAvailable?: number;
    totalCount?: number;
    currency?: string;
    onAssignBudget: (accountId: string, amountMinor: number) => Promise<void>;
  } = $props();

  const envelopes = $derived(propEnvelopes ?? budgetData?.envelopes ?? []);
  const totalAssigned = $derived(
    propTotalAssigned ?? budgetData?.totalAssigned ?? envelopes.reduce((s, e) => s + e.assigned, 0)
  );
  const totalActivity = $derived(
    propTotalActivity ?? budgetData?.totalActivity ?? envelopes.reduce((s, e) => s + e.activity, 0)
  );
  const totalAvailable = $derived(
    propTotalAvailable ??
      budgetData?.envelopes.reduce((sum, e) => sum + e.available, 0) ??
      envelopes.reduce((s, e) => s + e.available, 0)
  );
  const totalCount = $derived(propTotalCount ?? budgetData?.envelopes.length ?? envelopes.length);

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

  const totalPacing = $derived(getPacingStatus(totalAssigned, totalActivity, totalAvailable));
</script>

<div class="min-h-0 flex-1 overflow-y-auto">
  <table class="sharp-table w-full border-x-0 border-t-0" spellcheck="false">
    <thead class="bg-bg-card sticky top-0 z-10">
      <tr class="border-line border-b">
        <th class="py-2 pl-3">
          <span>{i18n.t.budgetCategoryEnvelope}</span>
          {#if monthPacing.isCurrentMonth}
            <span class="text-text-dim text-smaller ml-2 font-normal tracking-normal lowercase">
              ({monthPacing.label}
              {i18n.t.monthProgressLabel})
            </span>
          {/if}
        </th>
        <th class="numeric w-44 px-3 py-2 text-right whitespace-nowrap"
          >{i18n.t.budgetAssignedThisMonth}</th
        >
        <th class="numeric w-32 px-3 py-2 text-right whitespace-nowrap">{i18n.t.budgetActivity}</th>
        <th class="numeric w-32 px-3 py-2 text-right whitespace-nowrap">{i18n.t.budgetAvailable}</th
        >
        <th class="center w-28 py-2 pr-3 whitespace-nowrap">{i18n.t.pacingCol}</th>
      </tr>
    </thead>
    <tbody>
      {#each envelopes as env (env.accountId)}
        {@const usagePercent =
          env.assigned > 0
            ? Math.min(100, Math.round((env.activity / env.assigned) * 100))
            : env.activity > 0
              ? 100
              : 0}
        {@const pacing = getPacingStatus(env.assigned, env.activity, env.available)}
        {@const tooltipContent = [
          `${i18n.t.budgetAssignedLabel}: ${formatMinorGrouping(env.assigned, currency)}`,
          `${i18n.t.expense}: ${formatMinorGrouping(env.activity, currency)}`,
          `${i18n.t.net}: ${formatMinorGrouping(env.available, currency)}`,
          `${usagePercent}%`,
        ].join(' · ')}
        <tr class="hover:bg-bg-row-active group cursor-default transition-colors">
          <td class="py-2 pl-3">
            <Tooltip
              content={tooltipContent}
              placement="right"
              delay={200}
              class="block w-full no-underline"
            >
              <div class="flex items-center gap-2 pr-2 no-underline">
                <span class="text-text-muted font-proto text-smaller shrink-0 select-none"
                  >{env.accountCode || '—'}</span
                >
                <span
                  class="text-text-strong group-hover:text-teal font-aux text-small truncate font-medium no-underline transition-colors select-none"
                  spellcheck="false">{env.accountName}</span
                >
              </div>
            </Tooltip>
          </td>
          <td class="numeric w-44 px-3 py-2 text-right">
            <div
              class="border-line bg-bg-app focus-within:border-teal relative ml-auto flex w-40 items-center border transition-colors"
            >
              <span
                class="text-text-dim font-proto text-smaller pl-2 font-bold uppercase select-none"
                >{currency || 'IDR'}</span
              >
              <input
                type="text"
                inputmode="decimal"
                spellcheck="false"
                autocomplete="off"
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
          <td
            class="numeric font-proto text-smaller text-text-muted w-32 px-3 py-2 text-right whitespace-nowrap tabular-nums"
          >
            {formatMinorGrouping(env.activity, currency)}
          </td>
          <td
            class="numeric font-proto text-smaller w-32 px-3 py-2 text-right font-bold whitespace-nowrap tabular-nums"
          >
            <Badge
              size="m"
              tone={env.available > 0 ? 'ok' : env.available === 0 ? 'neutral' : 'err'}
            >
              {formatMinorGrouping(env.available, currency)}
            </Badge>
          </td>
          <td class="center font-proto w-28 py-2 pr-3">
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
      {:else}
        <tr>
          <td colspan="5" class="text-text-muted font-aux text-small py-8 text-center">
            {totalCount === 0 ? i18n.t.noAccountsYet : i18n.t.noEnvelopesMatch}
          </td>
        </tr>
      {/each}
    </tbody>
    {#if envelopes.length > 0}
      <tfoot class="border-line bg-bg-card font-proto sticky bottom-0 z-10 border-t-2 font-bold">
        <tr>
          <td class="text-text-strong py-2 pl-3 font-bold tracking-widest uppercase">
            {i18n.t.totals}
          </td>
          <td
            class="numeric text-teal font-proto text-smaller w-44 px-3 py-2 text-right font-bold tabular-nums"
          >
            {formatMinorGrouping(totalAssigned, currency)}
          </td>
          <td
            class="numeric text-expense font-proto text-smaller w-32 px-3 py-2 text-right font-bold tabular-nums"
          >
            {formatMinorGrouping(totalActivity, currency)}
          </td>
          <td
            class="numeric font-proto text-smaller w-32 px-3 py-2 text-right font-bold tabular-nums"
          >
            <Badge
              size="m"
              tone={totalAvailable > 0 ? 'ok' : totalAvailable === 0 ? 'neutral' : 'err'}
            >
              {formatMinorGrouping(totalAvailable, currency)}
            </Badge>
          </td>
          <td class="center font-proto w-28 py-2 pr-3">
            <Badge size="s" tone={totalPacing.tone}>
              {totalPacing.label}
            </Badge>
          </td>
        </tr>
      </tfoot>
    {/if}
  </table>
</div>
