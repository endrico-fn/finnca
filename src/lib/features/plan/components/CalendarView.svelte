<script lang="ts">
  import type { Account, JournalEntryView } from '$lib/core/ipc/bindings';
  import type { CalendarDay } from '$lib/core/format/date';
  import type { PaymentPlan, DateEvents } from '../state/plan.svelte';
  import CalendarGrid, { type CalendarNavigation } from './CalendarGrid.svelte';
  import CalendarAgenda from './CalendarAgenda.svelte';

  let {
    selectedDate = $bindable(),
    eventsByDate,
    calendarDays,
    nav,
    selectedDateRelative,
    accountsById,
    entries,
    postInstallment,
    postingBusyId,
    openCreatePlan,
  }: {
    selectedDate: string;
    eventsByDate: Map<string, DateEvents>;
    calendarDays: CalendarDay[];
    nav: CalendarNavigation;
    selectedDateRelative: string;
    accountsById?: Map<string, Account>;
    entries?: JournalEntryView[];
    postInstallment: (p: PaymentPlan) => void;
    postingBusyId: string | null;
    openCreatePlan?: () => void;
  } = $props();

  const selectedDayEvents = $derived(eventsByDate.get(selectedDate) ?? { txs: [], plans: [] });
</script>

<div class="grid min-h-0 flex-1 grid-cols-12 gap-2 overflow-hidden">
  <CalendarGrid {calendarDays} {eventsByDate} bind:selectedDate {nav} />

  <CalendarAgenda
    {selectedDate}
    {selectedDateRelative}
    events={selectedDayEvents}
    {accountsById}
    {entries}
    {postInstallment}
    {postingBusyId}
    {openCreatePlan}
  />
</div>
