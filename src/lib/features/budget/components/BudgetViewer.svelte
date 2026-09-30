<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteDate } from 'svelte/reactivity';
  import { shiftMonth, formatMonthLabel } from '$lib/core/format/date';
  import { budgetState, type BudgetMonthSummary } from '../state/budget.svelte';
  import { invokeIpc } from '$lib/core/ipc/client';
  import { PageLayout, Icon, Button, MonthPager, Card } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { listAccountsCmd } from '$lib/core/ipc/bindings';
  import BudgetSummaryCards from './BudgetSummaryCards.svelte';
  import BudgetEnvelopeTable from './BudgetEnvelopeTable.svelte';
  import BudgetFilterBar, { type BudgetFilterStatus } from './BudgetFilterBar.svelte';

  const dateObj = new SvelteDate();
  let currency = $state('IDR');
  let searchQuery = $state('');
  let filterStatus = $state<BudgetFilterStatus>('ALL');

  const currentMonth = $derived(
    dateObj.getFullYear() + '-' + String(dateObj.getMonth() + 1).padStart(2, '0')
  );

  const monthName = $derived(
    formatMonthLabel(dateObj.getFullYear(), dateObj.getMonth(), i18n.locale, true)
  );

  onMount(async () => {
    budgetState.setMonth(currentMonth);
    try {
      const items = await listAccountsCmd();
      if (items.length > 0 && items[0].account?.currency) {
        currency = items[0].account.currency;
      }
    } catch {
      currency = 'IDR';
    }
  });

  async function prevMonth() {
    dateObj.setMonth(dateObj.getMonth() - 1);
    await budgetState.setMonth(currentMonth);
  }

  async function nextMonth() {
    dateObj.setMonth(dateObj.getMonth() + 1);
    await budgetState.setMonth(currentMonth);
  }

  const budgetData = $derived(budgetState.monthCalculation);
  const totalAvailable = $derived(budgetData.envelopes.reduce((sum, e) => sum + e.available, 0));

  const filteredEnvelopes = $derived.by(() => {
    let list = budgetData.envelopes;
    const q = searchQuery.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (e) => e.accountName.toLowerCase().includes(q) || e.accountCode.toLowerCase().includes(q)
      );
    }
    if (filterStatus === 'funded') {
      list = list.filter((e) => e.assigned > 0);
    } else if (filterStatus === 'overspent') {
      list = list.filter((e) => e.available < 0);
    } else if (filterStatus === 'unassigned') {
      list = list.filter((e) => e.assigned === 0);
    }
    return list;
  });

  const statusCounts = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    const base = q
      ? budgetData.envelopes.filter(
          (e) => e.accountName.toLowerCase().includes(q) || e.accountCode.toLowerCase().includes(q)
        )
      : budgetData.envelopes;

    return {
      ALL: base.length,
      funded: base.filter((e) => e.assigned > 0).length,
      overspent: base.filter((e) => e.available < 0).length,
      unassigned: base.filter((e) => e.assigned === 0).length,
    };
  });

  function resetBudgetFilter() {
    searchQuery = '';
    filterStatus = 'ALL';
  }

  async function assignBudget(accountId: string, amountIdrMinor: number) {
    await budgetState.assignBudget(accountId, amountIdrMinor);
  }

  async function copyPreviousMonth() {
    const prevMonthStr = shiftMonth(currentMonth, -1);

    try {
      const prevSummary = await invokeIpc<BudgetMonthSummary>('get_budget_summary_cmd', {
        month: prevMonthStr,
      });

      const nonZero = prevSummary.envelopes.filter((e) => e.assigned > 0);
      if (nonZero.length === 0) {
        notificationState.addNotification({
          type: 'LEDGER_INTEGRITY',
          priority: 'low',
          title: i18n.t.budgetEnvelopeTitle,
          message: i18n.t.budgetRolloverEmpty,
        });
        return;
      }

      for (const b of nonZero) {
        await budgetState.assignBudget(b.account_id, b.assigned);
      }

      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.budgetEnvelopeTitle,
        message: i18n.t.budgetRolloverSuccess,
      });
    } catch {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.budgetEnvelopeTitle,
        message: i18n.t.budgetRolloverEmpty,
      });
    }
  }
</script>

<PageLayout title={i18n.t.budget}>
  {#snippet actions()}
    <div class="flex items-center gap-2">
      <Button
        variant="ghost"
        size="sm"
        onclick={copyPreviousMonth}
        title={i18n.t.budgetRolloverBtn}
        class="gap-1.5"
      >
        <Icon name="refresh" size={11} />
        {i18n.t.budgetRolloverBtn}
      </Button>

      <MonthPager variant="boxed" label={monthName} onPrev={prevMonth} onNext={nextMonth} />
    </div>
  {/snippet}

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    <BudgetSummaryCards {budgetData} {currency} />

    <Card
      title="{i18n.t.budgetEnvelopeTitle} ({filteredEnvelopes.length})"
      class="border-line bg-bg-card flex min-h-0 flex-1 flex-col overflow-hidden border"
      padding={false}
      borderHeader
    >
      <BudgetFilterBar
        bind:searchQuery
        bind:filterStatus
        {statusCounts}
        filteredCount={filteredEnvelopes.length}
        totalCount={budgetData.envelopes.length}
        onReset={resetBudgetFilter}
      />

      <BudgetEnvelopeTable
        envelopes={filteredEnvelopes}
        totalAssigned={budgetData.totalAssigned}
        totalActivity={budgetData.totalActivity}
        {totalAvailable}
        totalCount={budgetData.envelopes.length}
        {currency}
        onAssignBudget={assignBudget}
      />
    </Card>
  </div>
</PageLayout>
