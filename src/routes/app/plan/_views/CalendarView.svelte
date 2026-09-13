<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { formatIDR, formatMoney, isPlanPostedOnDate, todayString } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import type { PaymentPlan, Transaction } from '$lib/accounting/types';
  import type { CalendarDay, DateEvents } from '$lib/accounting/finance';
  import { Icon, Badge, Button, Card } from '$lib/components/ui';

  interface CalendarNavigation {
    monthName: string;
    dayHeaders: string[];
    currentYear?: number;
    prevMonth: () => void;
    nextMonth: () => void;
    goToday: () => void;
  }

  let {
    selectedDate = $bindable(),
    eventsByDate,
    calendarDays,
    nav,
    selectedDateRelative,
    postInstallment,
    postingBusyId,
    openCreatePlan,
  } = $props<{
    selectedDate: string;
    eventsByDate: Map<string, DateEvents>;
    calendarDays: CalendarDay[];
    nav: CalendarNavigation;
    selectedDateRelative: string;
    postInstallment: (p: PaymentPlan) => void;
    postingBusyId: string | null;
    openCreatePlan?: () => void;
  }>();

  const selectedDayEvents = $derived(eventsByDate.get(selectedDate) ?? { txs: [], plans: [] });

  const selectedDayTotalIn = $derived(
    selectedDayEvents.plans
      .filter((p: PaymentPlan) => p.type === 'RECEIVABLE')
      .reduce((sum: number, p: PaymentPlan) => sum + p.installmentAmount, 0)
  );

  const selectedDayTotalOut = $derived(
    selectedDayEvents.plans
      .filter((p: PaymentPlan) => p.type === 'PAYABLE')
      .reduce((sum: number, p: PaymentPlan) => sum + p.installmentAmount, 0)
  );

  const selectedDayNet = $derived(selectedDayTotalIn - selectedDayTotalOut);

  function getDayCashflow(evts: DateEvents | undefined): { in: number; out: number } {
    if (!evts || evts.plans.length === 0) return { in: 0, out: 0 };
    let inTotal = 0;
    let outTotal = 0;
    for (const p of evts.plans) {
      if (p.type === 'RECEIVABLE') inTotal += p.installmentAmount;
      else if (p.type === 'PAYABLE') outTotal += p.installmentAmount;
    }
    return { in: inTotal, out: outTotal };
  }

  function getTxPositiveTotal(tx: Transaction): number {
    return tx.splits.filter((s) => s.amount > 0).reduce((acc, sp) => acc + sp.amount, 0);
  }

  const todayStr = todayString();
  const isSelectedToday = $derived(selectedDate === todayStr);

  const selDateObj = $derived.by(() => {
    const [y, m, d] = selectedDate.split('-').map(Number);
    return new Date(y, m - 1, d);
  });
  const selWeekday = $derived(
    selDateObj
      .toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
        weekday: 'long',
      })
      .toUpperCase()
  );

  const rowCount = $derived(calendarDays.length / 7);

  function formatCompact(minor: number): string {
    const n = Math.abs(minor);
    if (n >= 1_000_000_000) {
      const val = (n / 1_000_000_000).toFixed(1).replace(/\.0$/, '');
      return `${val}${i18n.t.compactBillion}`;
    }
    if (n >= 1_000_000) {
      const val = (n / 1_000_000).toFixed(1).replace(/\.0$/, '');
      return `${val}${i18n.t.compactMillion}`;
    }
    if (n >= 1_000) {
      const val = (n / 1_000).toFixed(0);
      return `${val}${i18n.t.compactThousand}`;
    }
    return String(n);
  }
</script>

