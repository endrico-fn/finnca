<script lang="ts">
  import type { Currency } from '../state/journalDraft.svelte';
  import type { Account } from '$lib/core/ipc/bindings';
  import { parseStringAmountToMinor, formatMinorGrouping } from '$lib/core/format/currency';
  import { accountTypeLabel } from '$lib/core/format/account';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button, SelectDropdown } from '$lib/components/ui';

  let {
    standardFrom = $bindable(''),
    standardTo = $bindable(''),
    standardAmount = $bindable(''),
    adminFee = $bindable(''),
    adminFeeAccount = $bindable(''),
    currency,
    accounts,
    accountsById,
    onExpandToSplits,
  }: {
    standardFrom: string;
    standardTo: string;
    standardAmount: string;
    adminFee?: string;
    adminFeeAccount?: string;
    currency: Currency;
    accounts: Account[];
    accountsById: Map<string, Account>;
    onExpandToSplits: () => void;
  } = $props();

  let mode = $state<'EXPENSE' | 'INCOME' | 'TRANSFER' | 'OPENING_BALANCE'>('EXPENSE');
  let showFee = $state(false);

  function selectMode(m: typeof mode) {
    mode = m;
    const assets = accounts.filter((a) => a.account_type === 'ASSET');
    const expenses = accounts.filter((a) => a.account_type === 'EXPENSE');
    const incomes = accounts.filter((a) => a.account_type === 'INCOME');
    const equities = accounts.filter((a) => a.account_type === 'EQUITY');

    if (m === 'EXPENSE') {
      if (!assets.some((a) => a.id === standardFrom)) standardFrom = assets[0]?.id ?? '';
      if (!expenses.some((a) => a.id === standardTo)) standardTo = expenses[0]?.id ?? '';
    } else if (m === 'INCOME') {
      if (!incomes.some((a) => a.id === standardFrom)) standardFrom = incomes[0]?.id ?? '';
      if (!assets.some((a) => a.id === standardTo)) standardTo = assets[0]?.id ?? '';
    } else if (m === 'TRANSFER') {
      if (!assets.some((a) => a.id === standardFrom)) standardFrom = assets[0]?.id ?? '';
      const secondAsset = assets.find((a) => a.id !== standardFrom) ?? assets[1];
      if (secondAsset) standardTo = secondAsset.id;
    } else if (m === 'OPENING_BALANCE') {
      const obEquity =
        equities.find(
          (a) =>
            a.code === '3110' ||
            a.name.toLowerCase().includes('opening balance') ||
            a.name.toLowerCase().includes('saldo awal')
        ) ?? equities[0];
      if (obEquity) standardFrom = obEquity.id;
      if (!assets.some((a) => a.id === standardTo)) standardTo = assets[0]?.id ?? '';
    }
  }

  const accountOptions = $derived(
    accounts.map((a) => ({
      value: a.id,
      label: `${a.code} — ${a.name}`,
      sublabel: `[${accountTypeLabel(a.account_type)}]`,
    }))
  );

  const expenseOptions = $derived(
    accounts
      .filter((a) => a.account_type === 'EXPENSE')
      .map((a) => ({
        value: a.id,
        label: `${a.code} — ${a.name}`,
      }))
  );

  const minorAmount = $derived(parseStringAmountToMinor(standardAmount, currency));
  const minorFee = $derived(parseStringAmountToMinor(adminFee || '', currency));
  const fromAcc = $derived(accountsById.get(standardFrom));
  const toAcc = $derived(accountsById.get(standardTo));
  const feeAcc = $derived(accountsById.get(adminFeeAccount || ''));
  const formattedAmount = $derived(formatMinorGrouping(minorAmount, currency));
  const formattedFee = $derived(formatMinorGrouping(minorFee, currency));
  const formattedTotal = $derived(formatMinorGrouping(minorAmount + minorFee, currency));
</script>

