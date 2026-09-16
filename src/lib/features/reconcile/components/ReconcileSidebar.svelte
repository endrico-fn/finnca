<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { reconcileState } from '../state/reconcile.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button, Icon, SelectDropdown } from '$lib/components/ui';

  let {
    account = null,
    accountOptions = [],
    onSelectAccount,
    effectiveStartingBalance,
    effectiveClearedBalance,
    difference,
    onUploadCsv,
  }: {
    account?: Account | null;
    accountOptions: { value: string; label: string }[];
    onSelectAccount: (id: string) => void;
    effectiveStartingBalance: number;
    effectiveClearedBalance: number;
    difference: number;
    onUploadCsv: () => void;
  } = $props();

  let bankPreset = $state<'AUTO' | 'BCA' | 'MANDIRI' | 'BRI' | 'BNI'>('AUTO');
</script>

<div class="sharp-card flex w-80 shrink-0 flex-col gap-4 overflow-y-auto p-3">
  <div>
    <label
      for="reconcile-account"
      class="font-proto text-text-muted text-smaller mb-1.5 block tracking-widest uppercase"
    >
      {i18n.t.reconcileSelectAccount}
    </label>
    <SelectDropdown
      value={reconcileState.selectedAccountId}
      searchable
      onSelect={onSelectAccount}
      placeholder={i18n.t.reconcileChooseAccount}
      options={accountOptions}
      class="w-full"
    />
  </div>

  {#if reconcileState.selectedAccountId}
    <div>
      <label
        for="stmt-balance"
        class="font-proto text-text-muted text-smaller mb-1.5 block tracking-widest uppercase"
      >
        {i18n.t.reconcileStatementBalance}
      </label>
      <div class="relative">
        <span
          class="font-proto text-teal text-smaller absolute top-1/2 left-2.5 -translate-y-1/2 font-bold"
        >
          {account?.currency || 'IDR'}
        </span>
        <input
          id="stmt-balance"
          type="text"
          inputmode="decimal"
          bind:value={reconcileState.targetBalanceStr}
          class="sharp-input font-proto bg-bg-app border-line focus:border-teal text-small h-9 w-full border px-2 pl-12 font-bold"
          placeholder={i18n.t.reconcileAmountPlaceholder}
        />
      </div>
    </div>

    <!-- Telemetry Balance Status -->
    <div class="sharp-card border-line flex flex-col gap-2.5 border px-3 pt-2 pb-2.5">
      <div class="font-proto text-smaller flex items-center justify-between">
        <span class="text-text-muted uppercase">{i18n.t.reconcileStartingBalance}</span>
        <span class="text-text-strong font-proto font-medium tabular-nums">
          {formatMinorToDisplay(effectiveStartingBalance, account?.currency || 'IDR')}
        </span>
      </div>
      <div class="font-proto text-smaller flex items-center justify-between">
        <span class="text-text-muted uppercase">{i18n.t.reconcileClearedBalance}</span>
        <span class="text-teal font-proto font-bold tabular-nums">
          {formatMinorToDisplay(effectiveClearedBalance, account?.currency || 'IDR')}
        </span>
      </div>
      <div class="bg-line/60 h-px w-full"></div>
      <div class="font-proto text-small flex items-center justify-between">
        <span class="text-text-strong tracking-wider uppercase">
          {i18n.t.reconcileDifference}
        </span>
        <span
          class="font-proto font-bold tabular-nums {difference === 0
            ? 'text-income'
            : 'text-expense'}"
        >
          {formatMinorToDisplay(difference, account?.currency || 'IDR')}
        </span>
      </div>
      <div class="flex justify-end pt-1">
        <span
          class="font-proto text-smaller border px-1.5 py-0.5 {difference === 0 &&
          reconcileState.targetBalanceStr !== ''
            ? 'border-income/40 text-income bg-income/10'
            : 'border-line text-text-dim'}"
        >
          {difference === 0 && reconcileState.targetBalanceStr !== ''
            ? i18n.t.zeroDiscrepancy
            : i18n.t.outOfBalance}
        </span>
      </div>
    </div>

    {#if difference === 0 && reconcileState.targetBalanceStr !== ''}
      <div
        class="bg-income/10 border-income/30 text-income font-proto text-smaller border p-2 text-center"
      >
        {i18n.t.reconcileReadyMsg}
      </div>
    {/if}

    <div class="border-line mt-1 flex flex-col gap-2 border-t pt-3">
      <div>
        <label for="bank-preset-select" class="label-xs text-text-muted mb-1 block">
          {i18n.t.bankPresetLabel}
        </label>
        <SelectDropdown
          value={bankPreset}
          onSelect={(v) => (bankPreset = v as typeof bankPreset)}
          options={[
            { value: 'AUTO', label: i18n.t.bankPresetCustom },
            { value: 'BCA', label: i18n.t.bankPresetBca },
            { value: 'MANDIRI', label: i18n.t.bankPresetMandiri },
            { value: 'BRI', label: i18n.t.bankPresetBri },
            { value: 'BNI', label: i18n.t.bankPresetBni },
          ]}
          class="w-full"
        />
      </div>

      <Button variant="ghost" onclick={onUploadCsv} class="w-full">
        <Icon name="chart" size={12} />
        <span class="font-proto text-smaller font-semibold tracking-wider uppercase">
          {i18n.t.reconcileAutoMatch}
        </span>
      </Button>
    </div>
  {/if}
</div>
