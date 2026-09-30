<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import {
    fromMinor,
    parseStringAmountToMinor,
    formatMinorToDisplay,
  } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { Button, ModalShell, SelectDropdown, AccountSelectDropdown } from '$lib/components/ui';
  import { planState, type PlanType, type PlanFrequency } from '../state/plan.svelte';
  import { suggestInstallmentOptions } from '../planUtils';

  let {
    open = $bindable(false),
    selectedDate,
    accounts = [],
  }: {
    open: boolean;
    selectedDate: string;
    accounts?: Account[];
  } = $props();

  let planTitle = $state('');
  let planType = $state<PlanType>('RECEIVABLE');
  let planTotal = $state('');
  let planInstallment = $state('');
  let planFrequency = $state<PlanFrequency>('DAILY');
  let planDayOfMonth = $state<number>(10);
  let planAutoPost = $state(false);
  let planFromAcc = $state('');
  let planToAcc = $state('');
  let planNotes = $state('');
  let planError = $state('');
  let planBusy = $state(false);

  const accountsById = $derived(new Map<string, Account>(accounts.map((a: Account) => [a.id, a])));

  $effect(() => {
    if (open) {
      if (planState.editingPlan) {
        const ep = planState.editingPlan;
        const curr = accountsById.get(ep.fromAccountId)?.currency || 'IDR';
        planTitle = ep.title;
        planType = ep.type;
        planTotal = ep.totalAmount > 0 ? String(fromMinor(curr, ep.totalAmount)) : '';
        planInstallment =
          ep.installmentAmount > 0 ? String(fromMinor(curr, ep.installmentAmount)) : '';
        planFrequency = ep.frequency;
        planDayOfMonth = ep.dayOfMonth ?? 10;
        planAutoPost = ep.autoPost ?? false;
        planFromAcc = ep.fromAccountId;
        planToAcc = ep.toAccountId;
        planNotes = ep.notes ?? '';
        planError = '';
      } else {
        planTitle = '';
        planType = 'RECEIVABLE';
        planTotal = '';
        planInstallment = '';
        planFrequency = 'DAILY';
        planDayOfMonth = 10;
        planAutoPost = false;
        planFromAcc = accounts.find((a) => a.account_type === 'ASSET' && !a.placeholder)?.id ?? '';
        planToAcc =
          accounts.find(
            (a) =>
              (a.code.startsWith('1140') ||
                a.code.startsWith('1020') ||
                /receivable|piutang/i.test(a.name) ||
                a.account_type === 'ASSET') &&
              a.id !== planFromAcc
          )?.id ?? '';
        planNotes = '';
        planError = '';
      }
    }
  });

  const planCurrency = $derived(
    accountsById.get(planFromAcc)?.currency || accountsById.get(planToAcc)?.currency || 'IDR'
  );

  const parsedTotal = $derived(parseStringAmountToMinor(planTotal, planCurrency));
  const parsedInst = $derived(parseStringAmountToMinor(planInstallment, planCurrency));

  const installmentOptions = $derived(
    parsedTotal > 0 ? suggestInstallmentOptions(parsedTotal) : []
  );

  function selectOption(opt: { count: number; amount: number; frequency: string }) {
    planInstallment = String(fromMinor(planCurrency, opt.amount));
    planFrequency = opt.frequency as PlanFrequency;
  }

  const estimatedCount = $derived(
    parsedTotal > 0 && parsedInst > 0 ? Math.ceil(parsedTotal / parsedInst) : 0
  );

  async function savePlan() {
    planError = '';
    if (!planTitle.trim()) {
      planError = i18n.t.planTitleRequired;
      return;
    }

    if (planType === 'RECURRING') {
      if (parsedInst <= 0) {
        planError = i18n.t.planValidAmountsRequired;
        return;
      }
    } else {
      if (parsedTotal <= 0 || parsedInst <= 0) {
        planError = i18n.t.planValidAmountsRequired;
        return;
      }
    }
    if (!planFromAcc || !planToAcc) {
      planError = i18n.t.planSelectBothAccounts;
      return;
    }
    const fromAcc = accountsById.get(planFromAcc);
    const toAcc = accountsById.get(planToAcc);
    if (fromAcc && toAcc && fromAcc.currency !== toAcc.currency) {
      planError = i18n.t.planCurrencyMismatch;
      return;
    }

    const newPlanInput = {
      title: planTitle.trim(),
      plan_type: planType,
      status: 'ACTIVE' as const,
      total_amount: planType === 'RECURRING' && parsedTotal <= 0 ? 0 : parsedTotal,
      installment_amount: parsedInst,
      frequency: planFrequency,
      start_date: selectedDate,
      due_date: null,
      day_of_month: planFrequency === 'MONTHLY' ? Number(planDayOfMonth) : null,
      from_account_id: planFromAcc,
      to_account_id: planToAcc,
      notes: planNotes.trim() || null,
      auto_post: planType === 'RECURRING' ? planAutoPost : false,
    };

    planBusy = true;
    try {
      if (planState.editingPlan) {
        await planState.updatePlan(planState.editingPlan.id, {
          title: planTitle.trim(),
          plan_type: planType,
          total_amount: planType === 'RECURRING' && parsedTotal <= 0 ? 0 : parsedTotal,
          installment_amount: parsedInst,
          frequency: planFrequency,
          day_of_month: planFrequency === 'MONTHLY' ? Number(planDayOfMonth) : null,
          from_account_id: planFromAcc,
          to_account_id: planToAcc,
          notes: planNotes.trim() || null,
          auto_post: planType === 'RECURRING' ? planAutoPost : false,
        });
        notificationState.addNotification({
          type: 'LEDGER_INTEGRITY',
          priority: 'low',
          title: i18n.t.editPlanTitle,
          message: i18n.t.editPlanSuccess,
        });
      } else {
        await planState.createPlan(newPlanInput);
        notificationState.addNotification({
          type: 'LEDGER_INTEGRITY',
          priority: 'low',
          title: i18n.t.planSaved,
          message: i18n.t.planSavedMsg.replace('{title}', newPlanInput.title),
        });
      }
      open = false;
    } catch (e) {
      planError = extractErrorMessage(e);
    } finally {
      planBusy = false;
    }
  }