<div class="border-line bg-bg-app mt-2 border p-2.5">
  <div class="mb-2.5 flex flex-wrap items-center gap-1 border-b border-line/60 pb-2">
    <Button
      variant={mode === 'EXPENSE' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => selectMode('EXPENSE')}
    >
      {i18n.t.modeExpense}
    </Button>
    <Button
      variant={mode === 'INCOME' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => selectMode('INCOME')}
    >
      {i18n.t.modeIncome}
    </Button>
    <Button
      variant={mode === 'TRANSFER' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => selectMode('TRANSFER')}
    >
      {i18n.t.modeTransfer}
    </Button>
    <Button
      variant={mode === 'OPENING_BALANCE' ? 'primary' : 'ghost'}
      size="sm"
      onclick={() => selectMode('OPENING_BALANCE')}
    >
      {i18n.t.modeOpeningBalance}
    </Button>
  </div>

  <div class="grid gap-2 md:grid-cols-[1fr_1fr_160px]">
    <div>
      <span class="label-xs">
        {mode === 'EXPENSE'
          ? i18n.t.sourceAccount
          : mode === 'INCOME'
            ? i18n.t.accTypeIncome
            : mode === 'OPENING_BALANCE'
              ? i18n.t.accTypeEquity
              : i18n.t.sourceAccount}
      </span>
      <SelectDropdown
        bind:value={standardFrom}
        searchable
        placeholder={i18n.t.txSelectAccount}
        options={accountOptions}
        class="mt-1 w-full"
        menuClass="w-full"
      />
    </div>
    <div>
      <span class="label-xs">
        {mode === 'EXPENSE'
          ? i18n.t.accTypeExpense
          : mode === 'INCOME'
            ? i18n.t.accTypeAsset
            : mode === 'OPENING_BALANCE'
              ? i18n.t.targetAccount
              : i18n.t.targetAccount}
      </span>
      <SelectDropdown
        bind:value={standardTo}
        searchable
        placeholder={i18n.t.txSelectAccount}
        options={accountOptions}
        class="mt-1 w-full"
        menuClass="w-full"
      />
    </div>
    <div>
      <span class="label-xs">{i18n.t.amount} ({currency})</span>
      <input
        bind:value={standardAmount}
        placeholder={i18n.t.txAmountExample}
        type="text"
        inputmode="decimal"
        class="sharp-input font-proto mt-1 w-full text-right"
      />
    </div>
  </div>

  {#if mode === 'TRANSFER'}
    <div class="mt-2 pt-2 border-t border-line/40">
      <label class="inline-flex items-center gap-2 cursor-pointer font-proto text-smaller">
        <input type="checkbox" bind:checked={showFee} class="accent-teal" />
        <span>{i18n.t.adminFeeOpt}</span>
      </label>
      {#if showFee}
        <div class="mt-2 grid gap-2 md:grid-cols-[1fr_160px]">
          <div>
            <span class="label-xs">{i18n.t.adminFeeAccount}</span>
            <SelectDropdown
              bind:value={adminFeeAccount}
              searchable
              placeholder={i18n.t.txSelectAccount}
              options={expenseOptions}
              class="mt-1 w-full"
              menuClass="w-full"
            />
          </div>
          <div>
            <span class="label-xs">{i18n.t.amount} ({currency})</span>
            <input
              bind:value={adminFee}
              placeholder={i18n.t.txAmountExample}
              type="text"
              inputmode="decimal"
              class="sharp-input font-proto mt-1 w-full text-right"
            />
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if standardFrom && standardTo && minorAmount > 0}
    <div
      class="border-line/60 bg-bg-card font-proto text-smaller mt-2 border px-3 py-2 tabular-nums"
    >
      <p class="text-text-muted mb-1.5 tracking-wider">{i18n.t.previewJournal}</p>
      <div class="text-income flex justify-between">
        <span>{i18n.t.debit} &nbsp; {toAcc?.code} {toAcc?.name}</span>
        <span>{formattedAmount}</span>
      </div>
      {#if minorFee > 0 && adminFeeAccount}
        <div class="text-income flex justify-between">
          <span>{i18n.t.debit} &nbsp; {feeAcc?.code} {feeAcc?.name} ({i18n.t.adminFeeOpt})</span>
          <span>{formattedFee}</span>
        </div>
      {/if}
      <div class="text-text-base flex justify-between border-t border-line/40 mt-1 pt-1">
        <span>&nbsp;&nbsp;{i18n.t.credit} {fromAcc?.code} {fromAcc?.name}</span>
        <span>{minorFee > 0 && adminFeeAccount ? formattedTotal : formattedAmount}</span>
      </div>
    </div>
  {:else if minorAmount > 0 && (!standardFrom || !standardTo)}
    <div
      class="border-line/60 bg-bg-card font-proto text-smaller mt-2 border border-dashed px-3 py-2"
    >
      <p class="text-warning">{i18n.t.selectSourceTarget}</p>
    </div>
  {/if}

  <div class="mt-2 flex justify-end">
    <Button variant="ghost" size="sm" onclick={onExpandToSplits}>
      <span class="inline-flex items-center gap-1.5">
        <Icon name="plus" size={12} />
        {i18n.t.modeSplit}
      </span>
    </Button>
  </div>
</div>
