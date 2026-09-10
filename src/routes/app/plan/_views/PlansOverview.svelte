<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { formatIDR, calculatePlanProgress, determinePlanStatus } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import type { PaymentPlan } from '$lib/accounting/types';
  import { Icon, Badge, Tabs, EmptyState, Button, KpiCard, CloseButton } from '$lib/components/ui';
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

  const filterTabs = $derived([
    { id: 'ALL', label: `${i18n.t.planFilterAll} (${ledger.plans.length})` },
    {
      id: 'RECEIVABLE',
      label: `${i18n.t.planFilterReceivable} (${receivablePlansCount})`,
    },
    {
      id: 'PAYABLE',
      label: `${i18n.t.planFilterPayable} (${payablePlansCount})`,
    },
  ]);

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
    <div class="border-line bg-line/10 shrink-0 border-b px-3 py-2.5">
      <Tabs
        tabs={filterTabs}
        active={planFilter}
        onSelect={(id) => (planFilter = id)}
        variant="outline"
      />
    </div>

    <div class="flex-1 overflow-y-auto p-3">
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
                    class="font-proto text-[12px] font-semibold {p.type === 'RECEIVABLE'
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
                  <CloseButton
                    onclick={() => (confirmDeletePlan = p)}
                    label={i18n.t.deletePlan}
                  />
                </div>
              </div>

              <p class="text-text-muted font-proto mt-0.5 text-[9px] tracking-widest">
                {p.frequency} • {formatIDR(p.totalAmount)}
              </p>

              <div class="mt-1">
                <div class="font-proto mb-1 flex justify-between text-[9px]">
                  <span>{i18n.t.planPaid}: {formatIDR(prog.paidAmount)}</span>
                  <span>{prog.progressPercent}% ({prog.installmentsPaidCount}x)</span>
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

              <div class="text-text-muted font-proto flex items-center justify-between text-[10px]">
                <span>
                  {i18n.t.planTargetPer}:
                  <strong class="text-text-strong">{formatIDR(p.installmentAmount)}</strong>
                </span>
                <span class="text-text-dim">
                  {i18n.t.planLeft}: {formatIDR(prog.remainingAmount)}
                </span>
              </div>

              {#if fromAcc || toAcc}
                <div class="text-text-dim font-proto truncate text-[9px]">
                  {fromAcc?.name ?? '—'} -> {toAcc?.name ?? '—'}
                </div>
              {/if}

              {#if p.notes}
                <div class="text-text-muted truncate text-[9px] italic">
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
