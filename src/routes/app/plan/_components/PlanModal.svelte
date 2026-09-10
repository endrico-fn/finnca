<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { uid, toMinor, parseStringAmountToMinor, suggestInstallmentOptions, formatIDR } from '$lib/accounting/finance';
  import type { PaymentPlan, PlanType, PlanFrequency } from '$lib/accounting/types';
  import { i18n } from '$lib/i18n.svelte';
  import { notifStore } from '$lib/notifications/store.svelte';
  import { Button, ModalShell } from '$lib/components/ui';

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

  let prevOpen = open;
  if (open !== prevOpen) {
    prevOpen = open;
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
  }

  const parsedTotal = $derived(parseStringAmountToMinor(planTotal, 'IDR'));
  const parsedInst = $derived(parseStringAmountToMinor(planInstallment, 'IDR'));

  const installmentOptions = $derived(
    parsedTotal > 0 ? suggestInstallmentOptions(parsedTotal) : []
  );

  function selectOption(opt: { count: number; amount: number; frequency: string }) {
    planInstallment = String(opt.amount);
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
        class="sharp-input w-full px-2.5 py-1.5 text-[12px]"
      />
    </div>

    <div class="grid grid-cols-2 gap-2">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.type}
        </div>
        <select bind:value={planType} class="sharp-input bg-bg-app w-full px-2.5 py-2 text-[11px]">
          <option value="RECEIVABLE">{i18n.t.planTypeReceivableOpt}</option>
          <option value="PAYABLE">{i18n.t.planTypePayableOpt}</option>
          <option value="RECURRING">{i18n.t.planTypeRecurringOpt}</option>
        </select>
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planFrequencyLabel}
        </div>
        <select
          bind:value={planFrequency}
          class="sharp-input bg-bg-app w-full px-2.5 py-2 text-[11px]"
        >
          <option value="DAILY">{i18n.t.planFreqDailyOpt}</option>
          <option value="WEEKLY">{i18n.t.planFreqWeeklyOpt}</option>
          <option value="MONTHLY">{i18n.t.planFreqMonthlyOpt}</option>
        </select>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-2">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planTotalAmountLabel}
        </div>
        <input
          bind:value={planTotal}
          placeholder="1500000"
          class="sharp-input w-full px-2.5 py-1.5 text-[12px]"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.planPerInstallmentLabel}
        </div>
        <input
          bind:value={planInstallment}
          placeholder="50000"
          class="sharp-input w-full px-2.5 py-1.5 text-[12px]"
        />
      </div>
    </div>

    {#if installmentOptions.length > 0 && !planInstallment}
      <div class="bg-line/10 border-line border p-2">
        <span class="label-xs text-text-dim mb-1.5 block">{i18n.t.planSuggestedOptions}</span>
        <div class="flex flex-wrap gap-1.5">
          {#each installmentOptions as opt (opt.count)}
            <button
              type="button"
              onclick={() => selectOption(opt)}
              class="sharp-btn btn-ghost font-proto px-2 py-1 text-[10px]"
            >
              {opt.count}x • {formatIDR(toMinor('IDR', opt.amount))}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if estimatedCount > 0}
      <div class="text-teal font-proto px-1 text-right text-[10px]">
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
        <select
          bind:value={planFromAcc}
          class="sharp-input bg-bg-app w-full px-2.5 py-2 text-[11px]"
        >
          {#each eligibleAssetAccounts as a (a.id)}
            <option value={a.id}>{a.code} - {a.name} [{a.currency}]</option>
          {/each}
        </select>
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.targetAccount}
        </div>
        <select bind:value={planToAcc} class="sharp-input bg-bg-app w-full px-2.5 py-2 text-[11px]">
          {#each eligibleAssetAccounts as a (a.id)}
            <option value={a.id}>{a.code} - {a.name} [{a.currency}]</option>
          {/each}
        </select>
      </div>
    </div>

    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.notesMemo}
      </div>
      <input
        bind:value={planNotes}
        placeholder={i18n.t.planNotesExampleEn}
        class="sharp-input w-full px-2.5 py-1.5 text-[12px]"
      />
    </div>

    {#if planError}
      <p class="badge-err px-2.5 py-1 text-[11px]">{planError}</p>
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
