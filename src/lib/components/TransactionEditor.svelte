<script lang="ts">
  import type { Transaction, Split, Currency } from '$lib/accounting/types';
  import { ledger } from '$lib/accounting/store.svelte';
  import {
    uid,
    toMinor,
    fromMinor,
    transactionImbalance,
    todayString,
    parseStringAmountToMinor,
  } from '$lib/accounting/finance';
  import { accountTypeLabel } from '$lib/accounting/types';
  import { i18n } from '$lib/i18n.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import { Icon, Button, SelectDropdown } from '$lib/components/ui';
  import ReceiptScanner from '$lib/components/ui/ReceiptScanner.svelte';

  let {
    tx = $bindable<Transaction | null>(null),
    onSave,
    onCancel,
    onDelete,
  }: {
    tx: Transaction | null;
    onSave: (t: Transaction) => Promise<void>;
    onCancel: () => void;
    onDelete?: (id: string) => Promise<void>;
  } = $props();

  let expanded = $state(false);

  let standardFrom = $state('');
  let standardTo = $state('');
  let standardAmount = $state('');

  let draft = $state<Transaction>({
    id: uid(),
    date: todayString(),
    dueDate: '',
    settled: false,
    description: '',
    currency: 'IDR' as Currency,
    splits: [],
  });
  let error = $state('');
  let saving = $state(false);

  let prevTxId: string | null = $state(null);
  $effect(() => {
    const currentId = tx?.id ?? null;
    if (currentId === prevTxId) return;
    prevTxId = currentId;
    if (tx) {
      draft = structuredClone(tx);
      if (tx.splits.length > 2) {
        expanded = true;
      } else {
        expanded = false;
        const debitSplit = tx.splits.find((s) => s.amount >= 0);
        const creditSplit = tx.splits.find((s) => s.amount < 0);
        if (debitSplit && creditSplit) {
          standardTo = debitSplit.accountId;
          standardFrom = creditSplit.accountId;
          standardAmount = String(fromMinor(tx.currency, Math.abs(debitSplit.amount)));
        }
      }
    } else {
      const firstAcc = ledger.accounts.find((a) => !a.placeholder)?.id ?? '';
      const secondAcc = ledger.accounts.find((a) => !a.placeholder && a.id !== firstAcc)?.id ?? '';
      draft = {
        id: uid(),
        date: todayString(),
        dueDate: '',
        settled: false,
        description: '',
        currency: 'IDR',
        splits: [
          { id: uid(), accountId: firstAcc, amount: 0, reconcile: 'n' },
          { id: uid(), accountId: secondAcc, amount: 0, reconcile: 'n' },
        ],
      };
      standardFrom = firstAcc;
      standardTo = secondAcc;
      standardAmount = '';
      expanded = false;
    }
  });

  $effect(() => {
    if (tx) return;
    if (expanded) return;
    const leaf = ledger.accounts.filter((a) => !a.placeholder);
    if (leaf.length < 2) return;
    const first = leaf[0].id;
    const second = leaf.find((a) => a.id !== (standardFrom || first))?.id ?? leaf[1].id;
    if (!standardFrom) standardFrom = first;
    if (!standardTo || standardTo === standardFrom) standardTo = second;
    if (!draft.splits[0]?.accountId || !draft.splits[1]?.accountId) {
      const a0 = standardTo || first;
      const a1 = standardFrom || second;
      if (draft.splits[0]) draft.splits[0].accountId = a0;
      if (draft.splits[1]) draft.splits[1].accountId = a1;
    }
  });

  function expandToSplits() {
    const minor = parseStringAmountToMinor(standardAmount, draft.currency);
    if (standardFrom && standardTo && minor > 0) {
      draft.splits = [
        {
          id: draft.splits[0]?.id ?? uid(),
          accountId: standardTo,
          amount: minor,
          reconcile: draft.splits[0]?.reconcile ?? 'n',
        },
        {
          id: draft.splits[1]?.id ?? uid(),
          accountId: standardFrom,
          amount: -minor,
          reconcile: draft.splits[1]?.reconcile ?? 'n',
        },
      ];
    } else if (draft.splits.length < 2) {
      const firstAcc = ledger.accounts.find((a) => !a.placeholder)?.id ?? '';
      const secondAcc = ledger.accounts.find((a) => !a.placeholder && a.id !== firstAcc)?.id ?? '';
      draft.splits = [
        { id: uid(), accountId: firstAcc, amount: 0, reconcile: 'n' },
        { id: uid(), accountId: secondAcc, amount: 0, reconcile: 'n' },
      ];
    }
    expanded = true;
  }

  function collapseToSimple() {
    if (draft.splits.length > 2) {
      error = i18n.t.txRemoveSplitsToSimple;
      return;
    }
    if (draft.splits.length === 2) {
      const debitSplit = draft.splits.find((s) => s.amount >= 0);
      const creditSplit = draft.splits.find((s) => s.amount < 0);
      if (debitSplit && creditSplit && debitSplit.amount !== 0) {
        standardTo = debitSplit.accountId;
        standardFrom = creditSplit.accountId;
        standardAmount = String(fromMinor(draft.currency, Math.abs(debitSplit.amount)));
      } else {
        if (draft.splits[0]?.accountId) standardTo = draft.splits[0].accountId;
        if (draft.splits[1]?.accountId) standardFrom = draft.splits[1].accountId;
      }
    }
    error = '';
    expanded = false;
  }

  function addSplit() {
    draft.splits = [...draft.splits, { id: uid(), accountId: '', amount: 0, reconcile: 'n' }];
  }

  function removeSplit(id: string) {
    if (draft.splits.length <= 2) {
      error = i18n.t.minTwoSplits;
      return;
    }
    draft.splits = draft.splits.filter((s) => s.id !== id);
  }

  function setAmount(split: Split, field: 'debit' | 'credit', val: string) {
    const minor = parseStringAmountToMinor(val, draft.currency);
    if (field === 'debit') split.amount = minor;
    else split.amount = -minor;
    if (draft.splits.length === 2) {
      const other = draft.splits.find((s) => s.id !== split.id);
      if (other) other.amount = -split.amount;
    }
  }

  function displayAmount(split: Split, field: 'debit' | 'credit'): string {
    if (field === 'debit')
      return split.amount > 0 ? String(fromMinor(draft.currency, split.amount)) : '';
    return split.amount < 0 ? String(fromMinor(draft.currency, -split.amount)) : '';
  }

  function autoBalanceSplits() {
    if (imbalance === 0) return;
    const diff = imbalance;
    const emptySplit = draft.splits.find((s) => s.amount === 0);
    if (emptySplit) {
      emptySplit.amount = -diff;
    } else {
      draft.splits = [
        ...draft.splits,
        {
          id: uid(),
          accountId: '',
          amount: -diff,
          reconcile: 'n',
        },
      ];
    }
  }

  const imbalance = $derived(transactionImbalance(draft));
  const isBalanced = $derived(
    imbalance === 0 && draft.splits.length >= 2 && draft.description.trim().length > 0
  );
  const debitTotal = $derived(
    draft.splits.filter((s) => s.amount > 0).reduce((a, c) => a + c.amount, 0)
  );
  const creditTotal = $derived(
    -draft.splits.filter((s) => s.amount < 0).reduce((a, c) => a + c.amount, 0)
  );

  const simpleValid = $derived(
    draft.description.trim().length > 0 &&
      !!standardFrom &&
      !!standardTo &&
      standardFrom !== standardTo &&
      parseStringAmountToMinor(standardAmount, draft.currency) > 0
  );

  let showScanner = $state(false);

  async function save() {
    error = '';
    if (!draft.description.trim()) {
      error = i18n.t.txDescRequired;
      return;
    }
    if (!expanded) {
      if (!standardFrom || !standardTo) {
        error = i18n.t.selectSourceTarget;
        return;
      }
      if (standardFrom === standardTo) {
        error = i18n.t.sourceTargetSame;
        return;
      }
      const minor = parseStringAmountToMinor(standardAmount, draft.currency);
      if (minor <= 0) {
        error = i18n.t.txAmountPositive;
        return;
      }
      draft.splits = [
        {
          id: draft.splits[0]?.id ?? uid(),
          accountId: standardTo,
          amount: minor,
          reconcile: draft.splits[0]?.reconcile ?? 'n',
        },
        {
          id: draft.splits[1]?.id ?? uid(),
          accountId: standardFrom,
          amount: -minor,
          reconcile: draft.splits[1]?.reconcile ?? 'n',
        },
      ];
    } else {
      for (const s of draft.splits) {
        if (!s.accountId) {
          error = i18n.t.allSplitsNeedAccount;
          return;
        }
      }
      if (imbalance !== 0) {
        error = i18n.t.unbalancedTx
          .replace(
            '{amount}',
            fromMinor(draft.currency, Math.abs(imbalance)).toLocaleString('en-US')
          )
          .replace('{currency}', draft.currency);
        return;
      }
    }
    saving = true;
    try {
      await onSave(draft);
    } catch (e) {
      error = String(e).replace('Error: ', '');
    } finally {
      saving = false;
    }
  }

  let confirmDeleteOpen = $state(false);
  const leafAccounts = $derived(ledger.accounts.filter((a) => !a.placeholder));

  function applyPreset(desc: string, expenseKeyword: string, assetKeyword: string = 'bca') {
    draft.description = desc;
    const to = leafAccounts.find(
      (a) => a.type === 'EXPENSE' && a.name.toLowerCase().includes(expenseKeyword.toLowerCase())
    );
    const from = leafAccounts.find(
      (a) =>
        a.type === 'ASSET' &&
        (a.name.toLowerCase().includes(assetKeyword.toLowerCase()) ||
          a.name.toLowerCase().includes('cash') ||
          a.name.toLowerCase().includes('kas'))
    );
    if (to) standardTo = to.id;
    if (from) standardFrom = from.id;
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      save();
    } else if (e.key === 'Escape') {
      onCancel();
    }
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

  {#if showScanner}
    <div class="mt-4 mb-4">
      <ReceiptScanner
        onScanComplete={(detail: { text: string; amount: number | null }) => {
          if (detail.amount) {
            standardAmount = detail.amount.toString();
            if (expanded) {
              if (draft.splits.length > 0) {
                draft.splits[0].amount = toMinor(draft.currency, detail.amount);
              }
            }
          }
          if (detail.text) {
            draft.notes =
              (draft.notes ? draft.notes + '\n\n' : '') +
              i18n.t.ocrExtractedHeader +
              '\n' +
              detail.text.substring(0, 500);
          }
        }}
      />
    </div>
  {/if}

  {#if !tx && !expanded}
    <div class="font-proto text-smaller mt-2 flex items-center gap-1.5 overflow-x-auto pb-1">
      <span class="text-text-dim mr-1 shrink-0 tracking-wider uppercase"
        >{i18n.t.quickPresets}:</span
      >
      <Button
        variant="ghost"
        size="sm"
        onclick={() => applyPreset('Makan Siang / Minum', 'makan')}
        class="shrink-0"
      >
        {i18n.t.txPresetDining}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => applyPreset('Belanja Bulanan & Supermarket', 'belanja')}
        class="shrink-0"
      >
        {i18n.t.txPresetGroceries}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => applyPreset('Transport / Bensin / Tol', 'transport')}
        class="shrink-0"
      >
        {i18n.t.txPresetTransport}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => applyPreset('Tagihan Listrik / Air / Internet', 'tagihan')}
        class="shrink-0"
      >
        {i18n.t.txPresetBills}
      </Button>
    </div>
  {/if}

  <div class="mt-2 grid gap-2 md:grid-cols-[1fr_160px]">
    <div>
      <span class="label-xs">{i18n.t.description} *</span>
      <input
        bind:value={draft.description}
        placeholder={i18n.t.txDescPlaceholder}
        class="sharp-input text-small mt-1 w-full px-2.5 py-1.5"
      />
    </div>
    <div>
      <span class="label-xs">{i18n.t.date}</span>
      <input
        type="date"
        bind:value={draft.date}
        class="sharp-input text-small mt-1 w-full px-2.5 py-1.5"
      />
    </div>
  </div>

  <div class="mt-2 grid gap-2 md:grid-cols-[1fr_150px_130px]">
    <div>
      <span class="label-xs">{i18n.t.txRefInvoice}</span>
      <input
        bind:value={draft.num}
        placeholder={i18n.t.txRefPlaceholder}
        class="sharp-input text-small mt-1 w-full px-2.5 py-1.5"
      />
    </div>
    <div>
      <span class="label-xs">{i18n.t.txDueDateOpt}</span>
      <input
        type="date"
        bind:value={draft.dueDate}
        class="sharp-input text-small mt-1 w-full px-2.5 py-1.5"
      />
    </div>
    <div>
      <span class="label-xs">{i18n.t.currency}</span>
      <select bind:value={draft.currency} class="sharp-input text-small mt-1 w-full px-2.5 py-1.5">
        <option value="IDR">{i18n.t.currencyIdrOpt}</option>
        <option value="USD">{i18n.t.currencyUsdOpt}</option>
      </select>
    </div>
  </div>

  {#if draft.dueDate}
    <div class="mt-2 flex items-center gap-2">
      <label class="text-text-base font-proto text-small flex cursor-pointer items-center gap-2">
        <input type="checkbox" bind:checked={draft.settled} class="accent-teal" />
        {i18n.t.txMarkSettled}
      </label>
    </div>
  {/if}

  {#if !expanded}
    <div class="border-line bg-bg-app mt-2 border p-2.5">
      <div class="grid gap-2 md:grid-cols-[1fr_1fr_160px]">
        <div>
          <span class="label-xs">{i18n.t.sourceAccount}</span>
          <SelectDropdown
            bind:value={standardFrom}
            searchable
            placeholder={i18n.t.txSelectAccount}
            options={leafAccounts.map((a) => ({
              value: a.id,
              label: `${a.code} — ${a.name}`,
              sublabel: `[${accountTypeLabel(a.type)}]`,
            }))}
            class="w-full"
            menuClass="w-full"
          />
        </div>
        <div>
          <span class="label-xs">{i18n.t.targetAccount}</span>
          <SelectDropdown
            bind:value={standardTo}
            searchable
            placeholder={i18n.t.txSelectAccount}
            options={leafAccounts.map((a) => ({
              value: a.id,
              label: `${a.code} — ${a.name}`,
              sublabel: `[${accountTypeLabel(a.type)}]`,
            }))}
            class="w-full"
            menuClass="w-full"
          />
        </div>
        <div>
          <span class="label-xs">{i18n.t.amount} ({draft.currency})</span>
          <input
            bind:value={standardAmount}
            placeholder={i18n.t.txAmountExample}
            type="text"
            inputmode="decimal"
            class="sharp-input mt-1 w-full px-2.5 py-1.5 text-right"
          />
        </div>
      </div>
      {#if standardFrom && standardTo && parseStringAmountToMinor(standardAmount, draft.currency) > 0}
        {@const fromAcc = ledger.accountsById.get(standardFrom)}
        {@const toAcc = ledger.accountsById.get(standardTo)}
        {@const amt = fromMinor(
          draft.currency,
          parseStringAmountToMinor(standardAmount, draft.currency)
        ).toLocaleString('en-US', { minimumFractionDigits: draft.currency === 'USD' ? 2 : 0 })}
        <div
          class="border-line/60 bg-bg-card font-proto text-smaller mt-2 border px-3 py-2 tabular-nums"
        >
          <p class="text-text-muted mb-1.5 tracking-wider">{i18n.t.previewJournal}</p>
          <div class="text-income flex justify-between">
            <span>{i18n.t.debit} &nbsp; {toAcc?.code} {toAcc?.name}</span><span>{amt}</span>
          </div>
          <div class="text-text-base flex justify-between">
            <span>&nbsp;&nbsp;{i18n.t.credit} {fromAcc?.code} {fromAcc?.name}</span><span
              >{amt}</span
            >
          </div>
        </div>
      {:else if parseStringAmountToMinor(standardAmount, draft.currency) > 0 && (!standardFrom || !standardTo)}
        <div
          class="border-line/60 bg-bg-card font-proto text-smaller mt-2 border border-dashed px-3 py-2"
        >
          <p class="text-warning">{i18n.t.selectSourceTarget}</p>
        </div>
      {/if}
      <div class="mt-2 flex justify-end">
        <Button variant="ghost" size="sm" onclick={expandToSplits}>
          <span class="inline-flex items-center gap-1.5"
            ><Icon name="plus" size={12} /> {i18n.t.txSplitTransaction}</span
          >
        </Button>
      </div>
    </div>
  {:else}
    <div class="mt-4">
      <div class="flex items-center justify-between pb-2">
        <div>
          <p class="label-xs">
            {draft.splits.length}
            {i18n.t.splitsLabel}
          </p>
          <p class="text-text-muted font-proto text-smaller mt-0.5">
            {i18n.t.debitCreditHelp}
          </p>
        </div>
        <div class="flex shrink-0 items-center gap-1.5">
          {#if draft.splits.length === 2}
            <Button
              variant="ghost"
              size="sm"
              onclick={collapseToSimple}
              class="shrink-0 whitespace-nowrap"
              ><span class="inline-flex items-center gap-1.5"
                ><Icon name="chev-left" size={12} />
                {i18n.t.txSimpleView}</span
              ></Button
            >
          {/if}
          <Button variant="ghost" size="sm" onclick={addSplit} class="shrink-0 whitespace-nowrap"
            >{i18n.t.txAddRow}</Button
          >
        </div>
      </div>

      <div class="border-line mt-2 overflow-x-auto border">
        <table class="font-proto text-small w-full tabular-nums">
          <thead class="bg-bg-app text-text-base">
            <tr>
              <th class="label-xs px-3 py-2 text-left font-normal"
                >{i18n.t.code} &amp; {i18n.t.name}</th
              >
              <th class="label-xs w-36 px-2 py-2 text-right font-normal"
                >{i18n.t.totalDebit} ({draft.currency})</th
              >
              <th class="label-xs w-36 px-2 py-2 text-right font-normal"
                >{i18n.t.totalCredit} ({draft.currency})</th
              >
              <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.note}</th>
              <th class="w-8"></th>
            </tr>
          </thead>
          <tbody class="divide-line/40 bg-bg-app divide-y">
            {#each draft.splits as sp (sp.id)}
              <tr class="hover:bg-bg-row-active">
                <td class="px-2 py-1.5">
                  <SelectDropdown
                    bind:value={sp.accountId}
                    searchable
                    placeholder={i18n.t.txSelectAccount}
                    options={leafAccounts.map((a) => ({
                      value: a.id,
                      label: `${a.code} — ${a.name}`,
                      sublabel: `[${accountTypeLabel(a.type)}]`,
                    }))}
                    class="w-full"
                    menuClass="w-max"
                  />
                </td>
                <td class="px-2 py-1.5">
                  <input
                    value={displayAmount(sp, 'debit')}
                    oninput={(e) => setAmount(sp, 'debit', (e.target as HTMLInputElement).value)}
                    placeholder={i18n.t.commonZeroPlaceholder}
                    class="sharp-input text-income w-full px-2 py-1.5 text-right"
                  />
                </td>
                <td class="px-2 py-1.5">
                  <input
                    value={displayAmount(sp, 'credit')}
                    oninput={(e) => setAmount(sp, 'credit', (e.target as HTMLInputElement).value)}
                    placeholder={i18n.t.commonZeroPlaceholder}
                    class="sharp-input text-text-base w-full px-2 py-1.5 text-right"
                  />
                </td>
                <td class="px-2 py-1.5">
                  <input
                    bind:value={sp.memo}
                    placeholder={i18n.t.txSplitMemoPlaceholder}
                    class="sharp-input w-full px-2 py-1.5"
                  />
                </td>
                <td class="px-1 py-1.5 text-center">
                  <Button
                    variant="pager"
                    size="icon"
                    onclick={() => removeSplit(sp.id)}
                    title={i18n.t.txDeleteSplit}
                    ariaLabel={i18n.t.txDeleteSplit}><Icon name="close" size={12} /></Button
                  >
                </td>
              </tr>
            {/each}
          </tbody>
          <tfoot class="border-line bg-bg-app border-t">
            <tr>
              <td class="label-xs px-3 py-2 text-right">{i18n.t.totals}</td>
              <td
                class="font-proto text-medium px-2 py-2 text-right font-bold tabular-nums {debitTotal
                  ? 'text-income'
                  : 'text-text-muted'}"
              >
                {fromMinor(draft.currency, debitTotal).toLocaleString('en-US')}
              </td>
              <td
                class="font-proto text-medium px-2 py-2 text-right font-bold tabular-nums {creditTotal
                  ? 'text-text-base'
                  : 'text-text-muted'}"
              >
                {fromMinor(draft.currency, creditTotal).toLocaleString('en-US')}
              </td>
              <td
                colspan="2"
                class="font-proto text-smaller px-3 py-2 {isBalanced
                  ? 'text-income'
                  : 'text-warning'}"
              >
                {isBalanced
                  ? i18n.t.balanced
                  : `${i18n.t.imbalance}: ${fromMinor(draft.currency, Math.abs(imbalance)).toLocaleString('en-US')}`}
                {#if !isBalanced && imbalance !== 0}
                  <Button
                    variant="tactical"
                    size="sm"
                    onclick={autoBalanceSplits}
                    class="text-smaller ml-2 inline-flex h-6 py-0.5"
                  >
                    {i18n.t.autoBalanceBtn}
                  </Button>
                {/if}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </div>
  {/if}

  <div class="mt-2">
    <span class="label-xs">{i18n.t.notesMemo}</span>
    <textarea
      bind:value={draft.notes}
      rows="2"
      class="sharp-input mt-1 w-full resize-none px-3 py-1.5"
      placeholder={i18n.t.txNotesPlaceholder}></textarea>
  </div>

  <div class="border-line mt-3 flex gap-2 border-t pt-3">
    <Button
      variant={showScanner ? 'primary' : 'tactical'}
      pressed={showScanner}
      onclick={() => (showScanner = !showScanner)}
      class="flex-1"
    >
      <span class="flex items-center justify-center gap-2">
        <Icon name="chart" size={12} />
        <span>{i18n.t.txScanBtn}</span>
      </span>
    </Button>
    <Button
      variant="primary"
      onclick={save}
      disabled={saving || (expanded ? !isBalanced : !simpleValid)}
      class="flex-1"
    >
      {saving ? i18n.t.savingBtn : tx ? i18n.t.saveTransaction : i18n.t.createTransaction}
    </Button>
    <Button variant="ghost" onclick={onCancel}>{i18n.t.cancelBtn}</Button>
    {#if tx && onDelete}
      <Button variant="danger" onclick={() => (confirmDeleteOpen = true)}
        >{i18n.t.deleteAccountBtn}</Button
      >
    {/if}
  </div>

  {#if tx && onDelete}
    <ConfirmModal
      bind:open={confirmDeleteOpen}
      title={i18n.t.confirmTitle}
      message={i18n.t.confirmDeleteTxMsg}
      confirmLabel={i18n.t.confirmBtn}
      onConfirm={async () => {
        if (tx) await onDelete(tx.id);
      }}
    />
  {/if}
</div>
