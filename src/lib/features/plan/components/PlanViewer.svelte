<script lang="ts">
  import {
    listAccountsCmd,
    listJournalEntriesCmd,
    type Account,
    type JournalEntryView,
  } from '$lib/core/ipc/bindings';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { createTransferDraft } from '$lib/features/journal/state/journalFormUtils';
  import type { Currency } from '$lib/core/types';
  import {
    buildCalendarDays,
    todayString,
    diffCalendarDays,
    formatMonthLabel,
  } from '$lib/core/format/date';
  import { onMount } from 'svelte';
  import { Splash, ErrorState, PageLayout, Tabs, Button } from '$lib/components/ui';
  import { createTabRouter } from '$lib/core/router/tabRouter.svelte';

  import PlanModal from './PlanModal.svelte';
  import CalendarView from './CalendarView.svelte';
  import PlansOverview from './PlansOverview.svelte';
  import { planState, buildEventsByDate, type PaymentPlan } from '../state/plan.svelte';

  const validTabs = ['calendar', 'plans'] as const;
  type PlanTab = (typeof validTabs)[number];

  const tabRouter = createTabRouter<PlanTab>('calendar', validTabs, 'view');

  const currentTabLabel = $derived(
    tabRouter.current === 'calendar' ? i18n.t.planTabCalendar : i18n.t.planTabProgress
  );

  let accounts = $state<Account[]>([]);
  let recentEntries = $state<JournalEntryView[]>([]);
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));

  onMount(() => {
    Promise.all([
      listAccountsCmd().catch(() => []),
      listJournalEntriesCmd(50).catch(() => []),
      planState.load(),
    ]).then(([accs, entries]) => {
      accounts = accs.map((i) => i.account);
      recentEntries = entries;
    });

    const unsub = eventBus.on('transaction:posted', async () => {
      recentEntries = await listJournalEntriesCmd(50).catch(() => []);
      await planState.load();
    });
    return unsub;
  });

  let currentYear = $state(new Date().getFullYear());
  let currentMonth = $state(new Date().getMonth());
  let selectedDate = $state(todayString());
  let planFilter = $state<string>('ALL');
  let planModalOpen = $state(false);
  let postingBusyId = $state<string | null>(null);

  function prevMonth() {
    if (currentMonth === 0) {
      currentMonth = 11;
      currentYear--;
    } else currentMonth--;
  }
  function nextMonth() {
    if (currentMonth === 11) {
      currentMonth = 0;
      currentYear++;
    } else currentMonth++;
  }
  function goToday() {
    const d = new Date();
    currentYear = d.getFullYear();
    currentMonth = d.getMonth();
    selectedDate = todayString(d);
  }

  const monthName = $derived(formatMonthLabel(currentYear, currentMonth, i18n.locale));
  const dayHeaders = $derived(i18n.t.dayHeaders);
  const calendarDays = $derived(buildCalendarDays(currentYear, currentMonth));

  const eventsByDate = $derived.by(() => {
    return buildEventsByDate(calendarDays, recentEntries, planState.plans);
  });

  const calendarNav = $derived({
    monthName,
    currentYear,
    dayHeaders,
    prevMonth,
    nextMonth,
    goToday,
  });

  const selectedDateRelative = $derived.by(() => {
    const today = todayString();
    const diff = diffCalendarDays(selectedDate, today);
    if (diff === 0) return i18n.t.today;
    if (diff === 1) return i18n.t.planTomorrow;
    if (diff === -1) return i18n.t.planYesterday;
    if (diff > 1) return i18n.t.planDaysAhead.replace('{days}', String(diff));
    return i18n.t.planDaysAgo.replace('{days}', String(Math.abs(diff)));
  });

  async function postInstallment(p: PaymentPlan) {
    postingBusyId = p.id;
    try {
      const fromAcc = accountsById.get(p.fromAccountId);
      const toAcc = accountsById.get(p.toAccountId);
      if (!fromAcc || !toAcc) return;
      if (fromAcc.currency !== toAcc.currency) throw new Error(i18n.t.planCurrencyMismatch);
      const draft = createTransferDraft({
        fromId: p.fromAccountId,
        toId: p.toAccountId,
        amount: p.installmentAmount,
        currency: (fromAcc.currency as Currency) || 'IDR',
        description: `${i18n.t.planInstallmentFor} ${p.title}`,
        notes: `[Plan: ${p.id}]`,
        date: selectedDate,
      });
      modalState.openInspector({ draft, isNew: true });
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.planError,
        message: String(e),
      });
    } finally {
      postingBusyId = null;
    }
  }
</script>

<PageLayout crumb={i18n.t.plan} crumbHref="/app/plan" title={currentTabLabel} class="select-none">
  {#snippet actions()}
    <Button
      variant="primary"
      class="font-proto text-small h-8 px-2.5 font-bold tracking-wider whitespace-nowrap"
      title={i18n.t.newPlanTracker}
      ariaLabel={i18n.t.newPlanTracker}
      onclick={() => (planModalOpen = true)}
    >
      {i18n.t.newPlanTracker}
    </Button>
  {/snippet}

  {#if planState.loading && planState.items.length === 0}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if planState.error}
    <ErrorState message={planState.error} onRetry={() => planState.load()} />
  {:else}
    <div class="border-line mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2">
      <div class="flex min-w-0 items-center gap-1">
        <Tabs
          tabs={[
            { id: 'calendar', label: i18n.t.planTabCalendar },
            { id: 'plans', label: i18n.t.planTabProgress },
          ]}
          active={tabRouter.current}
          onSelect={(id) => tabRouter.setTab(id as PlanTab)}
        />
      </div>
    </div>

    {#if tabRouter.current === 'calendar'}
      <CalendarView
        bind:selectedDate
        {eventsByDate}
        {calendarDays}
        nav={calendarNav}
        {selectedDateRelative}
        {accountsById}
        entries={recentEntries}
        {postInstallment}
        {postingBusyId}
        openCreatePlan={() => (planModalOpen = true)}
      />
    {:else}
      <PlansOverview bind:planFilter {accountsById} openCreatePlan={() => (planModalOpen = true)} />
    {/if}
  {/if}

  <PlanModal bind:open={planModalOpen} {selectedDate} {accounts} />
</PageLayout>
