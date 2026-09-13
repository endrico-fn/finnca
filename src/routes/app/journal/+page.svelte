<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import TransactionEditor from '$lib/components/TransactionEditor.svelte';
  import type { Transaction } from '$lib/accounting/types';
  import { transactionImbalance, formatIDR, formatUSD } from '$lib/accounting/finance';
  import { onMount } from 'svelte';
  import {
    Splash,
    ErrorState,
    PageLayout,
    Card,
    Button,
    Icon,
    SearchBar,
    Pagination,
    FilterMenu,
    FilterSection,
    FilterOption,
    DateRangeDropdown,
    ModalShell,
  } from '$lib/components/ui';

  let q = $state('');
  let from = $state('');
  let to = $state('');
  let accFilter = $state('');
  let accFilterOpen = $state(false);
  let editing = $state<Transaction | null>(null);
  let showNew = $state(false);
  let expandedId = $state<string | null>(null);
  let txPage = $state(1);
  const TX_PAGE_SIZE = 20;

  onMount(() => {
    if (!ledger.data) ledger.load();
  });

  function resetTxPage() {
    txPage = 1;
  }

  const filtered = $derived(
    ledger.transactions
      .filter((t) => {
        if (
          q &&
          !`${t.description} ${t.num ?? ''} ${t.notes ?? ''}`
            .toLowerCase()
            .includes(q.toLowerCase())
        )
          return false;
        if (from && t.date < from) return false;
        if (to && t.date > to) return false;
        if (accFilter && !t.splits.some((s) => s.accountId === accFilter)) return false;
        return true;
      })
      .sort((a, b) => b.date.localeCompare(a.date))
  );

  const totalTxPages = $derived(Math.max(1, Math.ceil(filtered.length / TX_PAGE_SIZE)));
  const pagedTxs = $derived(filtered.slice((txPage - 1) * TX_PAGE_SIZE, txPage * TX_PAGE_SIZE));

  const baseTxs = $derived(
    ledger.transactions.filter((t) => {
      if (
        q &&
        !`${t.description} ${t.num ?? ''} ${t.notes ?? ''}`.toLowerCase().includes(q.toLowerCase())
      )
        return false;
      if (from && t.date < from) return false;
      if (to && t.date > to) return false;
      return true;
    })
  );

  const accTxCounts = $derived.by(() => {
    const counts: Record<string, number> = {};
    for (const t of baseTxs) {
      const seen: Record<string, true> = {};
      for (const s of t.splits) {
        if (seen[s.accountId]) continue;
        seen[s.accountId] = true;
        counts[s.accountId] = (counts[s.accountId] ?? 0) + 1;
      }
    }
    return counts;
  });

  const leafAccounts = $derived(
    ledger.accounts
      .filter((a) => !a.placeholder)
      .sort((a, b) => a.code.localeCompare(b.code, undefined, { numeric: true }))
  );

  const selectedAcc = $derived(accFilter ? ledger.accountsById.get(accFilter) : undefined);

  const accFilterLabel = $derived(
    selectedAcc ? `${selectedAcc.code} ${selectedAcc.name}` : i18n.t.filterBtn
  );

  const idrDebit = $derived(
    filtered
      .filter((t) => t.currency === 'IDR')
      .flatMap((t) => t.splits)
      .filter((s) => s.amount > 0)
      .reduce((a, c) => a + c.amount, 0)
  );
  const idrCredit = $derived(
    filtered
      .filter((t) => t.currency === 'IDR')
      .flatMap((t) => t.splits)
      .filter((s) => s.amount < 0)
      .reduce((a, c) => a - c.amount, 0)
  );
  const usdDebit = $derived(
    filtered
      .filter((t) => t.currency === 'USD')
      .flatMap((t) => t.splits)
      .filter((s) => s.amount > 0)
      .reduce((a, c) => a + c.amount, 0)
  );
  const usdCredit = $derived(
    filtered
      .filter((t) => t.currency === 'USD')
      .flatMap((t) => t.splits)
      .filter((s) => s.amount < 0)
      .reduce((a, c) => a - c.amount, 0)
  );

  let selectedRowIndex = $state<number | null>(null);

  function handleKeydown(e: KeyboardEvent) {
    // Don't trigger if user is typing in an input/textarea
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    // Command Palette CMD+K is handled globally, but we can do j/k here
    if (e.key === 'j') {
      e.preventDefault();
      if (selectedRowIndex === null) selectedRowIndex = 0;
      else selectedRowIndex = Math.min(pagedTxs.length - 1, selectedRowIndex + 1);

      const tx = pagedTxs[selectedRowIndex];
      if (tx) {
        // scroll into view
        document.getElementById(`tx-row-${tx.id}`)?.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'k') {
      e.preventDefault();
      if (selectedRowIndex === null) selectedRowIndex = pagedTxs.length - 1;
      else selectedRowIndex = Math.max(0, selectedRowIndex - 1);

      const tx = pagedTxs[selectedRowIndex];
      if (tx) {
        document.getElementById(`tx-row-${tx.id}`)?.scrollIntoView({ block: 'nearest' });
      }
    } else if (e.key === 'Enter' || e.key === 'x') {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        expandedId = expandedId === tx.id ? null : tx.id;
      }
    } else if (e.key === 'e') {
      if (selectedRowIndex !== null && pagedTxs[selectedRowIndex]) {
        e.preventDefault();
        const tx = pagedTxs[selectedRowIndex];
        editing = editing?.id === tx.id ? null : tx;
        showNew = false;
        expandedId = null;
      }
    } else if (e.key === 'Escape') {
      selectedRowIndex = null;
      expandedId = null;
      if (!editing && !showNew) {
        // already closed
      }
    }
  }

  async function handleSave(tx: Transaction) {
    await ledger.upsertTransaction(tx);
    editing = null;
    showNew = false;
  }
  async function handleDelete(id: string) {
    await ledger.deleteTransaction(id);
    editing = null;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<PageLayout title={i18n.t.journal}>
  {#snippet actions()}
    <Button
      variant="primary"
      onclick={() => {
        showNew = !showNew;
        if (showNew) {
          editing = null;
          expandedId = null;
          // When creating new, scroll to top
          const scroller = document.querySelector('.overflow-y-auto');
          if (scroller) scroller.scrollTop = 0;
        }
      }}
    >
      <span class="inline-flex items-center gap-1.5 whitespace-nowrap">
        <Icon name="plus" size={12} />
        {i18n.t.newTransaction}
      </span>
    </Button>
  {/snippet}

  {#if ledger.loading}
    <Splash variant="inline" label={i18n.t.loadingFinnca} />
  {:else if ledger.error}
    <ErrorState message={ledger.error} onRetry={() => ledger.load()} />
  {:else}
    <Card padding={false} class="min-h-0 flex-1">
      <!-- Filter toolbar -->
      <div class="border-line flex shrink-0 flex-wrap items-center gap-2 border-b px-3 py-2.5">
        <SearchBar
          bind:value={q}
          oninput={resetTxPage}
          onclear={resetTxPage}
          placeholder={i18n.t.searchPlaceholder}
          class="max-w-none min-w-36 flex-1"
        />
        <FilterMenu
          bind:open={accFilterOpen}
          label={accFilterLabel}
          active={accFilter !== ''}
          count={accFilter !== '' ? 1 : 0}
          onReset={() => {
            accFilter = '';
            resetTxPage();
          }}
          resetLabel={i18n.t.reset}
          resetDisabled={accFilter === ''}
          panelClass="w-72"
        >
          <FilterSection title={i18n.t.filterAccountTitle} layout="list">
            <FilterOption
              label={`${i18n.t.allTxs} — ${i18n.t.allAccountsFilter}`}
              count={baseTxs.length}
              selected={accFilter === ''}
              check={false}
              onclick={() => {
                accFilter = '';
                resetTxPage();
                accFilterOpen = false;
              }}
            />
            {#each leafAccounts as acc (acc.id)}
              <FilterOption
                label={`${acc.code} ${acc.name}`}
                count={accTxCounts[acc.id] ?? 0}
                selected={accFilter === acc.id}
                onclick={() => {
                  accFilter = acc.id;
                  resetTxPage();
                  accFilterOpen = false;
                }}
              />
            {/each}
          </FilterSection>
        </FilterMenu>
        <DateRangeDropdown bind:from bind:to onChange={resetTxPage} size="md" />
        <span class="text-text-dim font-proto text-smaller shrink-0 px-1">
          {filtered.length}/{ledger.transactions.length}
          {i18n.t.txUnit}
        </span>
        {#if q || from || to || accFilter}
          <Button
            variant="ghost"
            size="sm"
            onclick={() => {
              q = '';
              from = '';
              to = '';
              accFilter = '';
              resetTxPage();
            }}
          >
            {i18n.t.reset}
          </Button>
        {/if}
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto px-3 py-2">
        {#if filtered.length === 0}
          <div class="flex h-full items-center justify-center p-8">
            <p class="text-text-muted text-small">
              {ledger.transactions.length === 0 ? i18n.t.noTxRecorded : i18n.t.noTxMatchFilter}
            </p>
          </div>
        {:else}
          <table class="text-small w-full border-collapse">
            <thead class="bg-bg-card sticky top-0 z-10">
              <tr class="border-line text-text-base border-b">
                <th class="label-xs w-28 py-2 pl-2 text-left font-normal">{i18n.t.date}</th>
                <th class="label-xs px-3 py-2 text-left font-normal">{i18n.t.description}</th>
                <th class="label-xs w-36 px-3 py-2 text-right font-normal">{i18n.t.amount}</th>
                <th class="label-xs w-20 px-3 py-2 text-center font-normal">{i18n.t.status}</th>
                <th class="w-16 py-2 pr-2"></th>
              </tr>
            </thead>
            <tbody class="divide-line/40 divide-y">
              {#each pagedTxs as tx, index (tx.id)}
                {@const imb = transactionImbalance(tx)}
                {@const total = tx.splits
                  .filter((s) => s.amount > 0)
                  .reduce((a, c) => a + c.amount, 0)}
                {@const isEditing = editing?.id === tx.id}
                {@const expanded = expandedId === tx.id}
                {@const isSelected = selectedRowIndex === index}

                <tr
                  id="tx-row-{tx.id}"
                  class="cursor-pointer transition-colors {isSelected
                    ? 'bg-bg-row-active outline-teal outline outline-1 -outline-offset-1'
                    : ''} {isEditing ? 'bg-bg-row-active' : 'hover:bg-bg-row-active'}"
                  onclick={() => {
                    expandedId = expanded ? null : tx.id;
                    selectedRowIndex = index;
                  }}
                >
                  <td class="text-text-base font-proto py-2 pl-2 whitespace-nowrap">{tx.date}</td>
                  <td class="text-text-strong px-3 py-2">
                    <div class="flex items-center gap-2">
                      <span class="truncate">{tx.description}</span>
                      {#if tx.num}
                        <span
                          class="bg-bg-app border-line text-text-muted font-proto text-smaller border px-1"
                        >
                          {tx.num}
                        </span>
                      {/if}
                    </div>
                    {#if tx.notes}
                      <p class="text-text-muted text-smaller mt-0.5">
                        {tx.notes}
                      </p>
                    {/if}
                  </td>
                  <td class="text-text-base font-proto px-3 py-2 text-right whitespace-nowrap">
                    {tx.currency === 'USD' ? formatUSD(total) : formatIDR(total)}
                  </td>
                  <td class="px-3 py-2 text-center">
                    {#if imb !== 0}
                      <span class="badge-err text-smaller px-1.5 py-0.5">{i18n.t.badgeImbal}</span>
                    {:else}
                      <span class="badge-ok text-smaller px-1.5 py-0.5">{i18n.t.badgeOk}</span>
                    {/if}
                  </td>
                  <td class="py-2 pr-2 text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={(e) => {
                        e.stopPropagation();
                        editing = editing?.id === tx.id ? null : tx;
                        showNew = false;
                        expandedId = null;
                      }}
                    >
                      {i18n.t.edit}
                    </Button>
                  </td>
                </tr>

                {#if expanded && !isEditing}
                  <tr class="bg-bg-app">
                    <td colspan="5" class="p-3">
                      <table class="sharp-table">
                        <thead>
                          <tr class="border-line bg-bg-card text-text-muted border-b">
                            <th class="label-xs px-3 py-1.5 text-left font-normal"
                              >{i18n.t.account}</th
                            >
                            <th class="label-xs px-3 py-1.5 text-right font-normal"
                              >{i18n.t.debit}</th
                            >
                            <th class="label-xs px-3 py-1.5 text-right font-normal"
                              >{i18n.t.credit}</th
                            >
                            <th class="label-xs px-3 py-1.5 text-left font-normal">{i18n.t.note}</th
                            >
                          </tr>
                        </thead>
                        <tbody class="divide-line/40 divide-y">
                          {#each tx.splits as sp (sp.id)}
                            {@const acc = ledger.accountsById.get(sp.accountId)}
                            <tr>
                              <td class="text-text-base px-3 py-1.5">
                                <span class="text-text-muted font-proto">{acc?.code ?? '?'}</span>
                                <span class="ml-1">{acc?.name ?? sp.accountId.slice(0, 8)}</span>
                              </td>
                              <td
                                class="font-proto px-3 py-1.5 text-right {sp.amount > 0
                                  ? 'text-income'
                                  : 'text-text-muted'}"
                              >
                                {sp.amount > 0
                                  ? tx.currency === 'USD'
                                    ? formatUSD(sp.amount)
                                    : formatIDR(sp.amount)
                                  : '—'}
                              </td>
                              <td
                                class="font-proto px-3 py-1.5 text-right {sp.amount < 0
                                  ? 'text-text-base'
                                  : 'text-text-muted'}"
                              >
                                {sp.amount < 0
                                  ? tx.currency === 'USD'
                                    ? formatUSD(-sp.amount)
                                    : formatIDR(-sp.amount)
                                  : '—'}
                              </td>
                              <td class="text-text-muted px-3 py-1.5">{sp.memo ?? ''}</td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </td>
                  </tr>
                {/if}

                {#if isEditing}
                  <tr class="bg-bg-app">
                    <td colspan="5" class="p-3">
                      <TransactionEditor
                        tx={editing}
                        onSave={handleSave}
                        onCancel={() => (editing = null)}
                        onDelete={handleDelete}
                      />
                    </td>
                  </tr>
                {/if}
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

      <div
        class="border-line/40 flex shrink-0 items-center justify-between gap-2 border-t px-3 pt-1.5"
      >
        <div class="flex items-center gap-4">
          <span>
            {i18n.t.totalDebit}:
            <span class="text-income ml-1 font-bold">
              {formatIDR(idrDebit)}{usdDebit > 0 ? ` + ${formatUSD(usdDebit)}` : ''}
            </span>
          </span>
          <span class="text-line">|</span>
          <span>
            {i18n.t.totalCredit}:
            <span class="text-text-base ml-1 font-bold">
              {formatIDR(idrCredit)}{usdCredit > 0 ? ` + ${formatUSD(usdCredit)}` : ''}
            </span>
          </span>
        </div>

        <Pagination
          bind:currentPage={txPage}
          totalPages={totalTxPages}
          totalItems={filtered.length}
          itemLabel={i18n.t.txUnit}
        />
      </div>
    </Card>
  {/if}

  <ModalShell
    bind:open={showNew}
    title={i18n.t.newTransactionTitle}
    maxWidth="max-w-5xl"
    onClose={() => (showNew = false)}
  >
    <TransactionEditor tx={null} onSave={handleSave} onCancel={() => (showNew = false)} />
  </ModalShell>
</PageLayout>
