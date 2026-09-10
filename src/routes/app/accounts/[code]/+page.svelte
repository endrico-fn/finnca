<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    ledgerForAccount,
    fromMinor,
    accountBalanceMinor,
    todayString,
  } from '$lib/accounting/finance';
  import { isDebitNormal } from '$lib/accounting/types';
  import TransactionEditor from '$lib/components/TransactionEditor.svelte';
  import TransferModal from '$lib/components/TransferModal.svelte';
  import ExportOverlay from '$lib/components/ExportOverlay.svelte';
  import type { Transaction, ReconcileState } from '$lib/accounting/types';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { PageLayout, Card, Button, SearchBar } from '$lib/components/ui';

  let exportOpen = $state(false);

  onMount(() => {
    if (!ledger.data) ledger.load();
  });

  const code = $derived(page.params.code ?? '');
  const account = $derived(ledger.accounts.find((a) => a.code === code) ?? null);
  const isPlaceholder = $derived(account?.placeholder ?? false);

  const allEntries = $derived(
    ledger.data && account && !isPlaceholder ? ledgerForAccount(account.id, ledger.data, ledger.childrenMap) : []
  );
  const balance = $derived(
    ledger.data && account && !isPlaceholder ? accountBalanceMinor(account.id, ledger.data) : 0
  );

  let q = $state('');
  let from = $state('');
  let to = $state('');
  let reconcileFilter = $state<'ALL' | ReconcileState>('ALL');
  let showNew = $state(false);
  let editing = $state<Transaction | null>(null);

  let transferOpen = $state(false);

  function setThisMonth() {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const last = new Date(y, now.getMonth() + 1, 0).getDate();
    from = `${y}-${m}-01`;
    to = `${y}-${m}-${String(last).padStart(2, '0')}`;
  }
  function setLast30() {
    const end = new Date();
    const start = new Date(Date.now() - 29 * 86400000);
    from = todayString(start);
    to = todayString(end);
  }
  function clearRange() {
    from = '';
    to = '';
    q = '';
    reconcileFilter = 'ALL';
  }

  const filteredEntries = $derived.by(() => {
    let out = allEntries;
    if (q.trim()) {
      const qq = q.toLowerCase();
      out = out.filter((e) =>
        `${e.tx.description} ${e.tx.num ?? ''} ${e.split.memo ?? ''} ${e.tx.notes ?? ''}`
          .toLowerCase()
          .includes(qq)
      );
    }
    if (from) out = out.filter((e) => e.tx.date >= from);
    if (to) out = out.filter((e) => e.tx.date <= to);
    if (reconcileFilter !== 'ALL') out = out.filter((e) => e.split.reconcile === reconcileFilter);
    return out;
  });

  const periodStats = $derived.by(() => {
    let debit = 0,
      credit = 0,
      cleared = 0;
    for (const e of filteredEntries) {
      if (e.split.amount > 0) debit += e.split.amount;
      else credit += -e.split.amount;
      if (e.split.reconcile === 'y') cleared++;
    }
    return {
      debit,
      credit,
      net: debit - credit,
      cleared,
      count: filteredEntries.length,
    };
  });

  const isAbnormal = $derived.by(() => {
    if (!account || isPlaceholder) return false;
    if (balance === 0) return false;
    return isDebitNormal(account.type) ? balance < 0 : balance > 0;
  });

  async function handleSave(tx: Transaction) {
    await ledger.upsertTransaction(tx);
    editing = null;
    showNew = false;
  }

  async function toggleReconcile(entry: {
    tx: Transaction;
    split: { id: string; reconcile: ReconcileState };
  }) {
    const order: ReconcileState[] = ['n', 'c', 'y'];
    const cur = entry.split.reconcile;
    const next = order[(order.indexOf(cur) + 1) % 3];
    const cloned: Transaction = structuredClone(entry.tx);
    const target = cloned.splits.find((s) => s.id === entry.split.id);
    if (target) {
      target.reconcile = next;
      await ledger.upsertTransaction(cloned);
    }
  }

  const fmt = (n: number) => n.toLocaleString('en-US');
</script>

<PageLayout
  crumb={i18n.t.account}
  crumbHref="/app/accounts"
  title={account ? `${account.code} — ${account.name}` : 'NOT FOUND'}
