<script lang="ts">
  import { onMount } from 'svelte';
  import type { Component } from 'svelte';
  import type { Transaction, Split } from '$lib/core/types';
  import { listAccountsCmd, type Account } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { toMinor, parseStringAmountToMinor } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import JournalQuickPresets from './JournalQuickPresets.svelte';
  import JournalSimpleTransfer from './JournalSimpleTransfer.svelte';
  import JournalSplitsTable from './JournalSplitsTable.svelte';
  import JournalFormFields from './JournalFormFields.svelte';
  import JournalFormActions from './JournalFormActions.svelte';
  import JournalMissingAccountsBanner from './JournalMissingAccountsBanner.svelte';
  import {
    createEmptyTransaction,
    computeImbalance,
    computeTotals,
    buildSimpleSplitsWithFee,
    extractSimpleFromSplits,
    ensureDraftFields,
    autoBalanceSplits,
    updateSplitAmount,
    resolvePresetAccounts,
    validateDraft,
  } from '../state/journalFormUtils';

  let {
    tx = null,
    initialDraft = undefined,
    onSave,
    onCancel,
    onDelete,
  }: {
    tx?: Transaction | null;
    initialDraft?: Transaction;
    onSave: (t: Transaction) => Promise<void>;
    onCancel: () => void;
    onDelete?: (id: string) => Promise<void>;
  } = $props();

  let expanded = $state(false);
  let standardFrom = $state('');
  let standardTo = $state('');
  let standardAmount = $state('');
  let adminFee = $state('');
  let adminFeeAccount = $state('');

  let draft = $state<Transaction>(createEmptyTransaction());
  let error = $state('');
  let saving = $state(false);
  let showScanner = $state(false);
  let ScannerComp = $state<Component | null>(null);
  let scannerLoading = $state(false);

  $effect(() => {
    if (showScanner && !ScannerComp && !scannerLoading) {
      scannerLoading = true;
      import('$lib/components/ui/ReceiptScanner.svelte').then(
        (m) => {
          ScannerComp = m.default;
        },
        () => {
          scannerLoading = false;
          showScanner = false;
          error = i18n.t.scannerLoadFailed;
        }
      );
    }
  });

  let accounts = $state<Account[]>([]);
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));
  const leafAccounts = $derived(accounts.filter((a) => !a.placeholder));

  function pickDefaultAccounts(leaves: Account[], preferredId?: string) {
    if (leaves.length < 2) return;

    if (preferredId) {
      const pref = leaves.find((a) => a.id === preferredId);
      if (pref) {
        if (pref.account_type === 'EXPENSE') {
          standardTo = pref.id;
          const assetAcc =
            leaves.find((a) => a.account_type === 'ASSET' && a.id !== pref.id) ?? leaves[0];
          standardFrom = assetAcc?.id ?? '';
          return;
        } else if (pref.account_type === 'INCOME') {
          standardFrom = pref.id;
          const assetAcc =
            leaves.find((a) => a.account_type === 'ASSET' && a.id !== pref.id) ?? leaves[0];
          standardTo = assetAcc?.id ?? '';
          return;
        } else if (pref.account_type === 'ASSET') {
          standardFrom = pref.id;
          const expAcc =
            leaves.find((a) => a.account_type === 'EXPENSE' && a.id !== pref.id) ?? leaves[1];
          standardTo = expAcc?.id ?? '';
          return;
        } else if (pref.account_type === 'LIABILITY') {
          standardFrom = pref.id;
          const expAcc =
            leaves.find((a) => a.account_type === 'EXPENSE' && a.id !== pref.id) ??
            leaves.find((a) => a.id !== pref.id);
          standardTo = expAcc?.id ?? '';
          return;
        } else if (pref.account_type === 'EQUITY') {
          standardFrom = pref.id;
          const assetAcc =
            leaves.find((a) => a.account_type === 'ASSET' && a.id !== pref.id) ?? leaves[0];
          standardTo = assetAcc?.id ?? '';
          return;
        } else {
          standardTo = pref.id;
          const assetAcc =
            leaves.find((a) => a.account_type === 'ASSET' && a.id !== pref.id) ?? leaves[0];
          standardFrom = assetAcc?.id ?? '';
          return;
        }
      }
    }

    if (!standardFrom) {
      const assetAcc = leaves.find((a) => a.account_type === 'ASSET') ?? leaves[0];
      standardFrom = assetAcc?.id ?? '';
    }
    if (!standardTo || standardTo === standardFrom) {
      const second =
        leaves.find((a) => a.id !== standardFrom && a.account_type === 'EXPENSE') ??
        leaves.find((a) => a.id !== standardFrom);
      if (second) standardTo = second.id;
    }
  }

  const refreshAccounts = async () => {
    try {
      const items = await listAccountsCmd();
      accounts = items.map((i) => i.account);
      const leaves = items.map((i) => i.account).filter((a) => !a.placeholder);
      if (!tx && !expanded) {
        const preferredId = initialDraft?.splits?.find((s) => s.accountId)?.accountId;
        pickDefaultAccounts(leaves, preferredId);
      }
    } catch {
      accounts = [];
    }
  };

  onMount(() => {
    refreshAccounts();
    return eventBus.on('accounts:changed', refreshAccounts);
  });

  let prevTxToken: string | symbol = Symbol();
  $effect(() => {
    const currentToken = tx ? tx.id : initialDraft ? initialDraft.id : 'NEW';
    if (currentToken === prevTxToken) return;
    prevTxToken = currentToken;
    if (tx) {
      draft = ensureDraftFields($state.snapshot(tx));
      adminFee = '';
      adminFeeAccount = '';
      if (tx.splits.length > 2) {
        expanded = true;
      } else {
        expanded = false;
        const simple = extractSimpleFromSplits(tx.splits, tx.currency);
        if (simple) {
          standardTo = simple.standardTo;
          standardFrom = simple.standardFrom;
          standardAmount = simple.standardAmount;
        }
      }
    } else {
      const snap = initialDraft ? $state.snapshot(initialDraft) : null;
      const base = ensureDraftFields(snap || createEmptyTransaction());
      if (!base.splits || base.splits.length < 2) base.splits = createEmptyTransaction().splits;
      draft = base;
      expanded = false;
      adminFee = '';
      adminFeeAccount = '';
      const simple = extractSimpleFromSplits(base.splits, base.currency);
      const preferredId = base.splits.find((s) => s.accountId)?.accountId;
      if (simple && simple.standardTo && simple.standardFrom) {
        standardTo = simple.standardTo;
        standardFrom = simple.standardFrom;
        standardAmount = simple.standardAmount;
      } else {
        standardAmount = '';
        pickDefaultAccounts(leafAccounts, preferredId);
      }
    }
  });

  const imbalance = $derived(computeImbalance(draft.splits));
  const totals = $derived(computeTotals(draft.splits));
  const isBalanced = $derived(imbalance === 0 && draft.splits.length >= 2 && !!draft.description.trim());
  const simpleValid = $derived.by(() => {
    if (!draft.description.trim() || !standardFrom || !standardTo || standardFrom === standardTo)
      return false;
    if (parseStringAmountToMinor(standardAmount, draft.currency) <= 0) return false;
    const feeMinor = parseStringAmountToMinor(adminFee || '', draft.currency);
    if (feeMinor > 0 && !adminFeeAccount) return false;
    if (adminFeeAccount && feeMinor <= 0) return false;
    if (feeMinor > 0 && (adminFeeAccount === standardFrom || adminFeeAccount === standardTo))
      return false;
    return true;
  });

  function expandToSplits() {
    const minor = parseStringAmountToMinor(standardAmount, draft.currency);
    const feeMinor = parseStringAmountToMinor(adminFee, draft.currency);
    if (standardFrom && standardTo && minor > 0) {
      draft.splits = buildSimpleSplitsWithFee(
        standardFrom,
        standardTo,
        minor,
        feeMinor,
        adminFeeAccount,
        draft.splits
      );
    } else if (draft.splits.length < 2) {
      const firstAcc = leafAccounts[0]?.id ?? '';
      const secondAcc = leafAccounts[1]?.id ?? '';
      draft.splits = [
        { id: crypto.randomUUID(), accountId: firstAcc, amount: 0, reconcile: 'n' },
        { id: crypto.randomUUID(), accountId: secondAcc, amount: 0, reconcile: 'n' },
      ];
    }
    expanded = true;
  }

  function collapseToSimple() {
    if (draft.splits.length > 2) {
      error = i18n.t.txRemoveSplitsToSimple;
      return;
    }
    const simple = extractSimpleFromSplits(draft.splits, draft.currency);
    if (simple) {
      standardTo = simple.standardTo;
      standardFrom = simple.standardFrom;
      if (simple.standardAmount) standardAmount = simple.standardAmount;
    }
    error = '';
    expanded = false;
  }

  const addSplit = () => {
    draft.splits = [...draft.splits, { id: crypto.randomUUID(), accountId: '', amount: 0, reconcile: 'n' }];
  };
  const removeSplit = (id: string) => {
    if (draft.splits.length <= 2) {
      error = i18n.t.minTwoSplits;
      return;
    }
    draft.splits = draft.splits.filter((s) => s.id !== id);
  };
  const setAmount = (split: Split, field: 'debit' | 'credit', val: string) => {
    draft.splits = updateSplitAmount(draft.splits, split.id, field, val, draft.currency);
  };
  const handleAutoBalance = () => {
    draft.splits = autoBalanceSplits(draft.splits, imbalance);
  };

  function applyPreset(preset: { desc: string; expenseKeyword: string; assetKeyword?: string }) {
    draft.description = preset.desc;
    const matched = resolvePresetAccounts(preset, leafAccounts);
    if (matched.standardTo) standardTo = matched.standardTo;
    if (matched.standardFrom) standardFrom = matched.standardFrom;
  }

  async function save() {
    error = '';
    const validation = validateDraft(
      draft,
      expanded,
      standardFrom,
      standardTo,
      standardAmount,
      adminFee,
      adminFeeAccount,
      accountsById
    );
    if (!validation.valid) {
      error = validation.error ?? '';
      return;
    }
    if (!expanded) {
      const minor = parseStringAmountToMinor(standardAmount, draft.currency);
      const feeMinor = parseStringAmountToMinor(adminFee, draft.currency);
      draft.splits = buildSimpleSplitsWithFee(
        standardFrom,
        standardTo,
        minor,
        feeMinor,
        adminFeeAccount,
        draft.splits
      );
    }
    saving = true;
    try {
      await onSave(draft);
    } catch (e) {
      error = extractErrorMessage(e);
    } finally {
      saving = false;
    }
  }

  function handleScan(detail: { text: string; amount: number | null }) {
    if (detail.amount) {
      standardAmount = detail.amount.toString();
      if (expanded && draft.splits.length > 0) draft.splits[0].amount = toMinor(draft.currency, detail.amount);
    }
    if (detail.text) {
      draft.notes = `${draft.notes ? draft.notes + '\n\n' : ''}${i18n.t.ocrExtractedHeader}\n${detail.text.substring(0, 500)}`;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      save();
    } else if (e.key === 'Escape') onCancel();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  tabindex="-1"
  role="region"
  aria-label={i18n.t.txEditorAria}
  onkeydown={handleKeydown}
  class="outline-none"
>
  {#if error}
    <div class="badge-err font-proto text-small mt-2 px-3 py-2">{error}</div>
  {/if}

  {#if leafAccounts.length < 2}
    <JournalMissingAccountsBanner {onCancel} />
  {/if}

  {#if showScanner}
    <div class="my-3">
      {#if ScannerComp}
        <ScannerComp onScanComplete={handleScan} />
      {:else}
        <p class="text-text-muted font-proto text-small px-1 py-2">{i18n.t.loadingFinnca}</p>
      {/if}
    </div>
  {/if}

  {#if !tx && !expanded}
    <JournalQuickPresets onSelect={applyPreset} />
  {/if}

  <JournalFormFields
    bind:description={draft.description}
    bind:date={draft.date}
    bind:num={draft.num}
    bind:dueDate={draft.dueDate}
    bind:currency={draft.currency}
    bind:settled={draft.settled}
    bind:notes={draft.notes}
  />

  {#if !expanded}
    <JournalSimpleTransfer
      bind:standardFrom
      bind:standardTo
      bind:standardAmount
      bind:adminFee
      bind:adminFeeAccount
      currency={draft.currency}
      accounts={leafAccounts}
      {accountsById}
      onExpandToSplits={expandToSplits}
    />
  {:else}
    <JournalSplitsTable
      bind:splits={draft.splits}
      currency={draft.currency}
      accounts={leafAccounts}
      {isBalanced}
      {imbalance}
      debitTotal={totals.debitTotal}
      creditTotal={totals.creditTotal}
      onCollapseToSimple={collapseToSimple}
      onAddSplit={addSplit}
      onRemoveSplit={removeSplit}
      onAmountChange={setAmount}
      onAutoBalance={handleAutoBalance}
    />
  {/if}

  <JournalFormActions
    {saving}
    isEdit={!!tx}
    isValid={leafAccounts.length >= 2 && (expanded ? isBalanced : simpleValid)}
    bind:showScanner
    hasDelete={!!tx && !!onDelete}
    onSave={save}
    onCancel={onCancel}
    onDelete={tx && onDelete ? () => onDelete(tx.id) : undefined}
  />
</div>
