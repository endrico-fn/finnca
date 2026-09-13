<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import {
    formatIDR,
    formatMoney,
    calculatePlanProgress,
    determinePlanStatus,
  } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import type { PaymentPlan } from '$lib/accounting/types';
  import { Badge, EmptyState, KpiCard, CloseButton } from '$lib/components/ui';
  import { FilterMenu, FilterSection, FilterOption } from '$lib/components/ui';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';

  let { planFilter = $bindable('ALL'), openCreatePlan } = $props<{
    planFilter: string;
    openCreatePlan: () => void;
  }>();

  let confirmDeletePlan: PaymentPlan | null = $state(null);

  const totalReceivableAmount = $derived(
    ledger.plans.filter((p) => p.type === 'RECEIVABLE').reduce((a, c) => a + c.totalAmount, 0)
  );
  const totalPayableAmount = $derived(
    ledger.plans.filter((p) => p.type === 'PAYABLE').reduce((a, c) => a + c.totalAmount, 0)
  );
  const receivablePlansCount = $derived(ledger.plans.filter((p) => p.type === 'RECEIVABLE').length);
  const payablePlansCount = $derived(ledger.plans.filter((p) => p.type === 'PAYABLE').length);

  const filteredPlans = $derived(
    ledger.plans.filter((p) => planFilter === 'ALL' || p.type === planFilter)
  );

  const filterOptions = $derived([
    { id: 'ALL', label: i18n.t.planFilterAll, count: ledger.plans.length },
    { id: 'RECEIVABLE', label: i18n.t.planFilterReceivable, count: receivablePlansCount },
    { id: 'PAYABLE', label: i18n.t.planFilterPayable, count: payablePlansCount },
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
  <div class="grid shrink-0 grid-cols-2 gap-2">
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
  </div>

  <div class="sharp-card flex min-h-0 flex-1 flex-col overflow-hidden">
    <div class="border-line flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2.5">
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
            {@const fromAcc = ledger.accountsById.get(p.fromAccountId)}
            {@const toAcc = ledger.accountsById.get(p.toAccountId)}
            {@const curr = fromAcc?.currency || toAcc?.currency || 'IDR'}
            {@const prog = ledger.data
              ? calculatePlanProgress(p, ledger.data)
              : {
                  paidAmount: 0,
                  remainingAmount: p.totalAmount,
                  progressPercent: 0,
                  installmentsPaidCount: 0,
                }}
            {@const status = ledger.data ? determinePlanStatus(p, ledger.data) : 'ACTIVE'}
            <div
              class="border-line/40 hover:border-text-dim flex flex-col gap-2 border p-3 transition-colors"
            >
              <div class="flex items-start justify-between">
                <div class="flex items-center gap-2">
                  <h4
                    class="font-proto text-medium font-semibold {p.type === 'RECEIVABLE'
                      ? 'text-income'
                      : 'text-expense'}"
                  >
                    {p.title}
                  </h4>
                  <Badge tone={statusTone(status)}>{statusLabel(status)}</Badge>
                </div>
                <div class="flex items-center gap-1">
                  {#if prog.progressPercent >= 100}
                    <Badge tone="ok">{i18n.t.planSettled}</Badge>
                  {/if}
                  <CloseButton onclick={() => (confirmDeletePlan = p)} label={i18n.t.deletePlan} />
                </div>
              </div>

              <p class="text-text-muted font-proto text-smaller mt-0.5 tracking-widest">
                {planFreqLabel(p.frequency)} • {formatMoney(p.totalAmount, curr)}
              </p>

              <div class="mt-1">
                <div class="font-proto text-smaller mb-1 flex justify-between">
                  <span>{i18n.t.planPaid}: {formatMoney(prog.paidAmount, curr)}</span>
                  <span
                    >{prog.progressPercent}% ({i18n.t.planCountShort.replace(
                      '{n}',
                      String(prog.installmentsPaidCount)
                    )})</span
                  >
                </div>
                <div class="bg-bg-card border-line h-1.5 w-full overflow-hidden border">
                  <div
                    class="h-full transition-all {p.type === 'RECEIVABLE'
                      ? 'bg-income'
                      : 'bg-expense'}"
                    style="width: {prog.progressPercent}%"
                  ></div>
                </div>
              </div>

              <div
                class="text-text-muted font-proto text-smaller flex items-center justify-between"
              >
                <span>
                  {i18n.t.planTargetPer}:
                  <strong class="text-text-strong">{formatMoney(p.installmentAmount, curr)}</strong>
                </span>
                <span class="text-text-dim">
                  {i18n.t.planLeft}: {formatMoney(prog.remainingAmount, curr)}
                </span>
              </div>

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
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

{#if confirmDeletePlan}
  {@const plan = confirmDeletePlan}
  <ConfirmModal
    bind:open={
      () => !!confirmDeletePlan,
      (v) => {
        if (!v) confirmDeletePlan = null;
      }
    }
    title={i18n.t.planConfirmDeleteTitle}
    message={i18n.t.planConfirmDeleteMsg.replace('{title}', plan.title)}
    confirmLabel={i18n.t.deletePlan}
    onConfirm={() => {
      ledger.deletePlan(plan.id);
      confirmDeletePlan = null;
    }}
    onCancel={() => (confirmDeletePlan = null)}
  />
{/if}
