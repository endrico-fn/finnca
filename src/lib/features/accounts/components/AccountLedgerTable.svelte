<script lang="ts">
  import type { Account, AccountRunningLedgerItem } from '$lib/core/ipc/bindings';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button } from '$lib/components/ui';

  export type LedgerEntryRow = AccountRunningLedgerItem;

  let {
    entries = [],
    accountCurrency = 'IDR',
    hasAnyEntries = false,
    isPlaceholder = false,
    onToggleReconcile,
    onEdit,
    onNewEntry,
    onAddSubAccount,
  }: {
    entries?: AccountRunningLedgerItem[];
    accountsById?: Map<string, Account>;
    accountCurrency?: string;
    hasAnyEntries?: boolean;
    isPlaceholder?: boolean;
    onToggleReconcile?: (row: AccountRunningLedgerItem) => void;
    onEdit?: (entryId: string) => void;
    onNewEntry?: () => void;
    onAddSubAccount?: () => void;
  } = $props();
</script>

<div class="min-h-0 flex-1 overflow-y-auto">
  <table class="sharp-table w-full border-x-0 border-t-0" spellcheck="false">
    <thead class="bg-bg-card sticky top-0 z-10">
      <tr class="border-line border-b">
        <th class="w-24 py-2 pl-3 whitespace-nowrap">
          {i18n.t.date}
        </th>
        <th class="px-3 py-2">
          {i18n.t.description}
        </th>
        <th class="w-36 px-3 py-2">
          {i18n.t.offsetLabel}
        </th>
        <th class="numeric w-32 px-3 py-2 text-right whitespace-nowrap">
          {i18n.t.debit}
        </th>
        <th class="numeric w-32 px-3 py-2 text-right whitespace-nowrap">
          {i18n.t.credit}
        </th>
        <th class="center w-14 px-1 py-2">
          {i18n.t.reconcileColShort}
        </th>
        <th class="numeric w-36 px-3 py-2 text-right whitespace-nowrap">
          {i18n.t.colBalance}
        </th>
        <th class="w-16 py-2 pr-3"></th>
      </tr>
    </thead>
    <tbody>
      {#each entries as e (e.posting_id)}
        <tr
          class="hover:bg-bg-row-active cursor-pointer transition-colors"
          onclick={() => onEdit?.(e.entry_id)}
        >
          <td
            class="font-proto text-text-muted text-smaller w-24 py-2 pl-3 whitespace-nowrap tabular-nums"
          >
            {e.date}
          </td>
          <td class="px-3 py-2">
            <div class="flex items-center gap-1.5">
              <span class="font-aux text-text-strong text-small truncate normal-case"
                >{e.description}</span
              >
            </div>
            {#if e.memo}
              <p class="font-aux text-text-muted text-smaller mt-0.5 leading-tight normal-case">
                {e.memo}
              </p>
            {/if}
          </td>
          <td
            class="font-proto text-text-dim text-smaller w-36 max-w-36 truncate px-3 py-2 whitespace-nowrap"
          >
            {e.offset_account || '—'}
          </td>
          <td
            class="numeric font-proto text-smaller w-32 px-3 py-2 text-right whitespace-nowrap tabular-nums {e.amount >
            0
              ? 'text-income font-bold'
              : 'text-text-dim'}"
          >
            {e.amount > 0 ? formatMinorToDisplay(e.amount, accountCurrency) : '—'}
          </td>
          <td
            class="numeric font-proto text-smaller w-32 px-3 py-2 text-right whitespace-nowrap tabular-nums {e.amount <
            0
              ? 'text-text-strong font-bold'
              : 'text-text-dim'}"
          >
            {e.amount < 0 ? formatMinorToDisplay(-e.amount, accountCurrency) : '—'}
          </td>
          <td class="center w-14 px-1 py-2">
            <button
              type="button"
              onclick={(ev) => {
                ev.stopPropagation();
                onToggleReconcile?.(e);
              }}
              title={i18n.t.reconcileCycleHint}
              class="font-proto text-smaller inline-flex items-center justify-center border px-1.5 py-0.5 leading-none font-bold tracking-wider uppercase transition-colors {e.reconcile ===
              'y'
                ? 'badge-ok'
                : e.reconcile === 'c'
                  ? 'badge-warn'
                  : 'border-line bg-bg-app text-text-muted hover:border-teal/40'}"
            >
              {e.reconcile}
            </button>
          </td>
          <td
            class="numeric font-proto text-smaller text-text-strong w-36 px-3 py-2 text-right font-medium whitespace-nowrap tabular-nums"
          >
            {formatMinorToDisplay(e.running_balance, accountCurrency)}
          </td>
          <td class="py-2 pr-3 text-right">
            <div class="flex items-center justify-end gap-1">
              <Button
                variant="ghost"
                size="sm"
                onclick={(ev) => {
                  ev.stopPropagation();
                  onEdit?.(e.entry_id);
                }}
              >
                {i18n.t.edit}
              </Button>
            </div>
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="8" class="text-text-muted font-aux text-small py-12 text-center">
            {#if isPlaceholder}
              <p>{i18n.t.placeholderAccountNotice}</p>
              {#if onAddSubAccount}
                <div class="mt-3 flex justify-center gap-2">
                  <Button variant="primary" size="sm" onclick={onAddSubAccount}>
                    {i18n.t.addSubAccountBtn}
                  </Button>
                  <Button variant="ghost" size="sm" href="/app/accounts">
                    {i18n.t.chartOfAccounts} →
                  </Button>
                </div>
              {/if}
            {:else}
              <p>
                {#if !hasAnyEntries}
                  {i18n.t.noTxForAccount}
                {:else}
                  {i18n.t.noEntriesMatchFilter}
                {/if}
              </p>
              {#if !hasAnyEntries && onNewEntry}
                <div class="mt-3 flex justify-center gap-2">
                  <Button variant="primary" size="sm" onclick={onNewEntry}>
                    {i18n.t.newEntryBtn}
                  </Button>
                  <Button variant="ghost" size="sm" href="/app/journal">
                    {i18n.t.journal} →
                  </Button>
                </div>
              {/if}
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
