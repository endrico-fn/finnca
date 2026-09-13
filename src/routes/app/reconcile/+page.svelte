<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    formatMoney,
    parseStringAmountToMinor,
    isDebitNormal,
    reconciledBalanceMinor,
    clearedBalanceMinor,
  } from '$lib/accounting/finance';
  import { PageLayout, Button, Icon, EmptyState, SelectDropdown } from '$lib/components/ui';
  import { notifStore } from '$lib/notifications/store.svelte';

  let selectedAccountId = $state('');
  let targetBalanceStr = $state('');

  // Local state for cleared splits: splitId -> boolean
  let clearedMap = $state<Record<string, boolean>>({});

  function handleAccountChange() {
    clearedMap = {};
    if (selectedAccountId) {
      const accTxs = ledger.transactions.filter((t) =>
        t.splits.some((s) => s.accountId === selectedAccountId && s.reconcile === 'c')
      );
      accTxs.forEach((t) => {
        t.splits
          .filter((s) => s.accountId === selectedAccountId && s.reconcile === 'c')
          .forEach((s) => {
            clearedMap[s.id] = true;
          });
      });
    }
  }

  const account = $derived(ledger.accounts.find((a) => a.id === selectedAccountId));
  const isDebit = $derived(isDebitNormal(account?.type ?? 'ASSET'));

  const startingBalance = $derived(
    selectedAccountId && ledger.data ? reconciledBalanceMinor(selectedAccountId, ledger.data) : 0
  );

  const clearedBalance = $derived(
    selectedAccountId && ledger.data
      ? clearedBalanceMinor(selectedAccountId, ledger.data, clearedMap)
      : 0
  );

  const effectiveStartingBalance = $derived(isDebit ? startingBalance : -startingBalance);
  const effectiveClearedBalance = $derived(isDebit ? clearedBalance : -clearedBalance);

  const targetBalance = $derived.by(() => {
    if (!targetBalanceStr.trim()) return 0;
    return parseStringAmountToMinor(targetBalanceStr, account?.currency || 'IDR');
  });
  const difference = $derived(targetBalance - effectiveClearedBalance);

  const unclearedTransactions = $derived.by(() => {
    if (!selectedAccountId) return [];
    return ledger.transactions
      .filter((t) => t.splits.some((s) => s.accountId === selectedAccountId && s.reconcile !== 'y'))
      .sort((a, b) => a.date.localeCompare(b.date));
  });

  const unclearedSplitsCount = $derived(
    unclearedTransactions.reduce(
      (acc, t) =>
        acc +
        t.splits.filter((s) => s.accountId === selectedAccountId && s.reconcile !== 'y').length,
      0
    )
  );

  const clearedCount = $derived(Object.values(clearedMap).filter(Boolean).length);

  function toggleClear(splitId: string) {
    clearedMap[splitId] = !clearedMap[splitId];
  }

  function selectAllUncleared() {
    for (const tx of unclearedTransactions) {
      for (const s of tx.splits) {
        if (s.accountId === selectedAccountId && s.reconcile !== 'y') {
          clearedMap[s.id] = true;
        }
      }
    }
  }

  function clearAllSelected() {
    clearedMap = {};
  }

  async function finishReconciliation() {
    if (difference !== 0) return;

    const modifiedTxs = [];
    for (const tx of ledger.transactions) {
      let modified = false;
      for (const s of tx.splits) {
        if (s.accountId === selectedAccountId && clearedMap[s.id]) {
          s.reconcile = 'y';
          modified = true;
        }
      }
      if (modified) modifiedTxs.push(tx);
    }

    if (modifiedTxs.length > 0) {
      await Promise.all(modifiedTxs.map((tx) => ledger.upsertTransaction(tx)));
    }

    targetBalanceStr = '';
    selectedAccountId = '';
  }

  let bankPreset = $state<'AUTO' | 'BCA' | 'MANDIRI' | 'BRI' | 'BNI'>('AUTO');

  import { open } from '@tauri-apps/plugin-dialog';
  import { readTextFile } from '@tauri-apps/plugin-fs';
  import { jobManager } from '$lib/infrastructure/jobs.svelte';
  import '$lib/accounting/features/reconcileJobs'; // pastikan handler teregistrasi

  async function handleCsvUpload() {
    if (!selectedAccountId) {
      notifStore.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileTitle,
        message: i18n.t.reconcileSelectAccountWarning,
      });
      return;
    }

    const selectedPath = await open({
      multiple: false,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });

    if (!selectedPath || typeof selectedPath !== 'string') return;

    // Baca file secara lokal
    const fileContent = await readTextFile(selectedPath);

    // Alih-alih nge-block UI dan mem-parsing di sini, lempar payload ke Durable Job Queue
    jobManager.addJob('SYNC_CSV', {
      accountId: selectedAccountId,
      fileContent,
      bankPreset,
      currency: account?.currency,
    });

    notifStore.addNotification({
      type: 'LEDGER_INTEGRITY',
      priority: 'low',
      title: i18n.t.reconcileTitle,
      message: i18n.t.reconcileSyncStarted,
    });
  }