>
  {#snippet actions()}
    {#if account}
      {#if !isPlaceholder}
        <Button variant="ghost" onclick={() => (exportOpen = true)}>
          {i18n.t.exportCsv}
        </Button>
        <Button
          variant="ghost"
          onclick={() => (transferOpen = true)}
          class="text-income border-income/40 hover:border-income"
        >
          {i18n.t.transferBtn2}
        </Button>
        <Button
          variant="primary"
          onclick={() => {
            showNew = true;
            editing = null;
          }}
        >
          {i18n.t.addEntry}
        </Button>
      {:else}
        <Button variant="ghost" href="/app/accounts">
          {i18n.t.editInAccount}
        </Button>
      {/if}
    {/if}
  {/snippet}

  <ExportOverlay
    bind:open={exportOpen}
    reportTitle={account ? `${account.code} — ${account.name}` : 'ACCOUNT'}
    supportsPDF={false}
    onExport={async (_format, rangeFrom, rangeTo) => {
      if (!account) return;
      const { exportAccountStatementCSV } = await import('$lib/accounting/reports/export');
      const { notifStore } = await import('$lib/notifications/store.svelte');
      const rows = filteredEntries.map(e => ({
        date: e.tx.date,
        description: e.tx.description,
        debit: e.split.amount > 0 ? e.split.amount : 0,
        credit: e.split.amount < 0 ? Math.abs(e.split.amount) : 0,
        balance: e.running,
      }));
      await exportAccountStatementCSV(account, rows, rangeFrom && rangeTo ? `${rangeFrom} to ${rangeTo}` : 'ALL');
      notifStore.addNotification({ type: 'LEDGER_INTEGRITY', priority: 'low', title: i18n.t.exportSuccess, message: `CSV — ${account.name}` });
    }}
  />

  {#if !account}
    <div class="sharp-card flex flex-1 items-center justify-center p-8">
      <div class="text-center">
        <p class="text-expense text-[12px]">
          {i18n.t.accountNotFound.replace('{code}', code)}
        </p>
        <Button variant="primary" href="/app/accounts" class="mt-4 inline-block">
          {i18n.t.backToAccounts}
        </Button>
      </div>
    </div>
  {:else if isPlaceholder}
    <Card
      title="{account.code} — {account.name}"
      badge={i18n.t.badgePhPlaceholder}
      badgeTone="warn"
      class="flex-1 min-h-0"
    >
      <div class="space-y-2 p-3">
        <p class="text-text-muted text-[11px]">{i18n.t.placeholderNotPostable}</p>
        <p class="text-text-muted text-[11px]">{i18n.t.placeholderGroupDetail}</p>
        <div class="mt-4 flex gap-2 pt-2">
          <Button variant="primary" href="/app/accounts">{i18n.t.editInAccount}</Button>
          <Button variant="ghost" href="/app/accounts">{i18n.t.backToAccounts}</Button>
        </div>
      </div>
    </Card>
  {:else}
    <Card
      title="{account.code} — {account.name}"
      description={`${account.type} • ${account.currency} • Postable${account.note ? ` • ${account.note}` : ''}`}
      class="flex-1 min-h-0"
      padding={false}
    >
      {#snippet header()}
        <div class="font-proto shrink-0 text-right text-[10px]">
          <span class="text-text-muted">
            {allEntries.length} ENTRIES • {filteredEntries.length} FILTERED
          </span>
          <span class="text-line mx-2">|</span>
          <span class="text-text-dim">
            RUNNING BALANCE:
            <strong class="text-text-strong ml-1 font-bold tabular-nums">
              {account.currency === 'USD' ? '$' : 'Rp'} {fmt(fromMinor(account.currency, balance))}
            </strong>
          </span>
        </div>
      {/snippet}

      <div class="flex flex-col gap-2 px-3 pt-2 pb-2">
        {#if isAbnormal}
          <div class="border-expense/40 bg-expense/10 text-expense font-proto border px-2 py-1 text-[10px]">
            [ALERT] {i18n.t.abnormalBalance.replace('{type}', account.type)} ({isDebitNormal(
              account.type
            )
              ? i18n.t.shouldBeDebit
              : i18n.t.shouldBeCredit})
          </div>
        {/if}

        <div class="flex shrink-0 flex-wrap items-center gap-2">
          <SearchBar
            bind:value={q}
            placeholder={i18n.t.searchLedgerPlaceholder}
            class="min-w-40 flex-1 max-w-none"
          />
          <label class="text-text-base flex items-center gap-1.5 text-[11px]">
            {i18n.t.from}
            <input type="date" bind:value={from} class="sharp-input px-2 py-1 text-[11px]" />
          </label>
          <label class="text-text-base flex items-center gap-1.5 text-[11px]">
            {i18n.t.to}
            <input type="date" bind:value={to} class="sharp-input px-2 py-1 text-[11px]" />
          </label>
          <select
            bind:value={reconcileFilter}
            class="sharp-input min-w-22.5 px-2 py-1.5 text-[11px]"
          >
            <option value="ALL">{i18n.t.filterAllR}</option>
            <option value="n">{i18n.t.filterNew}</option>
            <option value="c">{i18n.t.filterCleared}</option>
            <option value="y">{i18n.t.filterReconciled}</option>
          </select>
          <div class="ml-auto flex items-center gap-1 text-[10px]">
            <Button variant="ghost" size="sm" onclick={setThisMonth}>{i18n.t.thisMonth}</Button>
            <Button variant="ghost" size="sm" onclick={setLast30}>30D</Button>
            <Button variant="ghost" size="sm" onclick={clearRange} class="text-teal uppercase">{i18n.t.reset}</Button>
          </div>
        </div>

        <div class="font-proto border-line flex shrink-0 flex-wrap items-center gap-4 border-t pt-2 text-[11px]">
          <span class="text-text-muted">
            DEBIT: <strong class="text-income ml-1 tabular-nums">{account.currency === 'USD' ? '$' : 'Rp'} {fmt(fromMinor(account.currency, periodStats.debit))}</strong>
          </span>
          <span class="text-line">|</span>
          <span class="text-text-muted">
            CREDIT: <strong class="text-text-base ml-1 tabular-nums">{account.currency === 'USD' ? '$' : 'Rp'} {fmt(fromMinor(account.currency, periodStats.credit))}</strong>
          </span>
          <span class="text-line">|</span>
          <span class="text-text-muted">
            NET: <strong class="{periodStats.net >= 0 ? 'text-income' : 'text-expense'} ml-1 tabular-nums">{account.currency === 'USD' ? '$' : 'Rp'} {fmt(fromMinor(account.currency, Math.abs(periodStats.net)))} {periodStats.net < 0 ? '(CR)' : ''}</strong>
          </span>
          <span class="text-line">|</span>
          <span class="text-text-muted">
            CLEARED: <strong class="text-teal ml-1 tabular-nums">{periodStats.cleared}/{periodStats.count}</strong> y
          </span>
          {#if filteredEntries.length !== allEntries.length}
            <span class="text-text-dim ml-auto text-[10px]">
              {filteredEntries.length}/{allEntries.length} shown
            </span>
          {/if}
        </div>
      </div>

      {#if editing}
        <div class="mb-3 shrink-0 px-3 pt-1 pb-2">
          <TransactionEditor
            tx={editing}
            onSave={handleSave}
            onCancel={() => (editing = null)}
            onDelete={async (id: string) => {
              await ledger.deleteTransaction(id);
              editing = null;
            }}
          />
        </div>
      {/if}

      <div class="min-h-0 flex-1 overflow-y-auto px-3 py-1">
        {#if filteredEntries.length === 0}
          <div class="flex h-full items-center justify-center p-8">
            <div class="text-center">
              <p class="text-text-muted text-[12px]">
                {#if allEntries.length === 0}
                  {i18n.t.noTxForAccount}
                {:else}
                  {i18n.t.noEntriesMatchFilter}
                {/if}
              </p>
              <div class="mt-3 flex justify-center gap-2">
                <Button
                  variant="primary"
                  size="sm"
                  onclick={() => {
                    showNew = true;
                    editing = null;
                  }}
                >
                  {i18n.t.newEntryBtn}
                </Button>
                <Button variant="ghost" size="sm" href="/app/journal">
                  {i18n.t.journal} →
                </Button>
              </div>
            </div>
          </div>
        {:else}
          <table class="w-full border-collapse font-mono text-[12px]">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr class="border-line text-text-base border-b">
                <th class="label-xs w-27.5 py-2 pl-2 text-left font-normal">{i18n.t.date}</th>
                <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.description}</th>
                <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.offsetLabel}</th>
                <th class="label-xs w-27.5 px-3 py-2 text-right font-normal">{i18n.t.debit}</th>
                <th class="label-xs w-27.5 px-3 py-2 text-right font-normal">{i18n.t.credit}</th>
                <th class="label-xs w-11.5 px-1 py-2 text-center font-normal">R</th>
                <th class="label-xs w-32.5 py-2 pr-2 text-right font-normal">{i18n.t.colBalance}</th>
                <th class="w-12.5 py-2 pr-2"></th>
              </tr>
            </thead>
            <tbody class="divide-line/40 divide-y">
              {#each filteredEntries as e (e.tx.id + e.split.id)}
                {@const other = e.tx.splits.find((s) => s.accountId !== e.split.accountId)}
                {@const otherAcc = other ? ledger.accountsById.get(other.accountId) : null}
                <tr class="hover:bg-bg-row-active/40 transition-colors">
                  <td class="text-text-base font-proto py-2 pl-2 whitespace-nowrap">{e.tx.date}</td>
                  <td class="text-text-strong px-3 py-2">
                    <div class="flex items-center gap-1.5">
                      <span class="truncate">{e.tx.description}</span>
                      {#if e.tx.num}<span
                          class="border-line bg-bg-app text-text-muted font-proto border px-1 text-[9px]"
                          >{e.tx.num}</span
                        >{/if}
                    </div>
                    {#if e.split.memo}<p class="text-text-muted mt-0.5 text-[10px]">
                        {e.split.memo}
                      </p>{/if}
                  </td>
                  <td class="text-text-muted max-w-35 truncate px-3 py-2 text-[11px]"
                    >{otherAcc ? `${otherAcc.code} ${otherAcc.name}` : '—'}</td
                  >
                  <td
                    class="font-proto px-3 py-2 text-right tabular-nums {e.split.amount > 0
                      ? 'text-income'
                      : 'text-text-dim'}"
                  >
                    {e.split.amount > 0
                      ? fmt(fromMinor(e.tx.currency, e.split.amount))
                      : '—'}
                  </td>
                  <td
                    class="font-proto px-3 py-2 text-right tabular-nums {e.split.amount < 0
                      ? 'text-text-strong'
                      : 'text-text-dim'}"
                  >
                    {e.split.amount < 0
                      ? fmt(fromMinor(e.tx.currency, -e.split.amount))
                      : '—'}
                  </td>
                  <td class="px-1 py-2 text-center">
                    <button
                      type="button"
                      onclick={() => toggleReconcile(e)}
                      title="Cycle n→c→y"
                      class="grid h-6 w-6 place-items-center border text-[10px] font-bold transition-colors {e
                        .split.reconcile === 'y'
                        ? 'bg-income/15 border-income/40 text-income'
                        : e.split.reconcile === 'c'
                          ? 'bg-warning/15 border-warning/40 text-warning'
                          : 'bg-bg-app border-line text-text-muted hover:border-teal/40'}"
                    >
                      {e.split.reconcile}
                    </button>
                  </td>
                  <td class="text-text-strong font-proto py-2 pr-2 text-right font-medium tabular-nums"
                    >{fmt(fromMinor(account.currency, e.running))}</td
                  >
                  <td class="py-2 pr-2 text-right">
                    <Button variant="ghost" size="sm" onclick={() => (editing = e.tx)}
                      >{i18n.t.edit}</Button
                    >
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    </Card>
  {/if}

  {#if showNew && account && !isPlaceholder}
    <div class="bg-overlay/60 fixed inset-0 z-50 flex items-center justify-center p-4 font-mono">
      <div class="anim-modal max-h-[90vh] w-full max-w-4xl overflow-y-auto">
        <TransactionEditor tx={null} onSave={handleSave} onCancel={() => (showNew = false)} />
      </div>
    </div>
  {/if}

  <TransferModal bind:open={transferOpen} initialFrom={account?.id ?? ''} />
</PageLayout>
