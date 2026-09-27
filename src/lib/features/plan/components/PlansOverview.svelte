<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { formatIDR, formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { planState, type PaymentPlan } from '../state/plan.svelte';
  import { Badge, EmptyState, KpiCard, CloseButton, ProgressBar, Icon } from '$lib/components/ui';
  import { FilterMenu, FilterSection, FilterOption } from '$lib/components/ui';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import RecurringRunnerModal from './RecurringRunnerModal.svelte';

  let {
    planFilter = $bindable('ALL'),
    accountsById = new Map<string, Account>(),
    openCreatePlan,
  } = $props<{
    planFilter: string;
    accountsById?: Map<string, Account>;
    openCreatePlan: () => void;
  }>();

  function promptDeletePlan(plan: PaymentPlan) {
    modalState.confirm({
      title: i18n.t.planConfirmDeleteTitle,
      message: i18n.t.planConfirmDeleteMsg.replace('{title}', plan.title),
      confirmLabel: i18n.t.deletePlan,
      cancelLabel: i18n.t.cancelBtn,
      danger: true,
      onConfirm: async () => {
        await planState.deletePlan(plan.id);
      },
    });
  }

  function handlePayInstallment(plan: PaymentPlan) {
    const fromAcc = accountsById.get(plan.fromAccountId);
    const toAcc = accountsById.get(plan.toAccountId);
    const curr = fromAcc?.currency || toAcc?.currency || 'IDR';
    const amount =
      plan.installmentAmount > 0
        ? plan.installmentAmount
        : (planState.getProgress(plan.id)?.remainingAmount ?? 0);

    modalState.confirm({
      title: i18n.t.recordInstallmentTitle,
      message: `${plan.title}: ${formatMinorToDisplay(amount, curr)} (${fromAcc?.name ?? '—'} → ${toAcc?.name ?? '—'})`,
      confirmLabel: i18n.t.recordInstallmentBtn,
      cancelLabel: i18n.t.cancelBtn,
      onConfirm: async () => {
        await planState.recordInstallment(plan.id, amount);
        notificationState.addNotification({
          type: 'INFO',
          priority: 'low',
          title: i18n.t.recordInstallmentTitle,
          message: i18n.t.recordInstallmentSuccess,
        });
      },
    });
  }

  const totalReceivableAmount = $derived(
    planState.plans.filter((p) => p.type === 'RECEIVABLE').reduce((a, c) => a + c.totalAmount, 0)
  );
  const totalPayableAmount = $derived(
    planState.plans.filter((p) => p.type === 'PAYABLE').reduce((a, c) => a + c.totalAmount, 0)
  );
  const receivablePlansCount = $derived(
    planState.plans.filter((p) => p.type === 'RECEIVABLE').length
  );
  const payablePlansCount = $derived(planState.plans.filter((p) => p.type === 'PAYABLE').length);
  const recurringPlansCount = $derived(
    planState.plans.filter((p) => p.type === 'RECURRING').length
  );

  const filteredPlans = $derived(
    planState.plans.filter((p) => planFilter === 'ALL' || p.type === planFilter)
  );

  const filterOptions = $derived([
    { id: 'ALL', label: i18n.t.planFilterAll, count: planState.plans.length },
    { id: 'RECEIVABLE', label: i18n.t.planFilterReceivable, count: receivablePlansCount },
    { id: 'PAYABLE', label: i18n.t.planFilterPayable, count: payablePlansCount },
    { id: 'RECURRING', label: i18n.t.planFilterRecurring, count: recurringPlansCount },
  ]);

  let planFilterOpen = $state(false);

  const planFilterLabel = $derived(
    filterOptions.find((o) => o.id === planFilter)?.label ?? i18n.t.planFilterAll
  );

  function resetPlanFilter() {
    planFilter = 'ALL';
  }

  function statusTone(status: string): 'ok' | 'warn' | 'err' {
    if (status === 'OVERDUE') return 'err';
    if (status === 'ARCHIVED') return 'warn';
    return 'ok';
  }

  function statusLabel(status: string): string {
    if (status === 'ACTIVE') return i18n.t.planStatusActive;
    if (status === 'COMPLETED') return i18n.t.planStatusCompleted;
    if (status === 'OVERDUE') return i18n.t.planStatusOverdue;
    return i18n.t.planStatusArchived;
  }

  function planFreqLabel(f: string): string {
    if (f === 'DAILY') return i18n.t.planFreqDailyOpt;
    if (f === 'WEEKLY') return i18n.t.planFreqWeeklyOpt;
    return i18n.t.planFreqMonthlyOpt;
  }
</script>

<div class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto pr-1">
  <div class="grid shrink-0 grid-cols-1 gap-2 md:grid-cols-3">
    <KpiCard
      label={i18n.t.planTotalReceivable}
      labelClass="text-income"
      value={formatIDR(totalReceivableAmount)}
      valueClass="text-income"
    />
    <KpiCard
      label={i18n.t.planTotalPayable}
      labelClass="text-expense"
      value={formatIDR(totalPayableAmount)}
      valueClass="text-expense"
    />
    <KpiCard
      label={i18n.t.recurringTotalActive}
      labelClass="text-teal"
      value={String(recurringPlansCount)}
      subValue={i18n.t.recurringRunnerTitle}
    />
  </div>

  <div class="sharp-card flex min-h-0 flex-1 flex-col overflow-hidden">
    <div
      class="border-line flex shrink-0 flex-wrap items-center justify-between gap-2 border-b px-3 py-2.5"
    >
      <FilterMenu
        bind:open={planFilterOpen}
        label={planFilterLabel}
        active={planFilter !== 'ALL'}
        count={planFilter !== 'ALL' ? 1 : 0}
        onReset={resetPlanFilter}
        resetLabel={i18n.t.reset}
        resetDisabled={planFilter === 'ALL'}
      >
        <FilterSection title={i18n.t.filterPlanTitle} layout="list">
          {#each filterOptions as opt (opt.id)}
            <FilterOption
              label={opt.label}
              count={opt.count}
              selected={planFilter === opt.id}
              check={opt.id !== 'ALL'}
              onclick={() => {
                planFilter = opt.id;
                planFilterOpen = false;
              }}
            />
          {/each}
        </FilterSection>
      </FilterMenu>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="border-line bg-teal/15 text-teal hover:bg-teal hover:text-bg-app font-proto text-smaller flex cursor-pointer items-center gap-1.5 border px-2.5 py-1 font-bold tracking-wider uppercase transition-colors"
          onclick={() => planState.openRunnerModal()}
        >
          <Icon name="refresh" size={12} />
          <span>{i18n.t.recurringRunnerTitle}</span>
          {#if planState.duePlans.length > 0}
            <span class="bg-teal text-bg-app text-micro px-1">{planState.duePlans.length}</span>
          {/if}
        </button>
        <button
          type="button"
          onclick={openCreatePlan}
          class="sharp-btn btn-primary font-proto text-smaller inline-flex cursor-pointer items-center gap-1 px-2.5 py-1 font-bold tracking-wider uppercase"
        >
          <Icon name="plus" size={12} />
          <span>{i18n.t.newPlanTracker}</span>
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-3 pt-2 pb-2.5">
      {#if filteredPlans.length === 0}
        {#if planFilter === 'ALL'}
          <EmptyState
            icon="calendar"
            title={i18n.t.planEmptySubtitle}
            onAction={openCreatePlan}
            actionLabel={i18n.t.createFirstPlan}
          />
        {:else}
          <EmptyState
            icon="calendar"
            title={i18n.t.noActivePlans}
            hint={i18n.t.planEmptySubtitle}
          />
        {/if}
      {:else}
        <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
          {#each filteredPlans as p (p.id)}
            {@const fromAcc = accountsById.get(p.fromAccountId)}
            {@const toAcc = accountsById.get(p.toAccountId)}
            {@const curr = fromAcc?.currency || toAcc?.currency || 'IDR'}
            {@const prog = planState.getProgress(p.id) ?? {
              paidAmount: 0,
              remainingAmount: p.totalAmount,
              progressPercent: 0,
              installmentsPaidCount: 0,
            }}
            {@const isRecurring = p.type === 'RECURRING'}
            {@const status = p.status ?? 'ACTIVE'}
            {@const titleColor =
              p.type === 'RECEIVABLE'
                ? 'text-income'
                : p.type === 'PAYABLE'
                  ? 'text-expense'
                  : 'text-teal'}
            <div
              class="border-line/40 hover:border-text-dim flex flex-col gap-2 border p-3 transition-colors"
            >
              <div class="flex items-start justify-between">
                <div class="flex items-center gap-2">
                  <h4 class="font-proto text-medium font-semibold {titleColor}">
                    {p.title}
                  </h4>
                  <Badge size="m" tone={statusTone(status)}>{statusLabel(status)}</Badge>
                  {#if isRecurring && p.autoPost}
                    <Badge size="s" tone="ok">{i18n.t.commonAuto}</Badge>
                  {/if}
                </div>
                <div class="flex items-center gap-1">
                  {#if !isRecurring && prog.progressPercent >= 100}
                    <Badge size="m" tone="ok">{i18n.t.planSettled}</Badge>
                  {/if}
                  <button
                    type="button"
                    onclick={() => planState.openEdit(p)}
                    class="sharp-btn btn-ghost font-proto text-text-dim hover:text-text-white text-smaller inline-flex cursor-pointer items-center p-1"
                    title={i18n.t.editPlan}
                    aria-label={i18n.t.editPlan}
                  >
                    <Icon name="pencil" size={13} />
                  </button>
                  <CloseButton onclick={() => promptDeletePlan(p)} label={i18n.t.deletePlan} />
                </div>
              </div>

              {#if isRecurring}
                <p class="text-text-muted font-proto text-smaller mt-0.5 tracking-widest">
                  {planFreqLabel(p.frequency)}
                  {#if p.dayOfMonth}
                    • {i18n.t.planDayOfMonthLabel.split(' ')[0]} {p.dayOfMonth}
                  {/if}
                  • {formatMinorToDisplay(p.installmentAmount, curr)}
                </p>

                <div class="bg-bg-app border-line/40 mt-1 flex flex-col gap-1 border p-2">
                  <div class="font-proto text-smaller flex justify-between">
                    <span class="text-text-muted">{i18n.t.recurringLastPosted}:</span>
                    <span class="text-text-strong font-semibold"
                      >{p.lastPostedDate ?? i18n.t.recurringNeverPosted}</span
                    >
                  </div>
                  <div class="font-proto text-smaller flex justify-between">
                    <span class="text-text-muted">{i18n.t.planPerInstallmentLabel}:</span>
                    <span class="text-teal font-semibold"
                      >{formatMinorToDisplay(p.installmentAmount, curr)}</span
                    >
                  </div>
                </div>
              {:else}
                <p class="text-text-muted font-proto text-smaller mt-0.5 tracking-widest">
                  {planFreqLabel(p.frequency)} • {formatMinorToDisplay(p.totalAmount, curr)}
                </p>

                <div class="mt-1">
                  <div class="font-proto text-smaller mb-1 flex justify-between">
                    <span>{i18n.t.planPaid}: {formatMinorToDisplay(prog.paidAmount, curr)}</span>
                    <span
                      >{prog.progressPercent}% ({i18n.t.planCountShort.replace(
                        '{n}',
                        String(prog.installmentsPaidCount)
                      )})</span
                    >
                  </div>
                  <ProgressBar
                    value={prog.progressPercent}
                    tone={p.type === 'RECEIVABLE' ? 'income' : 'expense'}
                    track="card"
                    size="s"
                  />
                </div>

                <div
                  class="text-text-muted font-proto text-smaller flex items-center justify-between"
                >
                  <span>
                    {i18n.t.planTargetPer}:
                    <strong class="text-text-strong"
                      >{formatMinorToDisplay(p.installmentAmount, curr)}</strong
                    >
                  </span>
                  <span class="text-text-dim">
                    {i18n.t.planLeft}: {formatMinorToDisplay(prog.remainingAmount, curr)}
                  </span>
                </div>
              {/if}

              {#if fromAcc || toAcc}
                <div class="text-text-dim font-proto text-smaller truncate">
                  {fromAcc?.name ?? '—'} → {toAcc?.name ?? '—'}
                </div>
              {/if}

              {#if p.notes}
                <div class="text-text-muted text-smaller truncate italic">
                  "{p.notes}"
                </div>
              {/if}

              {#if isRecurring || prog.remainingAmount > 0}
                <div class="border-line/40 mt-1.5 flex items-center justify-between border-t pt-2">
                  <span class="text-text-dim font-proto text-smaller">
                    {formatMinorToDisplay(p.installmentAmount, curr)} / {planFreqLabel(p.frequency)}
                  </span>
                  <button
                    type="button"
                    onclick={() => handlePayInstallment(p)}
                    class="sharp-btn btn-ghost font-proto text-teal hover:bg-teal/10 text-smaller inline-flex cursor-pointer items-center gap-1 px-2 py-0.5 font-bold tracking-wider uppercase"
                  >
                    <Icon name="check" size={11} />
                    <span
                      >{isRecurring ? i18n.t.recurringRecordNow : i18n.t.recordInstallmentBtn}</span
                    >
                  </button>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <RecurringRunnerModal bind:open={planState.runnerModalOpen} />
</div>
