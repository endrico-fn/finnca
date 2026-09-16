<script lang="ts">
  import {
    listAccountsCmd,
    listJournalEntriesCmd,
    postJournalEntryCmd,
    type Account,
    type JournalEntryView,
    type PostingInput,
  } from '$lib/core/ipc/bindings';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { buildCalendarDays, todayString, diffCalendarDays } from '$lib/core/format/date';
  import type { Transaction } from '$lib/core/types';
  import JournalEntryForm from '$lib/features/journal/components/JournalEntryForm.svelte';
  import { onMount } from 'svelte';
  import {
    Splash,
    ErrorState,
    PageLayout,
    Tabs,
    Button,
    ModalShell,
  } from '$lib/components/ui';
  import { createTabRouter } from '$lib/core/router/tabRouter.svelte';

  import PlanModal from './PlanModal.svelte';
  import CalendarView from './CalendarView.svelte';
  import PlansOverview from './PlansOverview.svelte';
  import {
    planState,
    buildEventsByDate,
    type PaymentPlan,
  } from '../state/plan.svelte';

  const validTabs = ['calendar', 'plans'] as const;
  type PlanTab = (typeof validTabs)[number];

  const tabRouter = createTabRouter<PlanTab>('calendar', validTabs, 'view');

  const currentTabLabel = $derived(
    tabRouter.current === 'calendar' ? i18n.t.planTabCalendar : i18n.t.planTabProgress
  );

  let accounts = $state<Account[]>([]);
  let recentEntries = $state<JournalEntryView[]>([]);
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));

  onMount(async () => {
    const [accs, entries] = await Promise.all([
      listAccountsCmd().catch(() => []),
      listJournalEntriesCmd(50).catch(() => []),
      planState.load(),
    ]);
    accounts = accs.map((i) => i.account);
    recentEntries = entries;
  });

  let currentYear = $state(new Date().getFullYear());
  let currentMonth = $state(new Date().getMonth());
  let selectedDate = $state(todayString());
  let planFilter = $state<string>('ALL');
  let planModalOpen = $state(false);
  let postingBusyId = $state<string | null>(null);
  let quickTxOpen = $state(false);
  let quickTxDraft = $state<Transaction | null>(null);

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

  const monthName = $derived(
    new Date(currentYear, currentMonth, 1)
      .toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', { month: 'long' })
      .toUpperCase()
  );
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
      quickTxDraft = {
        id: crypto.randomUUID(),
        date: selectedDate,
        description: `${i18n.t.planInstallmentFor} ${p.title}`,
        planId: p.id,
        currency: (fromAcc.currency as 'IDR' | 'USD') || 'IDR',
        splits: [
          {
            id: crypto.randomUUID(),
            accountId: p.fromAccountId,
            amount: -p.installmentAmount,
            reconcile: 'n',
          },
          {
            id: crypto.randomUUID(),
            accountId: p.toAccountId,
            amount: p.installmentAmount,
            reconcile: 'n',
          },
        ],
      };
      quickTxOpen = true;
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
      <PlansOverview
        bind:planFilter
        {accountsById}
        openCreatePlan={() => (planModalOpen = true)}
      />
    {/if}
  {/if}

  <PlanModal bind:open={planModalOpen} {selectedDate} {accounts} />

  <ModalShell
    bind:open={quickTxOpen}
    title={i18n.t.planTransactionPosted}
    maxWidth="max-w-5xl"
    onClose={() => {
      quickTxOpen = false;
      quickTxDraft = null;
    }}
  >
    {#if quickTxDraft}
      <JournalEntryForm
        tx={quickTxDraft}
        onSave={async (savedTx) => {
          const postings: PostingInput[] = savedTx.splits.map((s) => ({
            id: s.id || undefined,
            account_id: s.accountId,
            amount: Math.round(s.amount),
            memo: s.memo || null,
            action: null,
            reconcile: (s.reconcile === 'y' ? 'y' : s.reconcile === 'c' ? 'c' : null) as 'c' | 'y' | null,
          }));
          await postJournalEntryCmd({
            date: savedTx.date,
            description: savedTx.description,
            notes: savedTx.notes || null,
            currency: savedTx.currency,
            fx_rate: savedTx.fxRateAtTransaction || null,
            postings,
          });
          eventBus.emit('transaction:posted', { id: '' });
          eventBus.emit('accounts:changed', undefined);
          await planState.load();
          notificationState.addNotification({
            type: 'LEDGER_INTEGRITY',
            priority: 'low',
            title: i18n.t.planTransactionPosted,
            message: i18n.t.planInstallmentPostedMsg,
          });
          quickTxOpen = false;
          quickTxDraft = null;
        }}
        onCancel={() => {
          quickTxOpen = false;
          quickTxDraft = null;
        }}
      />
    {/if}
  </ModalShell>
</PageLayout>
