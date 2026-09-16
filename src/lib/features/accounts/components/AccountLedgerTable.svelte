<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import type { Transaction, Split } from '$lib/core/types';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button } from '$lib/components/ui';

  export interface LedgerEntryRow {
    tx: Transaction;
    split: Split;
    running: number;
  }

  let {
    entries = [],
    accountsById,
    accountCurrency = 'IDR',
    hasAnyEntries = false,
    isPlaceholder = false,
    onToggleReconcile,
    onEdit,
    onNewEntry,
    onAddSubAccount,
  }: {
    entries?: LedgerEntryRow[];
    accountsById: Map<string, Account>;
    accountCurrency?: string;
    hasAnyEntries?: boolean;
    isPlaceholder?: boolean;
    onToggleReconcile?: (row: LedgerEntryRow) => void;
    onEdit?: (tx: Transaction) => void;
    onNewEntry?: () => void;
    onAddSubAccount?: () => void;
  } = $props();
</script>

<div class="min-h-0 flex-1 overflow-y-auto px-3 py-1">
  {#if entries.length === 0}
    <div class="flex h-full items-center justify-center p-8">
      <div class="text-center">
        {#if isPlaceholder}
          <p class="text-text-muted text-small">
            {i18n.t.placeholderAccountNotice}
          </p>
          <div class="mt-3 flex justify-center gap-2">
            {#if onAddSubAccount}
              <Button variant="primary" size="sm" onclick={onAddSubAccount}>
                {i18n.t.addSubAccountBtn}
              </Button>
            {/if}
            <Button variant="ghost" size="sm" href="/app/accounts">
              {i18n.t.chartOfAccounts} →
            </Button>
          </div>
        {:else}
          <p class="text-text-muted text-small">
            {#if !hasAnyEntries}
              {i18n.t.noTxForAccount}
            {:else}
              {i18n.t.noEntriesMatchFilter}
            {/if}
          </p>
          <div class="mt-3 flex justify-center gap-2">
            <Button variant="primary" size="sm" onclick={onNewEntry}>
              {i18n.t.newEntryBtn}
            </Button>
            <Button variant="ghost" size="sm" href="/app/journal">
              {i18n.t.journal} →
            </Button>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <table class="font-proto text-smaller w-full border-collapse">
      <thead class="bg-bg-card sticky top-0 z-10">
        <tr class="border-line text-text-base border-b">
          <th class="label-xs w-24 py-1.5 pl-2 text-left font-normal whitespace-nowrap">
            {i18n.t.date}
          </th>
          <th class="label-xs px-3 py-1.5 text-left font-normal">
            {i18n.t.description}
          </th>
          <th class="label-xs px-3 py-1.5 text-left font-normal">
            {i18n.t.offsetLabel}
          </th>
          <th class="label-xs w-24 px-3 py-1.5 text-right font-normal whitespace-nowrap">
            {i18n.t.debit}
          </th>
          <th class="label-xs w-24 px-3 py-1.5 text-right font-normal whitespace-nowrap">
            {i18n.t.credit}
          </th>
          <th class="label-xs w-12 px-1 py-1.5 text-center font-normal">
            {i18n.t.reconcileColShort}
          </th>
          <th class="label-xs w-28 py-1.5 pr-2 text-right font-normal whitespace-nowrap">
            {i18n.t.colBalance}
          </th>
          <th class="w-12 py-1.5 pr-2"></th>
        </tr>
      </thead>
      <tbody class="divide-line/40 divide-y">
        {#each entries as e (e.tx.id + e.split.id)}
          {@const other = e.tx.splits.find((s) => s.accountId !== e.split.accountId)}
          {@const otherAcc = other ? accountsById.get(other.accountId) : null}
          <tr class="hover:bg-bg-row-active transition-colors">
            <td class="text-text-base font-proto py-1 pl-2 whitespace-nowrap tabular-nums">
              {e.tx.date}
            </td>
            <td class="text-text-strong text-smaller px-3 py-1">
              <div class="flex items-center gap-1.5">
                <span class="truncate">{e.tx.description}</span>
                {#if e.tx.num}
                  <span
                    class="border-line bg-bg-app text-text-muted font-proto text-smaller border px-1"
                  >
                    {e.tx.num}
                  </span>
                {/if}
              </div>
              {#if e.split.memo}
                <p class="text-text-muted text-smaller mt-0.5">
                  {e.split.memo}
                </p>
              {/if}
            </td>
            <td class="text-text-muted text-smaller max-w-36 truncate px-3 py-1">
              {otherAcc ? `${otherAcc.code} ${otherAcc.name}` : '—'}
            </td>
            <td
              class="font-proto text-smaller px-3 py-1 text-right whitespace-nowrap tabular-nums {e
                .split.amount > 0
                ? 'text-income'
                : 'text-text-dim'}"
            >
              {e.split.amount > 0 ? formatMinorGrouping(e.split.amount, e.tx.currency) : '—'}
            </td>
            <td
              class="font-proto text-smaller px-3 py-1 text-right whitespace-nowrap tabular-nums {e
                .split.amount < 0
                ? 'text-text-strong'
                : 'text-text-dim'}"
            >
              {e.split.amount < 0 ? formatMinorGrouping(-e.split.amount, e.tx.currency) : '—'}
            </td>
            <td class="px-1 py-1 text-center">
              <button
                type="button"
                onclick={() => onToggleReconcile?.(e)}
                title={i18n.t.reconcileCycleHint}
                class="font-proto text-smaller inline-flex items-center justify-center border px-1.5 py-0.5 leading-none font-bold tracking-wider uppercase transition-colors {e
                  .split.reconcile === 'y'
                  ? 'badge-ok'
                  : e.split.reconcile === 'c'
                    ? 'badge-warn'
                    : 'border-line bg-bg-app text-text-muted hover:border-teal/40'}"
              >
                {e.split.reconcile}
              </button>
            </td>
            <td
              class="text-text-strong font-proto text-smaller py-1 pr-2 text-right font-medium whitespace-nowrap tabular-nums"
            >
              {formatMinorGrouping(e.running, accountCurrency)}
            </td>
            <td class="py-1 pr-2 text-right">
              <Button variant="ghost" size="sm" onclick={() => onEdit?.(e.tx)}>
                {i18n.t.edit}
              </Button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