</script>

<PageLayout title={i18n.t.reconcileTitle}>
  {#snippet actions()}
    <div class="flex items-center gap-2">
      <Button
        variant={selectedAccountId && difference === 0 && targetBalanceStr !== ''
          ? 'tactical'
          : 'primary'}
        disabled={!selectedAccountId || difference !== 0 || targetBalanceStr === ''}
        onclick={finishReconciliation}
      >
        {i18n.t.finishReconciliationBtn}
      </Button>
    </div>
  {/snippet}

  <div class="flex flex-1 gap-2 overflow-hidden">
    <div class="sharp-card flex w-80 shrink-0 flex-col gap-4 overflow-y-auto p-3">
      <div>
        <label
          for="reconcile-account"
          class="font-proto text-text-muted text-smaller mb-1.5 block tracking-widest uppercase"
          >{i18n.t.reconcileSelectAccount}</label
        >
        <SelectDropdown
          bind:value={selectedAccountId}
          searchable
          onSelect={handleAccountChange}
          placeholder={i18n.t.reconcileChooseAccount}
          options={ledger.accounts
            .filter((a) => !a.placeholder && (a.type === 'ASSET' || a.type === 'LIABILITY'))
            .map((acc) => ({ value: acc.id, label: `${acc.code} - ${acc.name}` }))}
          class="w-full"
        />
      </div>

      {#if selectedAccountId}
        <div>
          <label
            for="stmt-balance"
            class="font-proto text-text-muted text-smaller mb-1.5 block tracking-widest uppercase"
            >{i18n.t.reconcileStatementBalance}</label
          >
          <div class="relative">
            <span
              class="font-proto text-teal text-smaller absolute top-1/2 left-2.5 -translate-y-1/2 font-bold"
              >{account?.currency}</span
            >
            <input
              id="stmt-balance"
              type="text"
              inputmode="decimal"
              bind:value={targetBalanceStr}
              class="sharp-input font-proto bg-bg-app border-line focus:border-teal text-small h-9 w-full border px-2 pl-12 font-bold"
              placeholder={i18n.t.reconcileAmountPlaceholder}
            />
          </div>
        </div>

        <!-- Telemetry Balance Status -->
        <div class="sharp-card border-line flex flex-col gap-2.5 border px-3 pt-2 pb-2.5">
          <div class="font-proto text-smaller flex items-center justify-between">
            <span class="text-text-muted uppercase">{i18n.t.reconcileStartingBalance}</span>
            <span class="text-text-strong font-proto font-medium tabular-nums"
              >{formatMoney(effectiveStartingBalance, account?.currency || 'IDR')}</span
            >
          </div>
          <div class="font-proto text-smaller flex items-center justify-between">
            <span class="text-text-muted uppercase">{i18n.t.reconcileClearedBalance}</span>
            <span class="text-teal font-proto font-bold tabular-nums"
              >{formatMoney(effectiveClearedBalance, account?.currency || 'IDR')}</span
            >
          </div>
          <div class="bg-line/60 h-px w-full"></div>
          <div class="font-proto text-small flex items-center justify-between">
            <span class="text-text-strong tracking-wider uppercase"
              >{i18n.t.reconcileDifference}</span
            >
            <span
              class="font-proto font-bold tabular-nums {difference === 0
                ? 'text-income'
                : 'text-expense'}"
            >
              {formatMoney(difference, account?.currency || 'IDR')}
            </span>
          </div>
          <div class="flex justify-end pt-1">
            <span
              class="font-proto text-smaller border px-1.5 py-0.5 {difference === 0 &&
              targetBalanceStr !== ''
                ? 'border-income/40 text-income bg-income/10'
                : 'border-line text-text-dim'}"
            >
              {difference === 0 && targetBalanceStr !== ''
                ? i18n.t.zeroDiscrepancy
                : i18n.t.outOfBalance}
            </span>
          </div>
        </div>

        {#if difference === 0 && targetBalanceStr !== ''}
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

          <Button variant="ghost" onclick={handleCsvUpload} class="w-full">
            <Icon name="chart" size={12} />
            <span class="font-proto text-smaller font-semibold tracking-wider uppercase"
              >{i18n.t.reconcileAutoMatch}</span
            >
          </Button>
        </div>
      {/if}
    </div>

    <!-- Main: Uncleared Transactions HUD Table -->
    <div class="sharp-card flex flex-1 flex-col overflow-y-auto">
      {#if !selectedAccountId}
        <div class="flex h-full items-center justify-center p-8">
          <EmptyState
            title={i18n.t.reconcileEmptyTitle}
            hint={i18n.t.reconcileEmptyHint}
            icon="check"
          />
        </div>
      {:else}
        <!-- Table Control Strip -->
        <div
          class="border-line flex shrink-0 items-center justify-between gap-2 border-b px-3 py-2.5"
        >
          <div class="font-proto text-smaller flex items-center gap-2">
            <span class="text-text-muted">{i18n.t.reconcileStatus}:</span>
            <span class="text-text-strong font-bold">{clearedCount}</span>
            <span class="text-text-muted">/ {unclearedSplitsCount} {i18n.t.clearedStatus}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <Button
              variant="ghost"
              size="sm"
              onclick={selectAllUncleared}
              class="font-proto text-smaller h-6 px-2"
            >
              {i18n.t.reconcileSelectAll}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onclick={clearAllSelected}
              class="font-proto text-smaller h-6 px-2"
            >
              {i18n.t.reconcileDeselectAll}
            </Button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto">
          <div
            class="border-line bg-line/20 font-proto text-text-muted text-smaller sticky top-0 z-10 grid grid-cols-12 gap-2 border-b px-3 py-2 tracking-widest uppercase"
          >
            <div class="col-span-1 text-center">{i18n.t.reconcileColClr}</div>
            <div class="col-span-2">{i18n.t.reconcileColDate}</div>
            <div class="col-span-6">{i18n.t.reconcileColDesc}</div>
            <div class="col-span-3 text-right">{i18n.t.reconcileColAmount}</div>
          </div>

          {#if unclearedTransactions.length === 0}
            <div class="text-text-muted font-proto text-small p-8 text-center">
              {i18n.t.reconcileNoUncleared}
            </div>
          {/if}

          {#each unclearedTransactions as tx (tx.id || tx)}
            {#each tx.splits.filter((s) => s.accountId === selectedAccountId && s.reconcile !== 'y') as split (split.id)}
              <button
                type="button"
                onclick={() => toggleClear(split.id)}
                class="border-line/40 font-proto hover:bg-bg-btn text-small grid w-full grid-cols-12 items-center gap-2 border-b px-3 py-2 text-left transition-colors"
              >
                <div class="col-span-1 flex justify-center">
                  <div
                    class="flex h-3.5 w-3.5 items-center justify-center border transition-colors {clearedMap[
                      split.id
                    ]
                      ? 'bg-teal border-teal text-bg-app'
                      : 'border-line hover:border-text-muted bg-transparent'}"
                  >
                    {#if clearedMap[split.id]}
                      <Icon name="check" size={9} />
                    {/if}
                  </div>
                </div>
                <div class="text-text-dim font-proto col-span-2 tabular-nums">
                  {tx.date}
                </div>
                <div class="text-text-strong col-span-6 truncate">
                  {tx.description}
                  {#if split.memo}
                    <span class="text-text-muted text-smaller">({split.memo})</span>
                  {/if}
                </div>
                <div
                  class="font-proto col-span-3 text-right font-bold tabular-nums {split.amount > 0
                    ? 'text-income'
                    : 'text-expense'}"
                >
                  {formatMoney(split.amount, account?.currency || 'IDR')}
                </div>
              </button>
            {/each}
          {/each}
        </div>
      {/if}
    </div>
  </div>
</PageLayout>