</script>

<ModalShell bind:open title={planState.editingPlan ? i18n.t.editPlanTitle : i18n.t.newPlanTitle}>
  <div class="space-y-2">
    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.planTitlePersonLabel}
      </div>
      <input
        bind:value={planTitle}
        placeholder={i18n.t.planTitleExampleEn}
        class="sharp-input text-small w-full"
      />
    </div>

    <div class="grid grid-cols-2 gap-2">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.type}
        </div>
        <SelectDropdown
          bind:value={planType}
          options={[
            { value: 'RECEIVABLE', label: i18n.t.planTypeReceivableOpt },
            { value: 'PAYABLE', label: i18n.t.planTypePayableOpt },
            { value: 'RECURRING', label: i18n.t.planTypeRecurringOpt },
          ]}
          class="w-full"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planFrequencyLabel}
        </div>
        <SelectDropdown
          bind:value={planFrequency}
          options={[
            { value: 'DAILY', label: i18n.t.planFreqDailyOpt },
            { value: 'WEEKLY', label: i18n.t.planFreqWeeklyOpt },
            { value: 'MONTHLY', label: i18n.t.planFreqMonthlyOpt },
          ]}
          class="w-full"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-2">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {planType === 'RECURRING'
            ? i18n.t.planTotalOptionalForRecurring
            : i18n.t.planTotalAmountLabel}
        </div>
        <input
          bind:value={planTotal}
          placeholder={i18n.t.planTotalExample}
          class="sharp-input font-proto text-small w-full text-right"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planPerInstallmentLabel}
        </div>
        <input
          bind:value={planInstallment}
          placeholder={i18n.t.planInstallmentExample}
          class="sharp-input font-proto text-small w-full text-right"
        />
      </div>
    </div>

    {#if planFrequency === 'MONTHLY'}
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planDayOfMonthLabel}
        </div>
        <input
          type="number"
          min="1"
          max="31"
          bind:value={planDayOfMonth}
          class="sharp-input font-proto text-small w-full"
        />
      </div>
    {/if}

    {#if planType === 'RECURRING'}
      <label class="border-line bg-bg-app flex cursor-pointer items-start gap-2.5 border p-2.5">
        <input type="checkbox" bind:checked={planAutoPost} class="accent-teal mt-0.5" />
        <div class="flex flex-col">
          <span class="font-proto text-small text-text-strong font-semibold">
            {i18n.t.planAutoPostLabel}
          </span>
          <span class="text-text-muted text-smaller">
            {i18n.t.planAutoPostDesc}
          </span>
        </div>
      </label>
    {/if}

    {#if installmentOptions.length > 0 && !planInstallment && planType !== 'RECURRING'}
      <div class="bg-bg-app border-line border p-2">
        <span class="label-xs text-text-dim mb-1.5 block">{i18n.t.planSuggestedOptions}</span>
        <div class="flex flex-wrap gap-1.5">
          {#each installmentOptions as opt (opt.count)}
            <button
              type="button"
              onclick={() => selectOption(opt)}
              class="sharp-btn btn-ghost font-proto text-smaller px-2 py-1"
            >
              {i18n.t.planCountShort.replace('{n}', String(opt.count))} • {formatMinorToDisplay(
                opt.amount,
                planCurrency
              )}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if estimatedCount > 0 && planType !== 'RECURRING'}
      <div class="text-teal font-proto text-smaller px-1 text-right">
        {i18n.t.planEstimatedEnd}: {i18n.t.planInstallmentsCount.replace(
          '{count}',
          String(estimatedCount)
        )}
      </div>
    {/if}

    <div class="mt-2 grid grid-cols-2 gap-2">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.sourceAccount}
        </div>
        <AccountSelectDropdown
          bind:value={planFromAcc}
          {accounts}
          {accountsById}
          placeholder={i18n.t.txSelectAccount}
          class="w-full"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.targetAccount}
        </div>
        <AccountSelectDropdown
          bind:value={planToAcc}
          {accounts}
          {accountsById}
          placeholder={i18n.t.txSelectAccount}
          class="w-full"
        />
      </div>
    </div>

    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.notesMemo}
      </div>
      <input
        bind:value={planNotes}
        placeholder={i18n.t.planNotesExampleEn}
        class="sharp-input text-small w-full"
      />
    </div>

    {#if planError}
      <p class="badge-err font-proto text-small px-2.5 py-1">{planError}</p>
    {/if}
  </div>

  <div class="border-line mt-3 flex justify-end gap-2 border-t pt-3">
    <Button variant="ghost" onclick={() => (open = false)} disabled={planBusy}>
      {i18n.t.cancelBtn}
    </Button>
    <Button variant="primary" onclick={savePlan} disabled={planBusy}>
      {planBusy ? i18n.t.planProcessing : i18n.t.saveChanges}
    </Button>
  </div>
</ModalShell>