<div class="grid min-h-0 flex-1 grid-cols-12 gap-2 overflow-hidden">
  <!-- Left Side: Calendar Grid -->
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
            title={i18n.t.prevMonth}><Icon name="chev-left" size={14} /></Button
          >
          <Button variant="pager" onclick={nav.goToday}>
            {i18n.t.today}
          </Button>
          <Button
            variant="pager"
            size="icon"
            onclick={nav.nextMonth}
            ariaLabel={i18n.t.nextMonth}
            title={i18n.t.nextMonth}><Icon name="chev-right" size={14} /></Button
          >
        </div>
      </div>
    {/snippet}

    <!-- Secondary Container: Calendar Table & Legend -->
    <div
      class="border-line bg-bg-app mx-3 mt-2 mb-2.5 flex min-h-0 flex-1 flex-col overflow-hidden border"
    >
      <!-- Headers -->
      <div class="border-line bg-bg-card grid shrink-0 grid-cols-7 border-b">
        {#each nav.dayHeaders as dh (dh)}
          <div class="font-proto text-text-muted text-smaller py-2 text-center tracking-widest">
            {dh}
          </div>
        {/each}
      </div>

      <!-- Cells -->
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
                  <span
                    class="font-proto text-income bg-income/10 border-income/20 text-smaller border px-1 py-px leading-none"
                  >
                    +{formatCompact(flow.in)}
                  </span>
                {/if}
                {#if flow.out > 0}
                  <span
                    class="font-proto text-expense bg-expense/10 border-expense/20 text-smaller border px-1 py-px leading-none"
                  >
                    -{formatCompact(flow.out)}
                  </span>
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
                  <div
                    class="text-text-muted font-proto border-line bg-bg-app text-smaller mt-auto self-end border px-1.5 py-0.5"
                  >
                    {i18n.t.planTxCount.replace('{count}', String(evts.txs.length))}
                  </div>
                {/if}
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <!-- Calendar Legend Footer inside Secondary Container -->
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

  <!-- Right Side: Agenda Panel -->
  <Card class="col-span-4 flex min-h-0 flex-col" padding={false}>
    {#snippet header()}
      <div class="flex min-h-7 w-full items-center justify-between">
        <h3 class="label-title truncate leading-none uppercase">{selWeekday}</h3>
        {#if isSelectedToday}
          <span class="badge-ok font-proto text-smaller px-1.5 py-0.5 leading-none uppercase">
            {i18n.t.today}
          </span>
        {:else}
          <span class="text-text-muted font-proto text-smaller leading-none uppercase">
            {selectedDateRelative}
          </span>
        {/if}
      </div>
    {/snippet}

    <!-- Agenda Content -->
    <div class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-3 pt-2.5 pb-2.5">
      <!-- Secondary Container: Daily Cash Flow Impact List -->
      <div class="font-proto flex shrink-0 flex-col gap-2">
        <div
          class="border-line/60 text-text-dim text-smaller flex items-center justify-between border-b pb-2 tracking-wider uppercase"
        >
          <span class="text-text-base flex items-center gap-1.5 font-medium">
            <Icon name="chart" size={10} />
            {i18n.t.planDailyCashflowImpact}
          </span>
          <span class="text-text-muted font-proto text-smaller tabular-nums">{selectedDate}</span>
        </div>

        <!-- [1] EXPECTED IN -->
        <div class="sharp-card bg-income/5 border-income/20 flex flex-col gap-1 p-2">
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-smaller tracking-wider uppercase"
              >{i18n.t.planExpectedIn}</span
            >
          </div>
          <span class="text-income font-proto text-small leading-none font-bold tabular-nums">
            {selectedDayTotalIn > 0 ? `+${formatIDR(selectedDayTotalIn)}` : formatIDR(0)}
          </span>
        </div>

        <!-- [2] EXPECTED OUT -->
        <div class="sharp-card bg-expense/5 border-expense/20 flex flex-col gap-1 p-2">
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-smaller tracking-wider uppercase"
              >{i18n.t.planExpectedOut}</span
            >
          </div>
          <span class="text-expense font-proto text-small leading-none font-bold tabular-nums">
            {selectedDayTotalOut > 0 ? `-${formatIDR(selectedDayTotalOut)}` : formatIDR(0)}
          </span>
        </div>

        <!-- [3] NET ESTIMATE -->
        <div class="sharp-card bg-bg-app border-line flex flex-col gap-1 p-2">
          <div class="flex items-center gap-1.5">
            <span class="text-text-strong text-smaller font-medium tracking-wider uppercase"
              >{i18n.t.planNetEstimate}</span
            >
          </div>
          <span
            class="font-proto text-small leading-none font-bold tabular-nums {selectedDayNet > 0
              ? 'text-income'
              : selectedDayNet < 0
                ? 'text-expense'
                : 'text-text-muted'}"
          >
            {selectedDayNet > 0
              ? `+${formatIDR(selectedDayNet)}`
              : selectedDayNet < 0
                ? `-${formatIDR(Math.abs(selectedDayNet))}`
                : formatIDR(0)}
          </span>
        </div>
      </div>
      {#if selectedDayEvents.plans.length === 0 && selectedDayEvents.txs.length === 0}
        <div
          class="m-2 flex flex-1 flex-col items-center justify-center py-8 text-center opacity-80"
        >
          <div class="text-text-dim mb-3"><Icon name="calendar" size={32} /></div>
          <p class="font-proto text-text-muted text-small mb-3 tracking-wide">
            {i18n.t.planNoDueOnDate}
          </p>
          {#if openCreatePlan}
            <button
              type="button"
              onclick={openCreatePlan}
              class="sharp-btn bg-teal/10 text-teal border-teal/30 hover:bg-teal hover:text-bg-app font-proto text-smaller border px-3 py-1.5 transition-all"
            >
              {i18n.t.planAddForDate}
            </button>
          {/if}
        </div>
      {:else}
        {#if selectedDayEvents.plans.length > 0}
          <div>
            <p
              class="label-xs text-text-muted mb-2 flex items-center gap-1.5 tracking-wider uppercase"
            >
              <Icon name="bell" size={11} />
              {i18n.t.planScheduledEvents}
            </p>
            <div class="flex flex-col gap-1.5">
              {#each selectedDayEvents.plans as p (p.id)}
                {@const isPosted = isPlanPostedOnDate(p, selectedDate, ledger.transactions)}
                {@const fromAcc = ledger.accountsById.get(p.fromAccountId)}
                {@const toAcc = ledger.accountsById.get(p.toAccountId)}

                <div
                  class="bg-bg-app border-line border p-2.5 transition-colors {p.type ===
                  'RECEIVABLE'
                    ? 'border-l-income border-l'
                    : 'border-l-expense border-l'}"
                >
                  <div class="flex items-start justify-between gap-2">
                    <div class="min-w-0 flex-1">
                      <span
                        class="font-proto text-text-strong text-small block truncate font-semibold"
                        title={p.title}
                      >
                        {p.title}
                      </span>
                      <span
                        class="text-text-muted font-proto text-smaller mt-1 flex items-center gap-1.5 truncate"
                      >
                        <span class="max-w-20 truncate" title={fromAcc?.name}
                          >{fromAcc?.name ?? '—'}</span
                        >
                        <span class="text-text-dim font-proto text-smaller">→</span>
                        <span class="max-w-20 truncate" title={toAcc?.name}
                          >{toAcc?.name ?? '—'}</span
                        >
                      </span>
                    </div>
                    <div class="flex shrink-0 flex-col items-end gap-1">
                      <span class="font-proto text-text-white text-small">
                        {formatIDR(p.installmentAmount)}
                      </span>
                      {#if isPosted}
                        <Badge tone="ok">{i18n.t.planMarkPosted}</Badge>
                      {:else}
                        <button
                          type="button"
                          onclick={() => postInstallment(p)}
                          disabled={postingBusyId === p.id}
                          class="sharp-btn bg-teal/10 text-teal border-teal/20 hover:bg-teal hover:text-bg-app text-smaller border px-2 py-0.5 transition-all"
                        >
                          {#if postingBusyId === p.id}
                            <span class="spinner-sm border-teal"></span>
                          {:else}
                            {i18n.t.recordEntry}
                          {/if}
                        </button>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if selectedDayEvents.txs.length > 0}
          <div>
            <p
              class="label-xs text-text-muted mt-1 mb-2 flex items-center gap-1.5 tracking-wider uppercase"
            >
              <Icon name="wallet" size={11} />
              {i18n.t.planRecordedTxs} ({selectedDayEvents.txs.length})
            </p>
            <div class="flex flex-col gap-1.5">
              {#each selectedDayEvents.txs as tx (tx.id)}
                {@const total = getTxPositiveTotal(tx)}
                <div
                  class="bg-bg-app border-line text-small flex items-center justify-between border p-2"
                >
                  <span class="text-text-base mr-3 truncate">{tx.description}</span>
                  <span class="font-proto text-text-dim shrink-0">
                    {formatMoney(total, tx.currency)}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </Card>
</div>
