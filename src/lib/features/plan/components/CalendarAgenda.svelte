<script lang="ts">
  import type { Account, JournalEntryView } from '$lib/core/ipc/bindings';
  import { formatIDR, formatMinorToDisplay } from '$lib/core/format/currency';
  import { todayString } from '$lib/core/format/date';
  import { i18n } from '$lib/core/i18n.svelte';
  import type { PaymentPlan, DateEvents } from '../state/plan.svelte';
  import { isPlanPostedOnDate } from '../planUtils';
  import { computeDayCashflowTotals, getTxPositiveTotal } from '../state/planCalendarUtils';
  import { Icon, Badge, Card, Button } from '$lib/components/ui';

  let {
    selectedDate,
    selectedDateRelative,
    events = { txs: [], plans: [] },
    accountsById = new Map<string, Account>(),
    entries = [],
    postInstallment,
    postingBusyId,
    openCreatePlan,
  }: {
    selectedDate: string;
    selectedDateRelative: string;
    events?: DateEvents;
    accountsById?: Map<string, Account>;
    entries?: JournalEntryView[];
    postInstallment: (p: PaymentPlan) => void;
    postingBusyId: string | null;
    openCreatePlan?: () => void;
  } = $props();

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

  const totals = $derived(computeDayCashflowTotals(events.plans));
</script>

<Card class="col-span-4 flex min-h-0 flex-col" padding={false}>
  {#snippet header()}
    <div class="flex min-h-7 w-full items-center justify-between">
      <h3 class="label-title truncate leading-none uppercase">{selWeekday}</h3>
      {#if isSelectedToday}
        <Badge size="m" tone="ok">{i18n.t.today}</Badge>
      {:else}
        <span class="text-text-muted font-proto text-smaller leading-none uppercase">
          {selectedDateRelative}
        </span>
      {/if}
    </div>
  {/snippet}

  <div class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-3 pt-2.5 pb-2.5">
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

      <div class="grid grid-cols-3 gap-1.5">
        <div class="sharp-card bg-income/5 border-income/20 flex flex-col gap-1 p-2">
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-smaller truncate tracking-wider uppercase">
              {i18n.t.planExpectedIn}
            </span>
          </div>
          <span
            class="text-income font-proto text-small truncate leading-none font-bold tabular-nums"
          >
            {totals.totalIn > 0 ? `+${formatIDR(totals.totalIn)}` : formatIDR(0)}
          </span>
        </div>

        <div class="sharp-card bg-expense/5 border-expense/20 flex flex-col gap-1 p-2">
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-smaller truncate tracking-wider uppercase">
              {i18n.t.planExpectedOut}
            </span>
          </div>
          <span
            class="text-expense font-proto text-small truncate leading-none font-bold tabular-nums"
          >
            {totals.totalOut > 0 ? `-${formatIDR(totals.totalOut)}` : formatIDR(0)}
          </span>
        </div>

        <div class="sharp-card bg-bg-app border-line flex flex-col gap-1 p-2">
          <div class="flex items-center gap-1.5">
            <span
              class="text-text-strong text-smaller truncate font-medium tracking-wider uppercase"
            >
              {i18n.t.planNetEstimate}
            </span>
          </div>
          <span
            class="font-proto text-small truncate leading-none font-bold tabular-nums {totals.net >
            0
              ? 'text-income'
              : totals.net < 0
                ? 'text-expense'
                : 'text-text-muted'}"
          >
            {totals.net > 0
              ? `+${formatIDR(totals.net)}`
              : totals.net < 0
                ? `-${formatIDR(Math.abs(totals.net))}`
                : formatIDR(0)}
          </span>
        </div>
      </div>
    </div>

    {#if events.plans.length === 0 && events.txs.length === 0}
      <div class="m-2 flex flex-1 flex-col items-center justify-center py-8 text-center opacity-80">
        <div class="text-text-dim mb-3"><Icon name="calendar" size={32} /></div>
        <p class="font-proto text-text-muted text-small mb-3 tracking-wide">
          {i18n.t.planNoDueOnDate}
        </p>
        {#if openCreatePlan}
          <Button variant="outline" size="sm" onclick={openCreatePlan}>
            {i18n.t.planAddForDate}
          </Button>
        {/if}
      </div>
    {:else}
      {#if events.plans.length > 0}
        <div>
          <p
            class="label-xs text-text-muted mb-2 flex items-center gap-1.5 tracking-wider uppercase"
          >
            <Icon name="bell" size={11} />
            {i18n.t.planScheduledEvents}
          </p>
          <div class="flex flex-col gap-1.5">
            {#each events.plans as p (p.id)}
              {@const isPosted = isPlanPostedOnDate(p, selectedDate, entries)}
              {@const fromAcc = accountsById.get(p.fromAccountId)}
              {@const toAcc = accountsById.get(p.toAccountId)}

              <div
                class="bg-bg-app border-line border p-2.5 transition-colors {p.type === 'RECEIVABLE'
                  ? 'border-l-income border-l'
                  : p.type === 'PAYABLE'
                    ? 'border-l-expense border-l'
                    : 'border-l-teal border-l'}"
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
                      <span class="max-w-20 truncate" title={fromAcc?.name}>
                        {fromAcc?.name ?? '—'}
                      </span>
                      <span class="text-text-dim font-proto text-smaller">→</span>
                      <span class="max-w-20 truncate" title={toAcc?.name}>
                        {toAcc?.name ?? '—'}
                      </span>
                    </span>
                  </div>
                  <div class="flex shrink-0 flex-col items-end gap-1">
                    <span class="font-proto text-text-white text-small">
                      {formatIDR(p.installmentAmount)}
                    </span>
                    {#if isPosted}
                      <Badge size="s" tone="ok">{i18n.t.planMarkPosted}</Badge>
                    {:else}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => postInstallment(p)}
                        disabled={postingBusyId === p.id}
                      >
                        {#if postingBusyId === p.id}
                          <span class="spinner-sm border-teal"></span>
                        {:else}
                          {i18n.t.recordEntry}
                        {/if}
                      </Button>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if events.txs.length > 0}
        <div>
          <p
            class="label-xs text-text-muted mt-1 mb-2 flex items-center gap-1.5 tracking-wider uppercase"
          >
            <Icon name="wallet" size={11} />
            {i18n.t.planRecordedTxs} ({events.txs.length})
          </p>
          <div class="flex flex-col gap-1.5">
            {#each events.txs as tx (tx.id)}
              {@const total = getTxPositiveTotal(tx)}
              <div
                class="bg-bg-app border-line text-small flex items-center justify-between border p-2"
              >
                <span class="text-text-base mr-3 truncate">{tx.description}</span>
                <span class="font-proto text-text-dim shrink-0">
                  {formatMinorToDisplay(total, tx.currency)}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</Card>
