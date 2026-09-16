<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteDate } from 'svelte/reactivity';
  import { budgetState, type BudgetMonthSummary } from '../state/budget.svelte';
  import { invokeIpc } from '$lib/core/ipc/client';
  import { PageLayout, Icon, Button } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import BudgetSummaryCards from './BudgetSummaryCards.svelte';
  import BudgetEnvelopeTable from './BudgetEnvelopeTable.svelte';

  const dateObj = new SvelteDate();

  const currentMonth = $derived(
    dateObj.getFullYear() + '-' + String(dateObj.getMonth() + 1).padStart(2, '0')
  );

  const monthName = $derived(
    dateObj
      .toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
        month: 'long',
        year: 'numeric',
      })
      .toUpperCase()
  );

  onMount(() => {
    budgetState.setMonth(currentMonth);
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

  async function assignBudget(accountId: string, amountIdrMinor: number) {
    await budgetState.assignBudget(accountId, amountIdrMinor);
  }

  async function copyPreviousMonth() {
    const prevD = new SvelteDate(dateObj);
    prevD.setMonth(prevD.getMonth() - 1);
    const prevMonthStr = `${prevD.getFullYear()}-${String(prevD.getMonth() + 1).padStart(2, '0')}`;

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

      <div class="border-line bg-bg-card flex h-7 items-center gap-1 border px-1">
        <Button
          variant="pager"
          size="icon"
          onclick={prevMonth}
          ariaLabel={i18n.t.prevMonth}
          title={i18n.t.prevMonth}
        >
          <Icon name="chev-left" size={12} />
        </Button>
        <span
          class="font-proto text-text-strong text-smaller min-w-28 text-center font-bold tracking-widest tabular-nums"
        >
          {monthName}
        </span>
        <Button
          variant="pager"
          size="icon"
          onclick={nextMonth}
          ariaLabel={i18n.t.nextMonth}
          title={i18n.t.nextMonth}
        >
          <Icon name="chev-right" size={12} />
        </Button>
      </div>
    </div>
  {/snippet}

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    <!-- Telemetry HUD Summary Strip -->
    <BudgetSummaryCards {budgetData} />

    <!-- Envelopes Table Grid -->
    <BudgetEnvelopeTable {budgetData} onAssignBudget={assignBudget} />
  </div>
</PageLayout>
