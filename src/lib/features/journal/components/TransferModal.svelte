<script lang="ts">
  import { listAccountsCmd, type Account } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { journalState } from '$lib/features/journal/state/journalDraft.svelte';
  import {
    parseStringAmountToMinor,
    formatMinorGrouping,
  } from '$lib/core/format/currency';
  import { buildSimpleSplitsWithFee } from '$lib/features/journal/state/journalFormUtils';
  import { ModalShell, Button, SelectDropdown } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import type { Currency } from '$lib/core/types';

  let {
    open = $bindable(false),
    initialFrom = '',
    initialTo = '',
    onSuccess,
  }: {
    open: boolean;
    initialFrom?: string;
    initialTo?: string;
    onSuccess?: () => void;
  } = $props();

  let accounts = $state<Account[]>([]);
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));

  // svelte-ignore state_referenced_locally
  let transferFrom = $state(initialFrom);
  // svelte-ignore state_referenced_locally
  let transferTo = $state(initialTo);
  let transferAmount = $state('');
  let transferDate = $state(new Date().toISOString().split('T')[0]);
  let transferNote = $state('');
  let transferError = $state('');
  let transferBusy = $state(false);
  let showFee = $state(false);
  let feeAmount = $state('');
  let feeAccount = $state('');

  $effect(() => {
    if (open) {
      transferFrom = initialFrom;
      transferTo = initialTo;
      transferError = '';
      showFee = false;
      feeAmount = '';
      feeAccount = '';
      if (!transferDate) transferDate = new Date().toISOString().split('T')[0];
      listAccountsCmd()
        .then((items) => {
          accounts = items.map((i) => i.account);
        })
        .catch(() => {
          accounts = [];
        });
    }
  });

  const transferAssetAccounts = $derived(
    accounts.filter(
      (a) => !a.placeholder && (a.account_type === 'ASSET' || a.account_type === 'LIABILITY')
    )
  );

  const feeExpenseAccounts = $derived(accounts.filter((a) => !a.placeholder && a.account_type === 'EXPENSE'));

  async function executeTransfer() {
    transferError = '';
    if (!transferFrom || !transferTo) {
      transferError = i18n.t.selectSourceTarget;
      return;
    }
    if (transferFrom === transferTo) {
      transferError = i18n.t.sourceTargetSame;
      return;
    }
    const fromAcc = accountsById.get(transferFrom);
    const toAcc = accountsById.get(transferTo);
    if (!fromAcc || !toAcc) return;

    const minor = parseStringAmountToMinor(transferAmount, fromAcc.currency);
    if (minor <= 0) {
      transferError = i18n.t.transferValidAmount;
      return;
    }
    if (fromAcc.currency !== toAcc.currency) {
      transferError = i18n.t.transferCurrencyMismatch
        .replace('{from}', fromAcc.name)
        .replace('{fromCur}', fromAcc.currency)
        .replace('{to}', toAcc.name)
        .replace('{toCur}', toAcc.currency);
      return;
    }

    const feeMinor = parseStringAmountToMinor(feeAmount || '', fromAcc.currency);
    if (showFee) {
      if (!feeAccount) {
        transferError = i18n.t.adminFeeAccountRequired;
        return;
      }
      if (feeMinor <= 0) {
        transferError = i18n.t.adminFeeInvalid;
        return;
      }
      const feeAcc = accountsById.get(feeAccount);
      if (!feeAcc || feeAcc.placeholder) {
        transferError = i18n.t.adminFeeAccountRequired;
        return;
      }
      if (feeAcc.currency !== fromAcc.currency) {
        transferError = i18n.t.transferCurrencyMismatch
          .replace('{from}', feeAcc.name)
          .replace('{fromCur}', feeAcc.currency)
          .replace('{to}', fromAcc.name)
          .replace('{toCur}', fromAcc.currency);
        return;
      }
      if (feeAccount === transferFrom || feeAccount === transferTo) {
        transferError = i18n.t.sourceTargetSame;
        return;
      }
    }

    transferBusy = true;
    try {
      const splits = buildSimpleSplitsWithFee(
        transferFrom,
        transferTo,
        minor,
        showFee ? feeMinor : 0,
        showFee ? feeAccount : '',
        []
      );
      await journalState.saveTransaction({
        id: crypto.randomUUID(),
        date: transferDate,
        description:
          transferNote.trim() ||
          i18n.t.transferDesc.replace('{from}', fromAcc.name).replace('{to}', toAcc.name),
        currency: fromAcc.currency as Currency,
        splits,
      });
      open = false;
      transferAmount = '';
      transferNote = '';
      feeAmount = '';
      feeAccount = '';
      showFee = false;
      transferError = '';
      onSuccess?.();
    } catch (e) {
      transferError = extractErrorMessage(e);
    } finally {
      transferBusy = false;
    }
  }
</script>

