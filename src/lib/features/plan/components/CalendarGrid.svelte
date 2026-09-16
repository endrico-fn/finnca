<script lang="ts">
  import { todayString, type CalendarDay } from '$lib/core/format/date';
  import { i18n } from '$lib/core/i18n.svelte';
  import type { DateEvents } from '../state/plan.svelte';
  import { getDayCashflow, formatCompact } from '../state/planCalendarUtils';
  import { Icon, Button, Badge, Card } from '$lib/components/ui';

  export interface CalendarNavigation {
    monthName: string;
    dayHeaders: string[];
    currentYear?: number;
    prevMonth: () => void;
    nextMonth: () => void;
    goToday: () => void;
  }

  let {
    calendarDays,
    eventsByDate,
    selectedDate = $bindable(),
    nav,
  }: {
    calendarDays: CalendarDay[];
    eventsByDate: Map<string, DateEvents>;
    selectedDate: string;
    nav: CalendarNavigation;
  } = $props();

  const todayStr = todayString();
  const rowCount = $derived(calendarDays.length / 7);
</script>

<Card class="col-span-8 flex min-h-0 flex-col" padding={false}>
  {#snippet header()}
    <div class="flex min-h-7 w-full items-center justify-between">
      <div class="flex items-baseline gap-2">
        <h2
          class="font-proto text-text-strong text-large leading-none font-bold tracking-tight uppercase tabular-nums"
        >
          {nav.monthName}
          {#if nav.currentYear}
            <span class="text-text-muted ml-2">{nav.currentYear}</span>
          {/if}
        </h2>
      </div>
      <div class="flex items-center gap-1.5">
        <Button
          variant="pager"
          size="icon"
          onclick={nav.prevMonth}
          ariaLabel={i18n.t.prevMonth}
          title={i18n.t.prevMonth}
        >
          <Icon name="chev-left" size={14} />
        </Button>
        <Button variant="pager" onclick={nav.goToday}>
          {i18n.t.today}
        </Button>
        <Button
          variant="pager"
          size="icon"
          onclick={nav.nextMonth}
          ariaLabel={i18n.t.nextMonth}
          title={i18n.t.nextMonth}
        >
          <Icon name="chev-right" size={14} />
        </Button>
      </div>
    </div>
  {/snippet}

  <div
    class="border-line bg-bg-app mx-3 mt-2 mb-2.5 flex min-h-0 flex-1 flex-col overflow-hidden border"
  >
    <div class="border-line bg-bg-card grid shrink-0 grid-cols-7 border-b">
      {#each nav.dayHeaders as dh (dh)}
        <div class="font-proto text-text-muted text-smaller py-2 text-center tracking-widest">
          {dh}
        </div>
      {/each}
    </div>

    <div
      class="bg-line grid min-h-0 flex-1 grid-cols-7 gap-px"
      style="grid-template-rows: repeat({rowCount}, minmax(0, 1fr));"
    >
      {#each calendarDays as day (day.dateStr)}
        {@const evts = eventsByDate.get(day.dateStr)}
        {@const flow = getDayCashflow(evts)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          onclick={() => (selectedDate = day.dateStr)}
          class="group relative flex h-full min-h-0 cursor-pointer flex-col p-1.5 transition-colors
          {day.dateStr === selectedDate
            ? 'bg-bg-row-active outline-teal z-10 outline outline-1 -outline-offset-1'
            : !day.isCurrentMonth
              ? 'bg-bg-app text-text-dim hover:bg-bg-btn'
              : 'bg-bg-card hover:bg-bg-btn'}
          "
        >
          <div class="mb-1 flex shrink-0 items-start justify-between gap-1">
            <span
              class="font-proto text-small flex h-6 min-w-6 items-center justify-center px-1 transition-colors
              {day.dateStr === todayStr
                ? 'bg-teal text-bg-app font-bold'
                : day.dateStr === selectedDate
                  ? 'text-teal font-semibold'
                  : !day.isCurrentMonth
                    ? 'text-text-dim'
                    : 'text-text-base'}"
            >
              {day.dayNum === 1
                ? `${day.dayNum} ${new Date(day.dateStr + 'T12:00:00').toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', { month: 'short' }).toUpperCase()}`
                : day.dayNum}
            </span>

            <div class="flex flex-col items-end gap-0.5">
              {#if flow.in > 0}
                <Badge size="s" tone="income">+{formatCompact(flow.in)}</Badge>
              {/if}
              {#if flow.out > 0}
                <Badge size="s" tone="expense">-{formatCompact(flow.out)}</Badge>
              {/if}
            </div>
          </div>

          <div class="no-scrollbar mt-0.5 flex flex-1 flex-col gap-0.5 overflow-y-auto">
            {#if evts}
              {#each evts.plans.slice(0, 2) as p (p.id)}
                <div
                  class="font-proto text-smaller truncate border-l px-1.5 py-0.5 {p.type ===
                  'RECEIVABLE'
                    ? 'border-income bg-income/10 text-income'
                    : 'border-expense bg-expense/10 text-expense'}"
                  title={p.title}
                >
                  {p.title}
                </div>
              {/each}
              {#if evts.plans.length > 2}
                <div class="text-text-dim font-proto text-smaller px-1 italic">
                  +{evts.plans.length - 2}
                  {i18n.t.calendarMoreLabel}
                </div>
              {/if}
              {#if evts.txs.length > 0}
                <Badge size="s" tone="neutral" class="mt-auto self-end">
                  {i18n.t.planTxCount.replace('{count}', String(evts.txs.length))}
                </Badge>
              {/if}
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <div
      class="bg-bg-card border-line font-proto text-text-muted text-smaller flex shrink-0 items-center gap-4 border-t px-3 py-1.5"
    >
      <div class="flex items-center gap-1.5">
        <div class="bg-income h-2 w-2"></div>
        <span>{i18n.t.planLegendReceivable}</span>
      </div>
      <div class="flex items-center gap-1.5">
        <div class="bg-expense h-2 w-2"></div>
        <span>{i18n.t.planLegendPayable}</span>
      </div>
      <div class="flex items-center gap-1.5">
        <div class="border-line bg-bg-app h-2 w-2 border"></div>
        <span>{i18n.t.planLegendRecorded}</span>
      </div>
    </div>
  </div>
</Card>
