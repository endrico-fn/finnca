<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import {
    fromMinor,
    parseStringAmountToMinor,
    formatMinorToDisplay,
  } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { Button, ModalShell, SelectDropdown } from '$lib/components/ui';
  import {
    planState,
    type PlanType,
    type PlanFrequency,
  } from '../state/plan.svelte';
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
  let planFromAcc = $state('');
  let planToAcc = $state('');
  let planNotes = $state('');
  let planError = $state('');
  let planBusy = $state(false);

  const accountsById = $derived(new Map<string, Account>(accounts.map((a: Account) => [a.id, a])));
  const eligibleAccounts = $derived(accounts.filter((a: Account) => !a.placeholder));

  $effect(() => {
    if (open) {
      planTitle = '';
      planType = 'RECEIVABLE';
      planTotal = '';
      planInstallment = '';
      planFrequency = 'DAILY';
      planDayOfMonth = 10;
      planFromAcc =
        accounts.find((a) => a.account_type === 'ASSET' && !a.placeholder)?.id ?? '';
      planToAcc =
        accounts.find(
          (a) =>
            (a.code.startsWith('1020') ||
              a.name.toLowerCase().includes('receivable') ||
              a.account_type === 'ASSET') &&
            a.id !== planFromAcc
        )?.id ?? '';
      planNotes = '';
      planError = '';
    }
  });

  const planCurrency = $derived(
    accountsById.get(planFromAcc)?.currency ||
      accountsById.get(planToAcc)?.currency ||
      'IDR'
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

    if (parsedTotal <= 0 || parsedInst <= 0) {
      planError = i18n.t.planValidAmountsRequired;
      return;
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
      total_amount: parsedTotal,
      installment_amount: parsedInst,
      frequency: planFrequency,
      start_date: selectedDate,
      due_date: null,
      day_of_month: planFrequency === 'MONTHLY' ? planDayOfMonth : null,
      from_account_id: planFromAcc,
      to_account_id: planToAcc,
      notes: planNotes.trim() || null,
    };

    planBusy = true;
    try {
      await planState.createPlan(newPlanInput);
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.planSaved,
        message: i18n.t.planSavedMsg.replace('{title}', newPlanInput.title),
      });
      open = false;
    } catch (e) {
      planError = String(e).replace('Error: ', '');
    } finally {
      planBusy = false;
    }
  }
</script>

<ModalShell bind:open title={i18n.t.newPlanTitle}>
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
          {i18n.t.planTotalAmountLabel}
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

    {#if installmentOptions.length > 0 && !planInstallment}
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

    {#if estimatedCount > 0}
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
        <SelectDropdown
          bind:value={planFromAcc}
          searchable
          placeholder={i18n.t.txSelectAccount}
          options={eligibleAccounts.map((a: Account) => ({
            value: a.id,
            label: `${a.code} - ${a.name} [${a.currency}]`,
          }))}
          class="w-full"
          menuClass="w-full"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.targetAccount}
        </div>
        <SelectDropdown
          bind:value={planToAcc}
          searchable
          placeholder={i18n.t.txSelectAccount}
          options={eligibleAccounts.map((a: Account) => ({
            value: a.id,
            label: `${a.code} - ${a.name} [${a.currency}]`,
          }))}
          class="w-full"
          menuClass="w-full"
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
      <p class="badge-err text-small px-2.5 py-1">{planError}</p>
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
