<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import { ModalShell, Button, Badge, EmptyState } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { todayString } from '$lib/core/format/date';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { planState } from '../state/plan.svelte';
  import type { DueRecurringPlanView } from '$lib/core/ipc/bindings';

  let {
    open = $bindable(false),
    onClose = () => {},
  }: {
    open: boolean;
    onClose?: () => void;
  } = $props();

  let selectedPlanIds = new SvelteSet<string>();
  let postDate = $state(todayString());
  let posting = $state(false);

  $effect(() => {
    if (open) {
      selectedPlanIds.clear();
      for (const p of planState.duePlans) {
        selectedPlanIds.add(p.plan_id);
      }
      postDate = todayString();
    }
  });

  function togglePlan(id: string) {
    if (selectedPlanIds.has(id)) {
      selectedPlanIds.delete(id);
    } else {
      selectedPlanIds.add(id);
    }
  }

  function selectAll() {
    for (const p of planState.duePlans) {
      selectedPlanIds.add(p.plan_id);
    }
  }

  function deselectAll() {
    selectedPlanIds.clear();
  }

  async function handlePostSelected() {
    if (selectedPlanIds.size === 0) return;
    posting = true;
    try {
      const ids = Array.from(selectedPlanIds);
      await planState.postDueBatch(ids, postDate);
      notificationState.addNotification({
        type: 'INFO',
        priority: 'low',
        title: i18n.t.recurringRunnerTitle,
        message: i18n.t.recurringBatchSuccess.replace('{count}', String(ids.length)),
      });
      open = false;
      onClose();
    } catch (e) {
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.recurringRunnerTitle,
        message: e instanceof Error ? e.message : String(e),
      });
    } finally {
      posting = false;
    }
  }

  function getBadgeProps(plan: DueRecurringPlanView): {
    tone: 'err' | 'warn' | 'neutral';
    text: string;
  } {
    if (plan.is_overdue) {
      return {
        tone: 'err',
        text: i18n.t.recurringOverdueDays.replace('{days}', String(Math.abs(plan.days_diff))),
      };
    }
    if (plan.days_diff === 0) {
      return {
        tone: 'warn',
        text: i18n.t.recurringDueToday,
      };
    }
    return {
      tone: 'neutral',
      text: i18n.t.recurringUpcomingDays.replace('{days}', String(plan.days_diff)),
    };
  }
</script>

<ModalShell bind:open title={i18n.t.recurringRunnerTitle} size="wide" tone="teal" {onClose}>
  <div class="font-proto text-smaller flex flex-col gap-3">
    <p class="text-text-dim font-aux text-small">
      {i18n.t.recurringRunnerDesc}
    </p>

    {#if planState.duePlans.length === 0}
      <div class="py-12">
        <EmptyState title={i18n.t.recurringNoDue} hint={i18n.t.planNoDueOnDate} />
      </div>
    {:else}
      <!-- Toolbar -->
      <div
        class="border-line bg-bg-card flex flex-wrap items-center justify-between gap-2 border p-2"
      >
        <div class="flex items-center gap-3">
          <span class="text-text-dim font-bold tracking-wider">
            {selectedPlanIds.size} / {planState.duePlans.length}
            {i18n.t.planMarkPosted}
          </span>
          <button type="button" class="text-teal hover:underline" onclick={selectAll}>
            {i18n.t.reconcileSelectAll}
          </button>
          <span class="text-text-muted">|</span>
          <button type="button" class="text-text-muted hover:underline" onclick={deselectAll}>
            {i18n.t.reconcileDeselectAll}
          </button>
        </div>

        <div class="flex items-center gap-2">
          <span class="text-text-dim tracking-wider uppercase">{i18n.t.entryDate}:</span>
          <input
            type="date"
            bind:value={postDate}
            class="border-line bg-bg-app text-text-base font-proto text-smaller focus:border-teal h-7 border px-2 focus:outline-none"
          />
        </div>
      </div>

      <!-- Plans Table -->
      <div class="border-line max-h-80 overflow-y-auto border">
        <table class="sharp-table w-full">
          <thead class="bg-bg-card sticky top-0 z-10">
            <tr>
              <th class="w-10 px-2 py-1.5 text-center">✓</th>
              <th class="px-3 py-1.5 text-left">{i18n.t.planTitleLabel}</th>
              <th class="px-3 py-1.5 text-left">{i18n.t.recurringNextDue}</th>
              <th class="px-3 py-1.5 text-left">{i18n.t.status}</th>
              <th class="px-3 py-1.5 text-right">{i18n.t.installmentAmount}</th>
            </tr>
          </thead>
          <tbody class="divide-line divide-y">
            {#each planState.duePlans as plan (plan.plan_id)}
              {@const badge = getBadgeProps(plan)}
              {@const isSelected = selectedPlanIds.has(plan.plan_id)}
              <tr
                class="hover:bg-bg-card/50 transition-colors {isSelected
                  ? 'bg-bg-card/30'
                  : 'opacity-60'}"
                onclick={() => togglePlan(plan.plan_id)}
              >
                <td class="px-2 py-2 text-center" onclick={(e) => e.stopPropagation()}>
                  <input
                    type="checkbox"
                    checked={isSelected}
                    class="accent-teal"
                    onchange={() => togglePlan(plan.plan_id)}
                  />
                </td>
                <td class="px-3 py-2">
                  <div class="text-text-base font-bold">{plan.title}</div>
                  <div class="text-text-muted text-micro font-aux flex items-center gap-1">
                    <span>{plan.from_account_name}</span>
                    <span>→</span>
                    <span class="text-teal">{plan.to_account_name}</span>
                  </div>
                </td>
                <td class="text-text-dim font-proto px-3 py-2">
                  {plan.next_due_date}
                </td>
                <td class="px-3 py-2">
                  <Badge size="s" tone={badge.tone}>
                    {badge.text}
                  </Badge>
                </td>
                <td class="text-text-base font-proto px-3 py-2 text-right font-bold">
                  {formatMinorToDisplay(plan.amount, plan.currency)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>

  <div class="border-line mt-3 flex shrink-0 justify-end gap-2 border-t pt-3">
    <Button
      variant="ghost"
      disabled={posting}
      onclick={() => {
        open = false;
        onClose();
      }}
    >
      {i18n.t.cancelBtn}
    </Button>

    {#if planState.duePlans.length > 0}
      <Button
        variant="primary"
        disabled={selectedPlanIds.size === 0 || posting}
        onclick={handlePostSelected}
      >
        {#if posting}
          {i18n.t.planProcessing}...
        {:else}
          {i18n.t.recurringPostSelected.replace('{count}', String(selectedPlanIds.size))}
        {/if}
      </Button>
    {/if}
  </div>
</ModalShell>