<ModalShell bind:open title={i18n.t.quickTransferTitle}>
  <div class="space-y-2">
    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.transferSource}
      </div>
      <SelectDropdown
        bind:value={transferFrom}
        searchable
        placeholder={i18n.t.transferSelectSource}
        options={transferAssetAccounts.map((a) => ({
          value: a.id,
          label: `${a.code} - ${a.name}`,
          sublabel: a.currency,
        }))}
        class="w-full"
      />
    </div>

    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.transferTarget}
      </div>
      <SelectDropdown
        bind:value={transferTo}
        searchable
        placeholder={i18n.t.transferSelectTarget}
        options={transferAssetAccounts.map((a) => ({
          value: a.id,
          label: `${a.code} - ${a.name}`,
          sublabel: a.currency,
        }))}
        class="w-full"
      />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <div class="label-xs text-text-base mb-1 block">
          {i18n.t.amount}
        </div>
        <input
          type="text"
          bind:value={transferAmount}
          placeholder={i18n.t.transferAmountExample}
          class="sharp-input font-proto text-small w-full text-right"
        />
      </div>
      <div>
        <div class="label-xs text-text-base mb-1 block">{i18n.t.date}</div>
        <input
          type="date"
          bind:value={transferDate}
          class="sharp-input font-proto text-small w-full"
        />
      </div>
    </div>

    <div>
      <div class="label-xs text-text-base mb-1 block">
        {i18n.t.transferNoteLabel}
      </div>
      <input
        type="text"
        bind:value={transferNote}
        placeholder={i18n.t.transferNotePlaceholder}
        class="sharp-input text-small w-full"
      />
    </div>

    <div class="border-line/40 mt-2 border-t pt-2">
      <label class="font-proto text-smaller inline-flex cursor-pointer items-center gap-2">
        <input type="checkbox" bind:checked={showFee} class="accent-teal" />
        <span>{i18n.t.adminFeeOpt}</span>
      </label>
      {#if showFee}
        <div class="mt-2 grid gap-2 md:grid-cols-[1fr_160px]">
          <div>
            <span class="label-xs">{i18n.t.adminFeeAccount}</span>
            <SelectDropdown
              bind:value={feeAccount}
              searchable
              placeholder={i18n.t.txSelectAccount}
              options={feeExpenseAccounts.map((a) => ({
                value: a.id,
                label: `${a.code} - ${a.name}`,
              }))}
              class="mt-1 w-full"
            />
          </div>
          <div>
            <span class="label-xs">{i18n.t.amount} ({accountsById.get(transferFrom)?.currency ?? ''})</span>
            <input
              type="text"
              bind:value={feeAmount}
              placeholder={i18n.t.txAmountExample}
              inputmode="decimal"
              class="sharp-input font-proto text-small mt-1 w-full text-right"
            />
          </div>
        </div>
      {/if}
    </div>

    {#if transferError}
      <p class="badge-err text-small px-2.5 py-1">{transferError}</p>
    {/if}

    {#if transferFrom && transferTo && transferFrom !== transferTo}
      {@const fromAcc = accountsById.get(transferFrom)}
      {@const toAcc = accountsById.get(transferTo)}
      {@const feeAcc = feeAccount ? accountsById.get(feeAccount) : null}
      {@const minorAmount = fromAcc
        ? parseStringAmountToMinor(transferAmount || '', fromAcc.currency)
        : 0}
      {#if minorAmount > 0 && fromAcc && toAcc}
        {@const minorFee =
          showFee && fromAcc ? parseStringAmountToMinor(feeAmount || '', fromAcc.currency) : 0}
        {@const cur = fromAcc.currency}
        <div
          class="border-line/60 bg-bg-card font-proto text-smaller mt-2 border px-3 py-2 tabular-nums"
        >
          <p class="text-text-muted mb-1.5 tracking-wider">{i18n.t.previewJournal}</p>
          <div class="text-income flex justify-between">
            <span>{i18n.t.debit} &nbsp; {toAcc.code} {toAcc.name}</span>
            <span>{formatMinorGrouping(minorAmount, cur)}</span>
          </div>
          {#if showFee && minorFee > 0 && feeAcc}
            <div class="text-income flex justify-between">
              <span>{i18n.t.debit} &nbsp; {feeAcc.code} {feeAcc.name}</span>
              <span>{formatMinorGrouping(minorFee, cur)}</span>
            </div>
          {/if}
          <div class="text-text-base mt-1 flex justify-between border-t border-line/40 pt-1">
            <span>&nbsp;&nbsp;{i18n.t.credit} {fromAcc.code} {fromAcc.name}</span>
            <span>
              {formatMinorGrouping(
                showFee && minorFee > 0 && feeAcc ? minorAmount + minorFee : minorAmount,
                cur
              )}
            </span>
          </div>
        </div>
      {/if}
    {/if}
  </div>

  <div class="border-line mt-3 flex justify-end gap-2 border-t pt-3">
    <Button variant="ghost" onclick={() => (open = false)} disabled={transferBusy}>
      {i18n.t.cancelBtn}
    </Button>
    <Button variant="primary" onclick={executeTransfer} disabled={transferBusy}>
      {transferBusy ? i18n.t.transferringBtn : i18n.t.transferBtn}
    </Button>
  </div>
</ModalShell>
