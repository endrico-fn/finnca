<script lang="ts">
  import { onMount } from 'svelte';
  import type { Component } from 'svelte';
  import type { Currency } from '$lib/core/types';
  import {
    listAccountsCmd,
    type Account,
    type CreateJournalEntryInput,
    type PostingInput,
    type JournalEntryView,
  } from '$lib/core/ipc/bindings';
  import { extractErrorMessage } from '$lib/core/ipc/errors';
  import { toMinor, fromMinor, parseStringAmountToMinor, formatMinorToDisplay } from '$lib/core/format/currency';
  import { hasMathExpression } from '$lib/core/format/mathExpression';
  import { fxState } from '$lib/core/state/fx.svelte';
  import { fetchLiveFxRate } from '$lib/features/settings/fxSync';
  import { i18n } from '$lib/core/i18n.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { closingBooksState } from '$lib/core/state/ledgerLock.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { todayString } from '$lib/core/format/date';
  import { Button, Icon, AccountSelectDropdown } from '$lib/components/ui';
  import ConfirmDialog from '$lib/components/feedback/ConfirmDialog.svelte';
  import JournalPreview from './JournalPreview.svelte';
  import JournalSplitsTable from './JournalSplitsTable.svelte';
  import TransferFeeFields from './TransferFeeFields.svelte';
  import JournalQuickPresets from './JournalQuickPresets.svelte';
  import JournalMissingAccountsBanner from './JournalMissingAccountsBanner.svelte';
  import {
    createEmptyJournalDraft,
    computeImbalance,
    computeTotals,
    buildSimpleSplitsWithFee,
    extractSimpleFromSplits,
    autoBalanceSplits,
    updateSplitAmount,
    resolvePresetAccounts,
    resolveDefaultAccounts,
    validateDraft,
    checkTransferLegs,
    type PresetInput,
  } from '../state/journalFormUtils';
  import {
    parseNoteTags,
    serializeNoteTags,
    stripSettledMarker,
  } from '../state/journalNoteTags';

  type InspectorViewMode = 'transfer' | 'journal';
  type SimpleCategory = 'TRANSFER' | 'EXPENSE' | 'INCOME' | 'CUSTOM';

  let {
    entry = null,
    initialDraft = undefined,
    initialMode = 'transfer',
    initialFrom = '',
    initialTo = '',
    onSave,
    onCancel,
    onDelete,
  }: {
    entry?: JournalEntryView | null;
    initialDraft?: CreateJournalEntryInput | null;
    initialMode?: InspectorViewMode;
    initialFrom?: string;
    initialTo?: string;
    onSave: (e: CreateJournalEntryInput) => Promise<void>;
    onCancel: () => void;
    onDelete?: (id: string) => Promise<void>;
  } = $props();

  let mode = $state<InspectorViewMode>('transfer');
  let simpleCategory = $state<SimpleCategory>('TRANSFER');

  let accounts = $state<Account[]>([]);
  let accountBalances = $state<Map<string, number>>(new Map());
  let accountsLoaded = $state(false);
  const accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));
  const leafAccounts = $derived(accounts.filter((a) => !a.placeholder));

  let standardFrom = $state('');
  let standardTo = $state('');
  let standardAmount = $state('');
  let showFee = $state(false);
  let adminFee = $state('');
  let adminFeeAccount = $state('');

  let description = $state('');
  let date = $state(todayString());
  let num = $state('');
  let dueDate = $state('');
  let settled = $state(false);
  let currency = $state<Currency>('IDR');
  let fxRate = $state<number>(fxState.rate);
  let syncingFx = $state(false);
  let notes = $state('');

  const defaultFxRate = $derived(fxState.rate);

  let postings = $state<PostingInput[]>([]);
  let error = $state('');
  let saving = $state(false);
  let confirmDeleteOpen = $state(false);

  let showScanner = $state(false);
  let ScannerComp = $state<Component | null>(null);
  let scannerLoading = $state(false);

  const isLocked = $derived(
    closingBooksState.isDateLocked(date) ||
      (entry ? closingBooksState.isDateLocked(entry.date) : false)
  );

  const minorAmount = $derived(parseStringAmountToMinor(standardAmount, currency));
  const minorFee = $derived(showFee ? parseStringAmountToMinor(adminFee || '', currency) : 0);
  const fromAcc = $derived(accountsById.get(standardFrom));
  const toAcc = $derived(accountsById.get(standardTo));
  const feeAcc = $derived(showFee && adminFeeAccount ? accountsById.get(adminFeeAccount) : undefined);
  const hasFeeLine = $derived(showFee && minorFee > 0 && !!adminFeeAccount);

  const fromBalance = $derived(accountBalances.get(standardFrom) ?? 0);
  const toBalance = $derived(accountBalances.get(standardTo) ?? 0);
  const projectedFrom = $derived(fromBalance - minorAmount);
  const projectedTo = $derived(toBalance + minorAmount);
  const isOverdraft = $derived(fromAcc?.account_type === 'ASSET' && minorAmount > 0 && projectedFrom < 0);

  const previewDebits = $derived(
    standardFrom && standardTo && minorAmount > 0 && toAcc
      ? [
          { label: `${i18n.t.debit}  ${toAcc.code} ${toAcc.name}`, amount: minorAmount },
          ...(hasFeeLine && feeAcc
            ? [
                {
                  label: `${i18n.t.debit}  ${feeAcc.code} ${feeAcc.name} (${i18n.t.adminFeeOpt})`,
                  amount: minorFee,
                },
              ]
            : []),
        ]
      : []
  );
  const previewCreditAmount = $derived(hasFeeLine ? minorAmount + minorFee : minorAmount);
  const previewCreditLabel = $derived(
    fromAcc ? `  ${i18n.t.credit} ${fromAcc.code} ${fromAcc.name}` : ''
  );

  const imbalance = $derived(computeImbalance(postings));
  const totals = $derived(computeTotals(postings));
  const isBalanced = $derived(imbalance === 0 && postings.length >= 2);

  const isCurrencyMismatch = $derived(
    fromAcc && toAcc && fromAcc.currency && toAcc.currency && fromAcc.currency !== toAcc.currency
  );

  const simpleValid = $derived.by(() => {
    if (!description.trim()) return false;
    return (
      checkTransferLegs(
        standardFrom,
        standardTo,
        minorAmount,
        adminFeeAccount,
        minorFee
      ) === null
    );
  });

  const isValid = $derived(
    leafAccounts.length >= 2 &&
      !!description.trim() &&
      (mode === 'journal' ? isBalanced : simpleValid)
  );

  function applyDefaultAccounts(leaves: Account[], preferredId?: string) {
    const resolved = resolveDefaultAccounts(leaves, preferredId, standardFrom, standardTo);
    standardFrom = resolved.standardFrom;
    standardTo = resolved.standardTo;
  }

  const refreshAccounts = async () => {
    try {
      const items = await listAccountsCmd();
      accounts = items.map((i) => i.account);
      accountBalances = new Map(items.map((i) => [i.account.id, i.direct_balance]));
      const leaves = items.map((i) => i.account).filter((a) => !a.placeholder);
      if (!entry && mode === 'transfer' && (!standardFrom || !standardTo)) {
        const preferredId = initialDraft?.postings?.find((s) => s.account_id)?.account_id;
        applyDefaultAccounts(leaves, preferredId);
      }
    } catch {
      accounts = [];
      accountBalances = new Map();
    } finally {
      accountsLoaded = true;
    }
  };

  async function handleSyncLiveFx() {
    if (syncingFx) return;
    syncingFx = true;
    try {
      const live = await fetchLiveFxRate();
      if (live && live > 1000) {
        fxRate = live;
        fxState.setRate(live);
      }
    } finally {
      syncingFx = false;
    }
  }

  onMount(() => {
    closingBooksState.load();
    refreshAccounts();
    return eventBus.on('accounts:changed', refreshAccounts);
  });

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

  let initToken: string | symbol = Symbol();
  $effect(() => {
    const currentToken = entry
      ? entry.id
      : initialDraft
        ? (initialDraft.id ?? 'DRAFT')
        : `${initialMode}_${initialFrom}_${initialTo}`;
    if (currentToken === initToken) return;
    initToken = currentToken;

    if (entry) {
      const tags = parseNoteTags(entry);
      num = tags.ref;
      settled = tags.settled;
      dueDate = tags.dueDate;
      notes = tags.cleanNotes || '';
      description = entry.description;
      date = entry.date;
      currency = (entry.currency as Currency) || 'IDR';
      fxRate = entry.fx_rate ?? defaultFxRate;
      postings = entry.postings.map((p) => ({
        id: p.id,
        account_id: p.account_id,
        amount: p.amount,
        memo: p.memo,
        reconcile: p.reconcile,
      }));

      adminFee = '';
      adminFeeAccount = '';

      if (postings.length > 2) {
        mode = 'journal';
      } else {
        mode = 'transfer';
        const simple = extractSimpleFromSplits(postings, currency);
        if (simple) {
          standardTo = simple.standardTo;
          standardFrom = simple.standardFrom;
          standardAmount = simple.standardAmount;
        }
      }
    } else if (initialDraft) {
      const snap = $state.snapshot(initialDraft);
      description = snap.description || '';
      date = snap.date || todayString();
      num = snap.reference_no || '';
      dueDate = snap.due_date || '';
      notes = snap.notes || '';
      currency = (snap.currency as Currency) || 'IDR';
      fxRate = snap.fx_rate ?? defaultFxRate;
      postings = snap.postings && snap.postings.length >= 2
        ? snap.postings
        : createEmptyJournalDraft().postings;

      adminFee = '';
      adminFeeAccount = '';

      if (postings.length > 2) {
        mode = 'journal';
      } else {
        mode = initialMode;
        const simple = extractSimpleFromSplits(postings, currency);
        if (simple && simple.standardTo && simple.standardFrom) {
          standardTo = simple.standardTo;
          standardFrom = simple.standardFrom;
          standardAmount = simple.standardAmount;
        } else {
          standardAmount = '';
          applyDefaultAccounts(leafAccounts);
        }
      }
    } else {
      mode = initialMode;
      date = todayString();
      description = '';
      num = '';
      dueDate = '';
      settled = false;
      currency = 'IDR';
      fxRate = defaultFxRate;
      notes = '';
      adminFee = '';
      adminFeeAccount = '';
      standardAmount = '';
      standardFrom = initialFrom;
      standardTo = initialTo;
      const firstAcc = initialFrom || leafAccounts[0]?.id || '';
      const secondAcc = initialTo || leafAccounts[1]?.id || '';
      postings = [
        { id: crypto.randomUUID(), account_id: firstAcc, amount: 0, reconcile: 'n' },
        { id: crypto.randomUUID(), account_id: secondAcc, amount: 0, reconcile: 'n' },
      ];
      if (!standardFrom || !standardTo) {
        applyDefaultAccounts(leafAccounts);
      }
    }
  });

  function selectSimpleCategory(cat: SimpleCategory) {
    simpleCategory = cat;
    const assets = leafAccounts.filter((a) => a.account_type === 'ASSET');
    const liabilities = leafAccounts.filter((a) => a.account_type === 'LIABILITY');
    const expenses = leafAccounts.filter((a) => a.account_type === 'EXPENSE');
    const incomes = leafAccounts.filter((a) => a.account_type === 'INCOME');

    if (cat === 'TRANSFER') {
      const candidates = [...assets, ...liabilities];
      if (!candidates.some((a) => a.id === standardFrom)) standardFrom = candidates[0]?.id ?? '';
      if (!candidates.some((a) => a.id === standardTo && a.id !== standardFrom)) {
        standardTo = candidates.find((a) => a.id !== standardFrom)?.id ?? '';
      }
    } else if (cat === 'EXPENSE') {
      if (!assets.some((a) => a.id === standardFrom)) standardFrom = assets[0]?.id ?? '';
      if (!expenses.some((a) => a.id === standardTo)) standardTo = expenses[0]?.id ?? '';
    } else if (cat === 'INCOME') {
      if (!incomes.some((a) => a.id === standardFrom)) standardFrom = incomes[0]?.id ?? '';
      if (!assets.some((a) => a.id === standardTo)) standardTo = assets[0]?.id ?? '';
    }
  }

  function switchToSplit() {
    if (standardFrom && standardTo && minorAmount > 0) {
      postings = buildSimpleSplitsWithFee(
        standardFrom,
        standardTo,
        minorAmount,
        minorFee,
        adminFeeAccount,
        postings
      );
    } else if (postings.length < 2) {
      const firstAcc = leafAccounts[0]?.id ?? '';
      const secondAcc = leafAccounts[1]?.id ?? '';
      postings = [
        { id: crypto.randomUUID(), account_id: firstAcc, amount: 0, reconcile: 'n' },
        { id: crypto.randomUUID(), account_id: secondAcc, amount: 0, reconcile: 'n' },
      ];
    }
    mode = 'journal';
  }

  function switchToSimple() {
    if (postings.length > 2) {
      error = i18n.t.txRemoveSplitsToSimple;
      return;
    }
    const simple = extractSimpleFromSplits(postings, currency);
    if (simple) {
      standardTo = simple.standardTo;
      standardFrom = simple.standardFrom;
      if (simple.standardAmount) standardAmount = simple.standardAmount;
    }
    error = '';
    mode = 'transfer';
  }

  const addSplit = () => {
    postings = [
      ...postings,
      { id: crypto.randomUUID(), account_id: '', amount: 0, reconcile: 'n' },
    ];
  };

  const removeSplit = (id?: string | null) => {
    if (!id) return;
    if (postings.length <= 2) {
      error = i18n.t.minTwoSplits;
      return;
    }
    postings = postings.filter((s) => s.id !== id);
  };

  const setSplitAmount = (split: PostingInput, field: 'debit' | 'credit', val: string) => {
    postings = updateSplitAmount(postings, split.id || '', field, val, currency);
  };

  const handleAutoBalance = () => {
    postings = autoBalanceSplits(postings, imbalance);
  };

  function applyPreset(preset: PresetInput) {
    description = preset.desc;
    if (preset.fromAccountId || preset.toAccountId) {
      if (preset.fromAccountId) standardFrom = preset.fromAccountId;
      if (preset.toAccountId) standardTo = preset.toAccountId;
    } else {
      const matched = resolvePresetAccounts(preset, leafAccounts);
      if (matched.standardTo) standardTo = matched.standardTo;
      if (matched.standardFrom) standardFrom = matched.standardFrom;
    }
    if (preset.amountMajor !== undefined) standardAmount = preset.amountMajor;
    if (preset.currency) currency = preset.currency as Currency;
  }

  async function handleSave() {
    error = '';
    if (isLocked) {
      error = i18n.t.periodLockedNotice;
      return;
    }

    const payloadDraft: CreateJournalEntryInput = {
      id: entry ? entry.id : crypto.randomUUID(),
      date,
      description: description.trim(),
      notes: serializeNoteTags(notes ?? '', settled),
      reference_no: num.trim() || null,
      due_date: stripSettledMarker(dueDate.trim()) || null,
      currency,
      fx_rate: currency === 'USD' ? fxRate : null,
      postings: [],
    };

    const isExpanded = mode === 'journal';
    const validation = validateDraft(
      payloadDraft,
      isExpanded,
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

    if (!isExpanded) {
      payloadDraft.postings = buildSimpleSplitsWithFee(
        standardFrom,
        standardTo,
        minorAmount,
        minorFee,
        adminFeeAccount,
        postings
      );
    } else {
      payloadDraft.postings = postings;
    }

    saving = true;
    try {
      await onSave(payloadDraft);
    } catch (e) {
      error = extractErrorMessage(e);
    } finally {
      saving = false;
    }
  }

  function handleScan(detail: { text: string; amount: number | null }) {
    if (detail.amount) {
      standardAmount = detail.amount.toString();
      if (mode === 'journal' && postings.length > 0) {
        postings[0].amount = toMinor(currency, detail.amount);
      }
    }
    if (detail.text) {
      notes = `${notes ? notes + '\n\n' : ''}${i18n.t.ocrExtractedHeader}\n${detail.text.substring(0, 500)}`;
    }
  }

  function handleSwapAccounts() {
    const tmp = standardFrom;
    standardFrom = standardTo;
    standardTo = tmp;
  }

  function addQuickAmount(delta: number) {
    const currentMinor = parseStringAmountToMinor(standardAmount, currency);
    const factor = currency === 'IDR' ? 1 : 100;
    const newMinor = Math.max(0, currentMinor + delta * factor);
    standardAmount = String(fromMinor(currency, newMinor));
    if (mode === 'journal' && postings.length > 0) {
      postings[0].amount = newMinor;
    }
  }

  function clearStandardAmount() {
    standardAmount = '';
    if (mode === 'journal' && postings.length > 0) {
      postings[0].amount = 0;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      if (isValid && !isLocked && !saving) {
        e.preventDefault();
        void handleSave();
      }
    } else if (e.altKey && (e.key === 'a' || e.key === 'A' || e.key === 'b' || e.key === 'B')) {
      if (mode === 'journal' && imbalance !== 0) {
        e.preventDefault();
        handleAutoBalance();
      }
    } else if (e.altKey && (e.key === 'n' || e.key === 'N')) {
      if (mode === 'journal') {
        e.preventDefault();
        addSplit();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<aside
  class="bg-bg-app flex h-full min-h-0 flex-1 flex-col overflow-hidden outline-none font-aux"
  tabindex="-1"
  aria-label={i18n.t.entryInspectorTitle}
>
  <header
    class="border-b-2 border-line bg-bg-card shrink-0 px-3 py-2 flex items-center justify-between select-none"
  >
    <div class="flex items-center gap-2.5">
      <span class="badge-neutral font-proto text-smaller font-bold">WORKSTATION</span>
      <span class="font-proto text-text-strong text-small font-bold uppercase tracking-wider">
        {entry
          ? `${i18n.t.editEntryTitle} ${num ? `· ${num}` : ''}`
          : mode === 'transfer'
            ? i18n.t.quickTransferTitle
            : i18n.t.newEntryTitle}
      </span>
      <span class="font-proto text-teal text-smaller font-bold border border-line bg-bg-app px-1.5 py-0.5">
        #{num || (entry ? entry.id.slice(0, 8).toUpperCase() : 'DRAFT')}
      </span>
      {#if isLocked}
        <span class="badge-warn font-proto text-smaller flex items-center gap-1">
          <Icon name="lock" size={10} />
          {i18n.t.lockedPeriodBadge}
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-2">
      <div class="border-line bg-bg-app flex border p-0.5">
        <button
          type="button"
          onclick={switchToSimple}
          class="font-proto text-smaller h-6 px-2.5 uppercase transition-colors {mode ===
          'transfer'
            ? 'bg-bg-btn text-teal font-bold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.modeSimpleTransfer}
        </button>
        <button
          type="button"
          onclick={switchToSplit}
          class="font-proto text-smaller h-6 px-2.5 uppercase transition-colors {mode ===
          'journal'
            ? 'bg-bg-btn text-teal font-bold'
            : 'text-text-muted hover:text-text-base'}"
        >
          {i18n.t.modeMultiSplit}
        </button>
      </div>

      <button
        type="button"
        onclick={() => modalState.toggleInspectorLayout()}
        title={modalState.inspectorLayout === 'docked' ? i18n.t.layoutCenterHud : i18n.t.layoutSplitDock}
        class="border border-line hover:border-teal hover:text-teal bg-bg-app text-text-muted font-proto text-smaller h-7 px-2 flex items-center gap-1 transition-colors"
      >
        <span>{modalState.inspectorLayout === 'docked' ? '⧉' : '◨'}</span>
        <span class="hidden sm:inline text-smaller">
          {modalState.inspectorLayout === 'docked' ? i18n.t.layoutCenterHud : i18n.t.layoutSplitDock}
        </span>
      </button>

      <button
        type="button"
        onclick={onCancel}
        class="border border-line hover:border-danger hover:text-danger bg-bg-app text-text-muted font-proto text-smaller h-7 w-7 flex items-center justify-center transition-colors"
        aria-label={i18n.t.closeBtn}
      >
        ✕
      </button>
    </div>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto p-3 space-y-3 select-text">
    {#if error}
      <div class="badge-err font-proto text-small px-3 py-2">{error}</div>
    {/if}

    {#if accountsLoaded && leafAccounts.length < 2}
      <JournalMissingAccountsBanner {onCancel} />
    {/if}

    {#if showScanner}
      <div class="border-line bg-bg-card border p-2">
        {#if ScannerComp}
          <ScannerComp onScanComplete={handleScan} />
        {:else}
          <p class="text-text-muted font-proto text-small p-2">{i18n.t.loadingFinnca}</p>
        {/if}
      </div>
    {/if}

    <div class="border-line bg-bg-card/40 border p-3">
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
        <div>
          <label for="inspector-date" class="label-xs mb-1 block font-proto font-bold">{i18n.t.date}</label>
          <input
            id="inspector-date"
            type="date"
            bind:value={date}
            class="sharp-input font-proto text-smaller h-8 w-full"
          />
        </div>

        <div>
          <label for="inspector-cur" class="label-xs mb-1 block font-proto font-bold">{i18n.t.currency}</label>
          <select
            id="inspector-cur"
            bind:value={currency}
            class="sharp-input font-proto text-smaller h-8 w-full uppercase"
          >
            <option value="IDR">{i18n.t.currencyIdrOpt}</option>
            <option value="USD">{i18n.t.currencyUsdOpt}</option>
          </select>
        </div>

        <div>
          <label for="inspector-ref" class="label-xs mb-1 block font-proto font-bold truncate">
            {i18n.t.voucherNo} / REF
          </label>
          <input
            id="inspector-ref"
            type="text"
            bind:value={num}
            placeholder={i18n.t.voucherRefPlaceholder}
            class="sharp-input font-proto text-smaller h-8 w-full"
          />
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <label for="inspector-due" class="label-xs font-proto font-bold truncate">
              {i18n.t.txDueDateOpt}
            </label>
            {#if dueDate}
              <label class="text-text-base font-proto text-smaller flex cursor-pointer items-center gap-1 select-none">
                <input type="checkbox" bind:checked={settled} class="accent-teal size-3" />
                <span class="text-smaller font-bold">{i18n.t.txMarkSettled}</span>
              </label>
            {/if}
          </div>
          <input
            id="inspector-due"
            type="date"
            bind:value={dueDate}
            class="sharp-input font-proto text-smaller h-8 w-full"
          />
        </div>
      </div>

      {#if currency === 'USD'}
        <div
          class="border border-line bg-bg-app font-proto text-smaller flex items-center justify-between p-2 mt-2.5"
        >
          <div class="flex items-center gap-2">
            <span class="badge-teal font-proto text-smaller font-bold">{i18n.t.fxRateLabel}</span>
            <span class="text-text-dim">1 USD =</span>
          </div>
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted">IDR</span>
            <input
              type="number"
              bind:value={fxRate}
              min="1"
              class="sharp-input font-proto text-smaller h-7 w-28 text-right tabular-nums"
            />
            <button
              type="button"
              onclick={handleSyncLiveFx}
              disabled={syncingFx}
              title={i18n.t.fxSyncLiveBtn}
              class="border border-line hover:border-teal hover:text-teal bg-bg-card text-text-dim px-2 h-7 font-proto text-smaller uppercase transition-colors disabled:opacity-50"
            >
              {syncingFx ? '...' : `⟳ ${i18n.t.fxSyncLiveBtn}`}
            </button>
          </div>
        </div>
      {/if}
    </div>

    {#if mode === 'transfer'}
      <div class="border-line bg-bg-card/40 border p-3 space-y-3">
        <div class="flex items-center justify-between gap-2 flex-wrap">
          <div class="flex flex-wrap items-center gap-1.5">
            <div class="flex items-center gap-1">
              <button
                type="button"
                onclick={() => selectSimpleCategory('TRANSFER')}
                class="font-proto text-smaller h-7 px-2.5 uppercase border transition-colors {simpleCategory ===
                'TRANSFER'
                  ? 'bg-bg-btn border-teal text-teal font-bold'
                  : 'border-line text-text-muted hover:text-text-base'}"
              >
                {i18n.t.modeTransfer}
              </button>
              <button
                type="button"
                onclick={() => selectSimpleCategory('EXPENSE')}
                class="font-proto text-smaller h-7 px-2.5 uppercase border transition-colors {simpleCategory ===
                'EXPENSE'
                  ? 'bg-bg-btn border-teal text-teal font-bold'
                  : 'border-line text-text-muted hover:text-text-base'}"
              >
                {i18n.t.modeExpense}
              </button>
              <button
                type="button"
                onclick={() => selectSimpleCategory('INCOME')}
                class="font-proto text-smaller h-7 px-2.5 uppercase border transition-colors {simpleCategory ===
                'INCOME'
                  ? 'bg-bg-btn border-teal text-teal font-bold'
                  : 'border-line text-text-muted hover:text-text-base'}"
              >
                {i18n.t.modeIncome}
              </button>
              <button
                type="button"
                onclick={() => (simpleCategory = 'CUSTOM')}
                class="font-proto text-smaller h-7 px-2.5 uppercase border transition-colors {simpleCategory ===
                'CUSTOM'
                  ? 'bg-bg-btn border-teal text-teal font-bold'
                  : 'border-line text-text-muted hover:text-text-base'}"
              >
                {i18n.t.modeCustom}
              </button>
            </div>

            {#if !entry}
              <div class="h-4 w-px bg-line mx-0.5 hidden sm:block"></div>
              <JournalQuickPresets onSelect={applyPreset} />
            {/if}
          </div>

          <button
            type="button"
            onclick={handleSwapAccounts}
            title={i18n.t.swapAccounts}
            class="border border-line hover:border-teal hover:text-teal bg-bg-app text-text-muted flex h-7 items-center gap-1 px-2.5 font-proto text-smaller transition-colors ml-auto"
          >
            <span>⇄</span>
            <span class="hidden sm:inline">{i18n.t.swapAccounts}</span>
          </button>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 items-start">
          <div class="border border-line bg-bg-app p-2 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="font-proto text-smaller font-bold text-text-dim uppercase tracking-wider">
                {simpleCategory === 'EXPENSE'
                  ? i18n.t.sourceAccount
                  : simpleCategory === 'INCOME'
                    ? i18n.t.accTypeIncome
                    : i18n.t.transferSource}
              </span>
              <span class="badge-neutral font-proto text-smaller">KREDIT</span>
            </div>
            <AccountSelectDropdown
              bind:value={standardFrom}
              {accounts}
              {accountsById}
              filterType={simpleCategory === 'EXPENSE'
                ? 'ASSET'
                : simpleCategory === 'INCOME'
                  ? 'INCOME'
                  : simpleCategory === 'TRANSFER'
                    ? ['ASSET', 'LIABILITY']
                    : undefined}
              placeholder={i18n.t.transferSelectSource}
              class="w-full"
            />
            {#if standardFrom}
              <div class="flex items-center justify-between font-proto text-smaller text-text-muted pt-1">
                <span>{i18n.t.currentBalance}: {formatMinorToDisplay(fromBalance, fromAcc?.currency ?? currency)}</span>
                {#if minorAmount > 0}
                  <span class="flex items-center gap-1 {isOverdraft ? 'text-expense font-bold' : 'text-text-dim'}">
                    <span>{i18n.t.projectedBalance}:</span>
                    <span class="tabular-nums">{formatMinorToDisplay(projectedFrom, fromAcc?.currency ?? currency)}</span>
                    {#if isOverdraft}
                      <span class="badge-warn font-proto text-smaller px-1 py-0">{i18n.t.overdraftWarning}</span>
                    {/if}
                  </span>
                {/if}
              </div>
            {/if}
          </div>

          <div class="border border-line bg-bg-app p-2 space-y-1.5">
            <div class="flex items-center justify-between">
              <span class="font-proto text-smaller font-bold text-text-dim uppercase tracking-wider">
                {simpleCategory === 'EXPENSE'
                  ? i18n.t.accTypeExpense
                  : simpleCategory === 'INCOME'
                    ? i18n.t.accTypeAsset
                    : i18n.t.transferTarget}
              </span>
              <span class="badge-neutral font-proto text-smaller">DEBIT</span>
            </div>
            <AccountSelectDropdown
              bind:value={standardTo}
              {accounts}
              {accountsById}
              filterType={simpleCategory === 'EXPENSE'
                ? 'EXPENSE'
                : simpleCategory === 'INCOME'
                  ? 'ASSET'
                  : simpleCategory === 'TRANSFER'
                    ? ['ASSET', 'LIABILITY']
                    : undefined}
              placeholder={i18n.t.transferSelectTarget}
              class="w-full"
            />
            {#if standardTo}
              <div class="flex items-center justify-between font-proto text-smaller text-text-muted pt-1">
                <span>{i18n.t.currentBalance}: {formatMinorToDisplay(toBalance, toAcc?.currency ?? currency)}</span>
                {#if minorAmount > 0}
                  <span class="text-text-dim flex items-center gap-1">
                    <span>{i18n.t.projectedBalance}:</span>
                    <span class="tabular-nums">{formatMinorToDisplay(projectedTo, toAcc?.currency ?? currency)}</span>
                  </span>
                {/if}
              </div>
            {/if}
          </div>
        </div>

        <TransferFeeFields
          bind:feeAccount={adminFeeAccount}
          bind:feeAmount={adminFee}
          bind:showFee
          currency={currency}
          {accounts}
          {accountsById}
        />

        {#if isCurrencyMismatch}
          <div
            class="border-warning/50 bg-warning/5 font-proto text-smaller flex items-center justify-between border p-2"
          >
            <span class="badge-warn font-proto text-smaller font-bold">{fromAcc?.currency} ≠ {toAcc?.currency}</span>
            <span class="text-text-base">
              {i18n.t.simpleCurrencyMismatch
                .replace('{from}', fromAcc?.name ?? '')
                .replace('{fromCur}', fromAcc?.currency ?? '')
                .replace('{to}', toAcc?.name ?? '')
                .replace('{toCur}', toAcc?.currency ?? '')}
            </span>
          </div>
        {/if}
      </div>

      <div class="border border-line bg-bg-app p-3 space-y-2">
        <div class="flex items-center justify-between">
          <span class="label-xs font-proto text-smaller font-bold uppercase tracking-wider">
            {i18n.t.amount} ({currency}) *
          </span>
          <span class="text-text-muted font-proto text-smaller">NOMINAL TRANSAKSI</span>
        </div>

        <div class="relative flex items-center">
          <span class="absolute left-3 font-proto text-base font-bold text-teal select-none">
            {currency === 'IDR' ? 'Rp' : '$'}
          </span>
          <input
            type="text"
            bind:value={standardAmount}
            placeholder={i18n.t.transferAmountExample}
            inputmode="decimal"
            autocomplete="off"
            class="sharp-input font-proto text-lg h-10 w-full pl-10 pr-3 text-right tabular-nums font-bold tracking-wide"
          />
        </div>

        {#if hasMathExpression(standardAmount) && minorAmount > 0}
          <div class="flex items-center justify-between font-proto text-smaller text-teal bg-teal/5 border border-teal/30 px-2 py-1">
            <span class="text-text-dim text-smaller font-bold">{i18n.t.calcResult}:</span>
            <span class="font-bold tabular-nums">
              {formatMinorToDisplay(minorAmount, currency)}
            </span>
          </div>
        {/if}

        <div class="flex flex-wrap items-center gap-1.5 pt-1">
          <span class="font-proto text-smaller text-text-dim mr-1">{i18n.t.quickAmount}:</span>
          {#if currency === 'IDR'}
            {#each [10000, 50000, 100000, 500000, 1000000] as chip (chip)}
              <button
                type="button"
                onclick={() => addQuickAmount(chip)}
                class="border-line bg-bg-card hover:bg-bg-btn hover:text-teal font-proto text-smaller h-6 px-1.5 border transition-colors"
              >
                +{chip >= 1000000 ? `${chip / 1000000}jt` : `${chip / 1000}k`}
              </button>
            {/each}
          {:else}
            {#each [5, 10, 20, 50, 100] as chip (chip)}
              <button
                type="button"
                onclick={() => addQuickAmount(chip)}
                class="border-line bg-bg-card hover:bg-bg-btn hover:text-teal font-proto text-smaller h-6 px-1.5 border transition-colors"
              >
                +${chip}
              </button>
            {/each}
          {/if}
          {#if standardAmount}
            <button
              type="button"
              onclick={clearStandardAmount}
              class="border-line text-text-muted hover:text-danger font-proto text-smaller h-6 px-1.5 border transition-colors ml-auto"
            >
              {i18n.t.clearAmount}
            </button>
          {/if}
        </div>
      </div>

      {#if previewDebits.length > 0 && previewCreditLabel}
        <JournalPreview
          {currency}
          debits={previewDebits}
          creditLabel={previewCreditLabel}
          creditAmount={previewCreditAmount}
        />
      {/if}
    {:else}
      <div class="border-line bg-bg-card/30 border p-2.5">
        <JournalSplitsTable
          bind:splits={postings}
          {currency}
          accounts={leafAccounts}
          {isBalanced}
          {imbalance}
          debitTotal={totals.debitTotal}
          creditTotal={totals.creditTotal}
          onAddSplit={addSplit}
          onRemoveSplit={removeSplit}
          onAmountChange={setSplitAmount}
          onAutoBalance={handleAutoBalance}
        />
      </div>
    {/if}

    <div class="border-line bg-bg-app border p-2.5 space-y-2">
      <div>
        <label for="inspector-desc" class="label-xs mb-1 block">
          {i18n.t.description} *
        </label>
        <input
          id="inspector-desc"
          type="text"
          bind:value={description}
          placeholder={i18n.t.txDescPlaceholder}
          class="sharp-input font-aux text-small h-8 w-full"
        />
      </div>

      <div>
        <label for="inspector-notes" class="label-xs mb-1 block">{i18n.t.notesMemo}</label>
        <textarea
          id="inspector-notes"
          bind:value={notes}
          rows="2"
          class="sharp-input font-aux text-small w-full resize-none px-3 py-1.5"
          placeholder={i18n.t.txNotesPlaceholder}
        ></textarea>
      </div>
    </div>
  </div>

  <footer class="border-line bg-bg-card/70 shrink-0 border-t p-3 flex items-center justify-between">
    <div class="flex items-center gap-1.5">
      <Button
        variant={showScanner ? 'primary' : 'tactical'}
        size="sm"
        onclick={() => (showScanner = !showScanner)}
        class="h-8 px-2.5"
      >
        <span class="flex items-center gap-1.5">
          <Icon name="chart" size={12} />
          <span class="hidden sm:inline">{i18n.t.txScanBtn}</span>
        </span>
      </Button>

      {#if entry && onDelete && !isLocked}
        <Button
          variant="danger"
          size="sm"
          onclick={() => (confirmDeleteOpen = true)}
          class="h-8 px-2.5"
        >
          {i18n.t.deleteBtn}
        </Button>
      {/if}
    </div>

    <div class="flex items-center gap-2">
      <Button variant="ghost" size="sm" onclick={onCancel} class="h-8 px-3">
        {i18n.t.cancelBtn}
      </Button>
      <Button
        variant="primary"
        size="sm"
        onclick={handleSave}
        disabled={saving || !isValid || isLocked}
        class="h-8 px-3 font-bold"
      >
        <span class="inline-flex items-center gap-1.5">
          <span>
            {saving
              ? i18n.t.savingBtn
              : entry
                ? i18n.t.saveTransaction
                : i18n.t.createTransaction}
          </span>
          <span
            class="border-line/60 bg-bg-app/40 font-proto border px-1 py-0.5 text-smaller opacity-80"
          >
            Ctrl+↵
          </span>
        </span>
      </Button>
    </div>
  </footer>

  {#if entry && onDelete && !isLocked}
    <ConfirmDialog
      bind:open={confirmDeleteOpen}
      title={i18n.t.confirmTitle}
      message={i18n.t.confirmDeleteTxMsg}
      confirmLabel={i18n.t.confirmBtn}
      onConfirm={async () => {
        if (entry) await onDelete(entry.id);
      }}
    />
  {/if}
</aside>
