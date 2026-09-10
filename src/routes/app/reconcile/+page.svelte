<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { formatIDR, fromMinor, toMinor, reconciledBalanceMinor, clearedBalanceMinor } from '$lib/accounting/finance';
  import { PageLayout, Button, Icon, EmptyState } from '$lib/components/ui';
  import Papa from 'papaparse';
  import { notifStore } from '$lib/notifications/store.svelte';

  let selectedAccountId = $state('');
  let targetBalanceStr = $state('');

  // Local state for cleared splits: splitId -> boolean
  let clearedMap = $state<Record<string, boolean>>({});

  let prevAccountId = '';
  
  $effect(() => {
    if (selectedAccountId !== prevAccountId) {
      prevAccountId = selectedAccountId;
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
  });

  const account = $derived(ledger.accounts.find((a) => a.id === selectedAccountId));

  const startingBalance = $derived(
    selectedAccountId && ledger.data ? reconciledBalanceMinor(selectedAccountId, ledger.data) : 0
  );

  const clearedBalance = $derived(
    selectedAccountId && ledger.data ? clearedBalanceMinor(selectedAccountId, ledger.data, clearedMap) : 0
  );

  const targetBalance = $derived(
    targetBalanceStr ? toMinor(account?.currency || 'IDR', Number(targetBalanceStr)) : 0
  );
  const difference = $derived(targetBalance - clearedBalance);

  const unclearedTransactions = $derived.by(() => {
    if (!selectedAccountId) return [];
    return ledger.transactions
      .filter((t) => t.splits.some((s) => s.accountId === selectedAccountId && s.reconcile !== 'y'))
      .sort((a, b) => a.date.localeCompare(b.date));
  });

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

    // Update all selected splits to 'y'
    for (const tx of ledger.transactions) {
      let modified = false;
      for (const s of tx.splits) {
        if (s.accountId === selectedAccountId && clearedMap[s.id]) {
          s.reconcile = 'y';
          modified = true;
        }
      }
      if (modified) {
        await ledger.upsertTransaction(tx); // Trigger store update
      }
    }

    targetBalanceStr = '';
    selectedAccountId = '';
  }

  let bankPreset = $state<'AUTO' | 'BCA' | 'MANDIRI' | 'BRI' | 'BNI'>('AUTO');

  // --- AUTO-RECONCILE / CSV SYNC WITH BANK PRESETS ---
  function handleCsvUpload(e: Event) {
    if (!selectedAccountId) {
      notifStore.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.reconcileTitle,
        message: i18n.t.reconcileSelectAccountWarning,
      });
      return;
    }
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;

    const file = target.files[0];
    Papa.parse(file, {
      header: true,
      skipEmptyLines: true,
      complete: async (results) => {
        let matchedCount = 0;
        let unmatchedCount = 0;

        for (const row of results.data as any[]) {
          const keys = Object.keys(row);
          let dateKey = '';
          let amountKey = '';
          let typeKey = '';
          let debitKey = '';
          let creditKey = '';

          if (bankPreset === 'BCA') {
            dateKey =
              keys.find(
                (k) => k.toLowerCase().includes('tgl') || k.toLowerCase().includes('date')
              ) || '';
            amountKey =
              keys.find(
                (k) => k.toLowerCase().includes('mutasi') || k.toLowerCase().includes('amount')
              ) || '';
            typeKey =
              keys.find(
                (k) => k.toLowerCase().includes('type') || k.toLowerCase().includes('cr/db')
              ) || '';
          } else if (bankPreset === 'MANDIRI' || bankPreset === 'BRI' || bankPreset === 'BNI') {
            dateKey =
              keys.find(
                (k) =>
                  k.toLowerCase().includes('tanggal') ||
                  k.toLowerCase().includes('tgl') ||
                  k.toLowerCase().includes('date')
              ) || '';
            debitKey =
              keys.find(
                (k) => k.toLowerCase().includes('debet') || k.toLowerCase().includes('debit')
              ) || '';
            creditKey =
              keys.find(
                (k) => k.toLowerCase().includes('kredit') || k.toLowerCase().includes('credit')
              ) || '';
          }

          if (!dateKey) {
            dateKey =
              keys.find(
                (k) =>
                  k.toLowerCase().includes('date') ||
                  k.toLowerCase().includes('tanggal') ||
                  k.toLowerCase().includes('tgl')
              ) || '';
          }

          if (!amountKey && !debitKey && !creditKey) {
            amountKey =
              keys.find(
                (k) =>
                  k.toLowerCase().includes('amount') ||
                  k.toLowerCase().includes('nominal') ||
                  k.toLowerCase().includes('jumlah') ||
                  k.toLowerCase().includes('mutasi')
              ) || '';
            typeKey =
              keys.find(
                (k) =>
                  k.toLowerCase().includes('type') ||
                  k.toLowerCase().includes('jenis') ||
                  k.toLowerCase().includes('status')
              ) || '';
          }

          const rowDateStr = row[dateKey];
          if (!rowDateStr) continue;

          let dateObj = new Date(rowDateStr);
          if (isNaN(dateObj.getTime())) {
            const parts = String(rowDateStr)
              .trim()
              .split(/[\/\-]/);
            if (parts.length === 3) {
              const y = parts[2].length === 2 ? `20${parts[2]}` : parts[2];
              dateObj = new Date(`${y}-${parts[1]}-${parts[0]}`);
            }
          }
          if (isNaN(dateObj.getTime())) continue;
          const time = dateObj.getTime();

          let amtRaw = 0;
          if (debitKey && row[debitKey]) {
            const v = parseFloat(String(row[debitKey]).replace(/[^\d.-]/g, ''));
            if (!isNaN(v) && v > 0) amtRaw = -v;
          }
          if (creditKey && row[creditKey] && amtRaw === 0) {
            const v = parseFloat(String(row[creditKey]).replace(/[^\d.-]/g, ''));
            if (!isNaN(v) && v > 0) amtRaw = v;
          }
          if (amtRaw === 0 && amountKey && row[amountKey]) {
            amtRaw = parseFloat(String(row[amountKey]).replace(/[^\d.-]/g, '')) || 0;
            const t = String(typeKey ? row[typeKey] : row[amountKey]).toLowerCase();
            if (t.includes('db') || t.includes('debit') || t.includes('d'))
              amtRaw = -Math.abs(amtRaw);
            else if (t.includes('cr') || t.includes('kredit') || t.includes('c'))
              amtRaw = Math.abs(amtRaw);
          }

          if (amtRaw === 0) continue;

          let matched = false;
          for (const tx of unclearedTransactions) {
            const s = tx.splits.find(
              (sp) =>
                sp.accountId === selectedAccountId && sp.reconcile !== 'y' && !clearedMap[sp.id]
            );
            if (s) {
              const txTime = new Date(tx.date).getTime();
              const diffDays = Math.abs((txTime - time) / 86400000);
              const minorAmtRaw = toMinor(account?.currency || 'IDR', amtRaw);
              const isAmtMatch = Math.abs(s.amount) === Math.abs(minorAmtRaw);

              if (diffDays <= 4 && isAmtMatch) {
                clearedMap[s.id] = true;
                matched = true;
                matchedCount++;
                break;
              }
            }
          }
          if (!matched) unmatchedCount++;
        }

        notifStore.addNotification({
          type: 'LEDGER_INTEGRITY',
          title: i18n.t.reconcileCsvSyncCompleteTitle,
          message: i18n.t.reconcileCsvSyncCompleteMsg
            .replace('{matched}', String(matchedCount))
            .replace('{unmatched}', String(unmatchedCount)),
          priority: 'high',
        });
      },
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
    <!-- Sidebar: Telemetry & Controls -->
    <div class="sharp-card flex w-80 shrink-0 flex-col gap-4 overflow-y-auto p-3">
      <div>
        <label
          for="reconcile-account"
          class="font-proto text-text-muted mb-1.5 block text-[10px] tracking-widest uppercase"
          >// {i18n.t.reconcileSelectAccount}</label
        >
        <select
          id="reconcile-account"
          bind:value={selectedAccountId}
          class="sharp-input font-proto bg-bg-app border-line focus:border-teal h-8 w-full border px-2 text-xs"
        >
          <option value="">{i18n.t.reconcileChooseAccount}</option>
          {#each ledger.accounts.filter((a) => !a.placeholder && (a.type === 'ASSET' || a.type === 'LIABILITY')) as acc (acc.id || acc)}
            <option value={acc.id}>{acc.code} - {acc.name}</option>
          {/each}
        </select>
      </div>

      {#if selectedAccountId}
        <div>
          <label
            for="stmt-balance"
            class="font-proto text-text-muted mb-1.5 block text-[10px] tracking-widest uppercase"
            >// {i18n.t.reconcileStatementBalance}</label
          >
          <div class="relative">
            <span
              class="font-proto text-teal absolute top-1/2 left-2.5 -translate-y-1/2 text-[10px] font-bold"
              >{account?.currency}</span
            >
            <input
              id="stmt-balance"
              type="number"
              bind:value={targetBalanceStr}
              class="sharp-input font-proto bg-bg-app border-line focus:border-teal h-9 w-full border px-2 pl-12 text-xs font-bold"
              placeholder="0"
            />
          </div>
        </div>

        <!-- Telemetry Balance Status -->
        <div class="sharp-card bg-bg-app/50 border-line flex flex-col gap-2.5 border p-3">
          <div class="font-proto flex items-center justify-between text-[10px]">
            <span class="text-text-muted uppercase">// {i18n.t.reconcileStartingBalance}</span>
            <span class="text-text-strong font-medium tabular-nums font-proto"
              >{formatIDR(fromMinor('IDR', startingBalance))}</span
            >
          </div>
          <div class="font-proto flex items-center justify-between text-[10px]">
            <span class="text-text-muted uppercase">// {i18n.t.reconcileClearedBalance}</span>
            <span class="text-teal font-bold tabular-nums font-proto"
              >{formatIDR(fromMinor('IDR', clearedBalance))}</span
            >
          </div>
          <div class="bg-line/60 h-px w-full"></div>
          <div class="font-proto flex items-center justify-between text-[11px]">
            <span class="text-text-strong tracking-wider uppercase"
              >// {i18n.t.reconcileDifference}</span
            >
            <span
              class="font-bold tabular-nums font-proto {difference === 0 ? 'text-income' : 'text-expense'}"
            >
              {formatIDR(fromMinor('IDR', difference))}
            </span>
          </div>
          <div class="flex justify-end pt-1">
            <span
              class="font-proto border px-1.5 py-0.5 text-[9px] {difference === 0 &&
              targetBalanceStr !== ''
                ? 'border-income/40 text-income bg-income/10'
                : 'border-line text-text-dim'}"
            >
              {difference === 0 && targetBalanceStr !== '' ? i18n.t.zeroDiscrepancy : i18n.t.outOfBalance}
            </span>
          </div>
        </div>

        {#if difference === 0 && targetBalanceStr !== ''}
          <div
            class="bg-income/10 border-income/30 text-income font-proto border p-2 text-center text-[10px]"
          >
            {i18n.t.reconcileReadyMsg}
          </div>
        {/if}

        <div class="border-line mt-1 flex flex-col gap-2 border-t pt-3">
          <div>
            <label for="bank-preset-select" class="label-xs text-text-muted mb-1 block">
              // {i18n.t.bankPresetLabel}
            </label>
            <select
              id="bank-preset-select"
              bind:value={bankPreset}
              class="sharp-input font-proto w-full cursor-pointer px-2 py-1 text-[11px]"
            >
              <option value="AUTO">{i18n.t.bankPresetCustom}</option>
              <option value="BCA">{i18n.t.bankPresetBca}</option>
              <option value="MANDIRI">{i18n.t.bankPresetMandiri}</option>
              <option value="BRI">{i18n.t.bankPresetBri}</option>
              <option value="BNI">{i18n.t.bankPresetBni}</option>
            </select>
          </div>

          <label
            for="csv-upload"
            class="sharp-btn btn-ghost border-line hover:border-teal flex h-8 w-full cursor-pointer items-center justify-center gap-2 border transition-all"
          >
            <Icon name="chart" size={12} />
            <span class="font-proto text-[10px] font-semibold tracking-wider uppercase"
              >{i18n.t.reconcileAutoMatch}</span
            >
            <input
              id="csv-upload"
              type="file"
              accept=".csv"
              class="hidden"
              onchange={handleCsvUpload}
            />
          </label>
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
          class="border-line bg-line/10 flex shrink-0 items-start justify-between border-b px-3 py-2"
        >
          <div class="font-proto flex items-center gap-2 text-[10px]">
            <span class="text-text-muted">// STATUS:</span>
            <span class="text-text-strong font-bold">{clearedCount}</span>
            <span class="text-text-muted">/ {unclearedTransactions.length} {i18n.t.clearedStatus}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <Button
              variant="ghost"
              size="sm"
              onclick={selectAllUncleared}
              class="font-proto h-6 px-2 text-[9px]"
            >
              {i18n.t.reconcileSelectAll}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onclick={clearAllSelected}
              class="font-proto h-6 px-2 text-[9px]"
            >
              {i18n.t.reconcileDeselectAll}
            </Button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto">
          <div
            class="border-line bg-line/20 font-proto text-text-muted sticky top-0 z-10 grid grid-cols-12 gap-2 border-b px-3 py-2 text-[10px] tracking-widest uppercase"
          >
            <div class="col-span-1 text-center">{i18n.t.reconcileColClr}</div>
            <div class="col-span-2">{i18n.t.reconcileColDate}</div>
            <div class="col-span-6">{i18n.t.reconcileColDesc}</div>
            <div class="col-span-3 text-right">{i18n.t.reconcileColAmount}</div>
          </div>

          {#if unclearedTransactions.length === 0}
            <div class="text-text-muted font-proto p-8 text-center text-[11px]">
              {i18n.t.reconcileNoUncleared}
            </div>
          {/if}

          {#each unclearedTransactions as tx (tx.id || tx)}
            {@const split = tx.splits.find(
              (s) => s.accountId === selectedAccountId && s.reconcile !== 'y'
            )}
            {#if split}
              <button
                type="button"
                onclick={() => toggleClear(split.id)}
                class="border-line/40 font-proto hover:bg-bg-row-hover grid w-full grid-cols-12 items-center gap-2 border-b px-3 py-2 text-left text-[11px] transition-colors"
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
                <div class="text-text-dim font-proto col-span-2 tabular-nums font-proto">{tx.date}</div>
                <div class="text-text-strong col-span-6 truncate">
                  {tx.description}
                </div>
                <div
                  class="font-proto col-span-3 text-right font-bold tabular-nums font-proto {split.amount > 0
                    ? 'text-income'
                    : 'text-expense'}"
                >
                  {formatIDR(fromMinor('IDR', split.amount))}
                </div>
              </button>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  </div>
</PageLayout>
