<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import {
    uid,
    fromMinor,
    parseStringAmountToMinor,
    suggestInstallmentOptions,
    formatMoney,
  } from '$lib/accounting/finance';
  import type { PaymentPlan, PlanType, PlanFrequency } from '$lib/accounting/types';
  import { i18n } from '$lib/i18n.svelte';
  import { notifStore } from '$lib/notifications/store.svelte';
  import { Button, ModalShell, SelectDropdown } from '$lib/components/ui';

  let { open = $bindable(false), selectedDate } = $props<{
    open: boolean;
    selectedDate: string;
  }>();

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

  const eligibleAssetAccounts = $derived(
    ledger.accounts.filter(
      (a) =>
        !a.placeholder &&
        (a.type === 'ASSET' ||
          a.type === 'LIABILITY' ||
          a.type === 'EXPENSE' ||
          a.type === 'INCOME')
    )
  );

  $effect(() => {
    if (open) {
      planTitle = '';
      planType = 'RECEIVABLE';
      planTotal = '';
      planInstallment = '';
      planFrequency = 'DAILY';
      planDayOfMonth = 10;
      planFromAcc = ledger.accounts.find((a) => a.type === 'ASSET' && !a.placeholder)?.id ?? '';
      planToAcc =
        ledger.accounts.find(
          (a) =>
            (a.code.startsWith('1020') ||
              a.name.toLowerCase().includes('receivable') ||
              a.type === 'ASSET') &&
            a.id !== planFromAcc
        )?.id ?? '';
      planNotes = '';
      planError = '';
    }
  });

  const planCurrency = $derived(
    ledger.accountsById.get(planFromAcc)?.currency ||
      ledger.accountsById.get(planToAcc)?.currency ||
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
    const fromAcc = ledger.accountsById.get(planFromAcc);
    const toAcc = ledger.accountsById.get(planToAcc);
    if (fromAcc && toAcc && fromAcc.currency !== toAcc.currency) {
      planError = i18n.t.planCurrencyMismatch;
      return;
    }

    const newPlan: PaymentPlan = {
      id: uid(),
      title: planTitle.trim(),
      type: planType,
      status: 'ACTIVE',
      totalAmount: parsedTotal,
      installmentAmount: parsedInst,
      frequency: planFrequency,
      startDate: selectedDate,
      dayOfMonth: planFrequency === 'MONTHLY' ? planDayOfMonth : undefined,
      fromAccountId: planFromAcc,
      toAccountId: planToAcc,
      notes: planNotes.trim(),
      createdAt: new Date().toISOString(),
    };

    planBusy = true;
    try {
      await ledger.upsertPlan(newPlan);
      notifStore.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.planSaved,
        message: i18n.t.planSavedMsg.replace('{title}', newPlan.title),
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
        class="sharp-input text-small w-full px-2.5 py-1.5"
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
          class="sharp-input text-small w-full px-2.5 py-1.5"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planPerInstallmentLabel}
        </div>
        <input
          bind:value={planInstallment}
          placeholder={i18n.t.planInstallmentExample}
          class="sharp-input text-small w-full px-2.5 py-1.5"
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
              {i18n.t.planCountShort.replace('{n}', String(opt.count))} • {formatMoney(
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
          options={eligibleAssetAccounts.map((a) => ({
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
          options={eligibleAssetAccounts.map((a) => ({
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
        class="sharp-input text-small w-full px-2.5 py-1.5"
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
