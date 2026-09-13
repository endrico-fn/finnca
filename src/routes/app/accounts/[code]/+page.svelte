<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    ledgerForAccount,
    fromMinor,
    accountBalanceMinor,
    formatMoney,
  } from '$lib/accounting/finance';
  import { isDebitNormal, accountTypeLabel } from '$lib/accounting/types';
  import TransactionEditor from '$lib/components/TransactionEditor.svelte';
  import TransferModal from '$lib/components/TransferModal.svelte';
  import ExportOverlay from '$lib/components/ExportOverlay.svelte';
  import type { Transaction, ReconcileState } from '$lib/accounting/types';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import {
    PageLayout,
    Card,
    Button,
    SearchBar,
    FilterMenu,
    FilterSection,
    FilterOption,
    DateRangeDropdown,
  } from '$lib/components/ui';

  let exportOpen = $state(false);

  onMount(() => {
    if (!ledger.data) ledger.load();
  });

  const code = $derived(page.params.code ?? '');
  const account = $derived(ledger.accounts.find((a) => a.code === code) ?? null);
  const isPlaceholder = $derived(account?.placeholder ?? false);

  const allEntries = $derived(
    ledger.data && account && !isPlaceholder
      ? ledgerForAccount(account.id, ledger.data, ledger.childrenMap)
      : []
  );
  const balance = $derived(
    ledger.data && account && !isPlaceholder ? accountBalanceMinor(account.id, ledger.data) : 0
  );

  let q = $state('');
  let from = $state('');
  let to = $state('');
  let reconcileFilter = $state<'ALL' | ReconcileState>('ALL');
  let statusFilterOpen = $state(false);
  let showNew = $state(false);
  let editing = $state<Transaction | null>(null);

  let transferOpen = $state(false);

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

  const baseForStatus = $derived.by(() => {
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
    return out;
  });

  const statusOptions = $derived([
    { id: 'ALL' as const, label: i18n.t.filterAllR },
    { id: 'n' as const, label: i18n.t.filterNew },
    { id: 'c' as const, label: i18n.t.filterCleared },
    { id: 'y' as const, label: i18n.t.filterReconciled },
  ]);

  const statusCounts = $derived.by(() => {
    const counts: Record<'ALL' | ReconcileState, number> = {
      ALL: baseForStatus.length,
      n: 0,
      c: 0,
      y: 0,
    };
    for (const e of baseForStatus) counts[e.split.reconcile] += 1;
    return counts;
  });

  const statusFilterLabel = $derived(
    statusOptions.find((o) => o.id === reconcileFilter)?.label ?? i18n.t.filterBtn
  );

  function resetStatusFilter() {
    reconcileFilter = 'ALL';
  }

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
  title={account
    ? `${account.code} — ${account.name}`
    : i18n.t.accountNotFound.replace('{code}', code)}
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
    reportTitle={account ? `${account.code} — ${account.name}` : i18n.t.account}
    supportsPDF={false}
    onExport={async (_format, rangeFrom, rangeTo) => {
      if (!account) return;
      const { exportAccountStatementCSV } = await import('$lib/accounting/reports/export');
      const { notifStore } = await import('$lib/notifications/store.svelte');
      const rows = filteredEntries.map((e) => ({
        date: e.tx.date,
        description: e.tx.description,
        debit: e.split.amount > 0 ? e.split.amount : 0,
        credit: e.split.amount < 0 ? Math.abs(e.split.amount) : 0,
        balance: e.running,
      }));
      await exportAccountStatementCSV(
        account,
        rows,
        rangeFrom && rangeTo ? `${rangeFrom} → ${rangeTo}` : i18n.t.allTime
      );
      notifStore.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'low',
        title: i18n.t.exportSuccess,
        message: `CSV — ${account.name}`,
      });
    }}
  />

  {#if !account}
    <div class="sharp-card flex flex-1 items-center justify-center p-8">
      <div class="text-center">
        <p class="text-expense text-small">
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
      class="min-h-0 flex-1"
    >
      <div class="space-y-2 p-3">
        <p class="text-text-muted text-small">{i18n.t.placeholderNotPostable}</p>
        <p class="text-text-muted text-small">{i18n.t.placeholderGroupDetail}</p>
        <div class="mt-4 flex gap-2 pt-2">
          <Button variant="primary" href="/app/accounts">{i18n.t.editInAccount}</Button>
          <Button variant="ghost" href="/app/accounts">{i18n.t.backToAccounts}</Button>
        </div>
      </div>
    </Card>
  {:else}
    <Card
      title="{account.code} — {account.name}"
      description={`${accountTypeLabel(account.type)} • ${account.currency} • ${i18n.t.accountPostableTag}${account.note ? ` • ${account.note}` : ''}`}
      class="min-h-0 flex-1"
      padding={false}
    >
      {#snippet header()}
        <div class="font-proto text-smaller shrink-0 text-right">
          <span class="text-text-muted">
            {allEntries.length}
            {i18n.t.entriesLabel} • {filteredEntries.length}
            {i18n.t.filteredLabel}
          </span>
          <span class="text-line mx-2">|</span>
          <span class="text-text-dim">
            {i18n.t.runningBalanceLabel}
            <strong class="text-text-strong ml-1 font-bold tabular-nums">
              {formatMoney(balance, account.currency)}
            </strong>
          </span>
        </div>
      {/snippet}

      <div class="flex flex-col gap-2 px-3 pt-2 pb-2">
        {#if isAbnormal}
          <div
            class="border-expense/40 bg-expense/10 text-expense font-proto text-smaller border px-2 py-1"
          >
            {i18n.t.alertTag}{i18n.t.abnormalBalance.replace(
              '{type}',
              accountTypeLabel(account.type)
            )}
            ({isDebitNormal(account.type) ? i18n.t.shouldBeDebit : i18n.t.shouldBeCredit})
          </div>
        {/if}

        <div class="flex shrink-0 flex-wrap items-center gap-2">
          <SearchBar
            bind:value={q}
            placeholder={i18n.t.searchLedgerPlaceholder}
            class="max-w-none min-w-40 flex-1"
          />
          <FilterMenu
            bind:open={statusFilterOpen}
            label={statusFilterLabel}
            active={reconcileFilter !== 'ALL'}
            count={reconcileFilter !== 'ALL' ? 1 : 0}
            onReset={resetStatusFilter}
            resetLabel={i18n.t.reset}
            resetDisabled={reconcileFilter === 'ALL'}
          >
            <FilterSection title={i18n.t.filterStatusTitle} layout="list">
              {#each statusOptions as opt (opt.id)}
                <FilterOption
                  label={opt.label}
                  count={statusCounts[opt.id]}
                  selected={reconcileFilter === opt.id}
                  check={opt.id !== 'ALL'}
                  onclick={() => {
                    reconcileFilter = opt.id;
                    statusFilterOpen = false;
                  }}
                />
              {/each}
            </FilterSection>
          </FilterMenu>
          <DateRangeDropdown bind:from bind:to size="md" />
          <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
            {filteredEntries.length}/{allEntries.length}
          </span>
          {#if q || from || to || reconcileFilter !== 'ALL'}
            <div class="text-smaller ml-auto flex items-center gap-1">
              <Button variant="ghost" size="sm" onclick={clearRange}>{i18n.t.reset}</Button>
            </div>
          {/if}
        </div>

        <div
          class="font-proto border-line text-small flex shrink-0 flex-wrap items-center gap-4 border-t pt-2"
        >
          <span class="text-text-muted">
            {i18n.t.debitLabel}
            <strong class="text-income ml-1 tabular-nums"
              >{account.currency === 'USD' ? '$' : 'Rp'}
              {fmt(fromMinor(account.currency, periodStats.debit))}</strong
            >
          </span>
          <span class="text-line">|</span>
          <span class="text-text-muted">
            {i18n.t.creditLabel}
            <strong class="text-text-base ml-1 tabular-nums"
              >{account.currency === 'USD' ? '$' : 'Rp'}
              {fmt(fromMinor(account.currency, periodStats.credit))}</strong
            >
          </span>
          <span class="text-line">|</span>
          <span class="text-text-muted">
            {i18n.t.netLabel}
            <strong
              class="{periodStats.net >= 0 ? 'text-income' : 'text-expense'} ml-1 tabular-nums"
              >{account.currency === 'USD' ? '$' : 'Rp'}
              {fmt(fromMinor(account.currency, Math.abs(periodStats.net)))}
              {periodStats.net < 0 ? i18n.t.crSuffix : ''}</strong
            >
          </span>
          <span class="text-line">|</span>
          <span class="text-text-muted">
            {i18n.t.clearedLabel}
            <strong class="text-teal ml-1 tabular-nums"
              >{periodStats.cleared}/{periodStats.count}</strong
            >
            {i18n.t.reconcileCountSuffix}
          </span>
          {#if filteredEntries.length !== allEntries.length}
            <span class="text-text-dim text-smaller ml-auto">
              {filteredEntries.length}/{allEntries.length}
              {i18n.t.shownSuffix}
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
              <p class="text-text-muted text-small">
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
          <table class="font-proto text-small w-full border-collapse">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr class="border-line text-text-base border-b">
                <th class="label-xs w-28 py-2 pl-2 text-left font-normal">{i18n.t.date}</th>
                <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.description}</th>
                <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.offsetLabel}</th>
                <th class="label-xs w-28 px-3 py-2 text-right font-normal">{i18n.t.debit}</th>
                <th class="label-xs w-28 px-3 py-2 text-right font-normal">{i18n.t.credit}</th>
                <th class="label-xs w-12 px-1 py-2 text-center font-normal"
                  >{i18n.t.reconcileColShort}</th
                >
                <th class="label-xs w-32 py-2 pr-2 text-right font-normal">{i18n.t.colBalance}</th>
                <th class="w-12 py-2 pr-2"></th>
              </tr>
            </thead>
            <tbody class="divide-line/40 divide-y">
              {#each filteredEntries as e (e.tx.id + e.split.id)}
                {@const other = e.tx.splits.find((s) => s.accountId !== e.split.accountId)}
                {@const otherAcc = other ? ledger.accountsById.get(other.accountId) : null}
                <tr class="hover:bg-bg-row-active transition-colors">
                  <td class="text-text-base font-proto py-2 pl-2 whitespace-nowrap">{e.tx.date}</td>
                  <td class="text-text-strong px-3 py-2">
                    <div class="flex items-center gap-1.5">
                      <span class="truncate">{e.tx.description}</span>
                      {#if e.tx.num}<span
                          class="border-line bg-bg-app text-text-muted font-proto text-smaller border px-1"
                          >{e.tx.num}</span
                        >{/if}
                    </div>
                    {#if e.split.memo}<p class="text-text-muted text-smaller mt-0.5">
                        {e.split.memo}
                      </p>{/if}
                  </td>
                  <td class="text-text-muted text-small max-w-36 truncate px-3 py-2"
                    >{otherAcc ? `${otherAcc.code} ${otherAcc.name}` : '—'}</td
                  >
                  <td
                    class="font-proto px-3 py-2 text-right tabular-nums {e.split.amount > 0
                      ? 'text-income'
                      : 'text-text-dim'}"
                  >
                    {e.split.amount > 0 ? fmt(fromMinor(e.tx.currency, e.split.amount)) : '—'}
                  </td>
                  <td
                    class="font-proto px-3 py-2 text-right tabular-nums {e.split.amount < 0
                      ? 'text-text-strong'
                      : 'text-text-dim'}"
                  >
                    {e.split.amount < 0 ? fmt(fromMinor(e.tx.currency, -e.split.amount)) : '—'}
                  </td>
                  <td class="px-1 py-2 text-center">
                    <button
                      type="button"
                      onclick={() => toggleReconcile(e)}
                      title={i18n.t.reconcileCycleHint}
                      class="text-smaller grid h-6 w-6 place-items-center border font-bold transition-colors {e
                        .split.reconcile === 'y'
                        ? 'bg-income/15 border-income/40 text-income'
                        : e.split.reconcile === 'c'
                          ? 'bg-warning/15 border-warning/40 text-warning'
                          : 'bg-bg-app border-line text-text-muted hover:border-teal/40'}"
                    >
                      {e.split.reconcile}
                    </button>
                  </td>
                  <td
                    class="text-text-strong font-proto py-2 pr-2 text-right font-medium tabular-nums"
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
    <div class="bg-overlay font-proto fixed inset-0 z-50 flex items-center justify-center p-4">
      <div class="anim-modal max-h-[90vh] w-full max-w-4xl overflow-y-auto">
        <TransactionEditor tx={null} onSave={handleSave} onCancel={() => (showNew = false)} />
      </div>
    </div>
  {/if}

  <TransferModal bind:open={transferOpen} initialFrom={account?.id ?? ''} />
</PageLayout>
