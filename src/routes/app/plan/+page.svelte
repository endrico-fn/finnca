<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { notifStore } from '$lib/notifications/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    buildCalendarDays,
    buildEventsByDate,
    formatIDR,
    calculatePlanProgress,
    isPlanPostedOnDate as checkPlanPosted,
    todayString,
    diffCalendarDays,
  } from '$lib/accounting/finance';
  import type { PaymentPlan, Transaction } from '$lib/accounting/types';
  import TransactionEditor from '$lib/components/TransactionEditor.svelte';
  import { onMount } from 'svelte';
  import {
    Splash,
    ErrorState,
    PageLayout,
    Tabs,
    Button,
    Icon,
    ModalShell,
  } from '$lib/components/ui';

  import PlanModal from './_components/PlanModal.svelte';
  import CalendarView from './_views/CalendarView.svelte';
  import PlansOverview from './_views/PlansOverview.svelte';

  onMount(() => {
    if (!ledger.data) ledger.load();
  });

  let currentYear = $state(new Date().getFullYear());
  let currentMonth = $state(new Date().getMonth());
  let selectedDate = $state(todayString());
  let viewMode = $state<'calendar' | 'plans'>('calendar');
  let planFilter = $state<string>('ALL');
  let planModalOpen = $state(false);
  let postingBusyId = $state<string | null>(null);
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
    `${String(currentMonth + 1).padStart(2, '0')} ${new Date(currentYear, currentMonth, 1)
      .toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', { month: 'long' })
      .toUpperCase()}`
  );
  const dayHeaders = $derived(i18n.t.dayHeaders);

  const calendarDays = $derived(buildCalendarDays(currentYear, currentMonth));

  const eventsByDate = $derived.by(() => {
    return buildEventsByDate(calendarDays, ledger.transactions, ledger.plans);
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

  $effect(() => {
    if (!ledger.data || ledger.plans.length === 0) return;
    const today = todayString();

    for (const p of ledger.plans) {
      if (p.status === 'ARCHIVED' || p.status === 'COMPLETED') continue;

      const prog = calculatePlanProgress(p, ledger.data);
      if (prog.isSettled) continue;

      const isPostedToday = checkPlanPosted(p, today, ledger.transactions);

      let isDueToday = false;
      if (p.frequency === 'DAILY') isDueToday = true;
      else if (p.frequency === 'MONTHLY' && p.dayOfMonth === new Date().getDate())
        isDueToday = true;
      else if (p.dueDate === today) isDueToday = true;

      if (isDueToday && !isPostedToday) {
        notifStore.addNotification({
          type: 'DUE_DATE',
          priority: 'high',
          title: i18n.t.planDueAlertTitle,
          message: i18n.t.planDueAlertMsg
            .replace('{title}', p.title)
            .replace('{amount}', formatIDR(p.installmentAmount)),
          actionHref: '/app/plan',
          actionLabel: i18n.t.plan,
        });
      }

      if (prog.progressPercent >= 90 && prog.progressPercent < 100) {
        notifStore.addNotification({
          type: 'DUE_DATE',
          priority: 'low',
          title: i18n.t.planNearCompleteTitle,
          message: i18n.t.planNearCompleteMsg
            .replace('{title}', p.title)
            .replace('{percent}', String(prog.progressPercent)),
        });
      }
    }
  });

  async function postInstallment(p: PaymentPlan) {
    postingBusyId = p.id;
    try {
      const fromAcc = ledger.accountsById.get(p.fromAccountId);
      const toAcc = ledger.accountsById.get(p.toAccountId);
      if (!fromAcc || !toAcc) return;
      if (fromAcc.currency !== toAcc.currency) throw new Error(i18n.t.planCurrencyMismatch);
      quickTxDraft = {
        id: '',
        date: selectedDate,
        description: `${i18n.t.planInstallmentFor} ${p.title}`,
        planId: p.id,
        currency: fromAcc.currency,
        splits: [
          { id: '', accountId: p.fromAccountId, amount: -p.installmentAmount, reconcile: 'n' },
          { id: '', accountId: p.toAccountId, amount: p.installmentAmount, reconcile: 'n' },
        ],
      };
    } catch (e) {
      notifStore.addNotification({
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

<PageLayout title={i18n.t.planAndCalendar} class="select-none">
  {#snippet actions()}
    <Button variant="primary" onclick={() => (planModalOpen = true)}>
      <span class="inline-flex items-center gap-1.5">
        <Icon name="plus" size={12} />
        {i18n.t.newPlanTracker}
      </span>
    </Button>
  {/snippet}

  {#if ledger.loading}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if ledger.error}
    <ErrorState message={ledger.error} onRetry={() => ledger.load()} />
  {:else}
    <div class="border-line mb-2 flex shrink-0 items-center justify-between gap-2 border-b pb-2">
      <div class="flex min-w-0 items-center gap-1">
        <Tabs
          tabs={[
            { id: 'calendar', label: i18n.t.planTabCalendar },
            { id: 'plans', label: i18n.t.planTabProgress },
          ]}
          active={viewMode}
          onSelect={(id) => (viewMode = id as 'calendar' | 'plans')}
        />
      </div>
    </div>

    {#if viewMode === 'calendar'}
      <CalendarView
        bind:selectedDate
        {eventsByDate}
        {calendarDays}
        nav={calendarNav}
        {selectedDateRelative}
        {postInstallment}
        {postingBusyId}
        openCreatePlan={() => (planModalOpen = true)}
      />
    {:else}
      <PlansOverview bind:planFilter openCreatePlan={() => (planModalOpen = true)} />
    {/if}
  {/if}

  <PlanModal bind:open={planModalOpen} {selectedDate} />

  {#if quickTxDraft}
    <ModalShell
      bind:open={
        () => !!quickTxDraft,
        (v) => {
          if (!v) quickTxDraft = null;
        }
      }
      title={i18n.t.planTransactionPosted}
      maxWidth="max-w-4xl"
    >
      <TransactionEditor
        tx={quickTxDraft}
        onSave={async (savedTx) => {
          await ledger.upsertTransaction(savedTx);
          notifStore.addNotification({
            type: 'LEDGER_INTEGRITY',
            priority: 'low',
            title: i18n.t.planTransactionPosted,
            message: i18n.t.planInstallmentPostedMsg,
          });
          quickTxDraft = null;
        }}
        onCancel={() => (quickTxDraft = null)}
      />
    </ModalShell>
  {/if}
</PageLayout>
