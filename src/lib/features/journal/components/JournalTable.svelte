<script lang="ts">
  import type { Transaction } from '../state/journalDraft.svelte';
  import type { Account } from '$lib/core/ipc/bindings';
  import { formatIDR, formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button, Badge } from '$lib/components/ui';
  import JournalEntryForm from './JournalEntryForm.svelte';
  import JournalSplitsDetail from './JournalSplitsDetail.svelte';

  const formatUSD = (val: number) => formatMinorToDisplay(val, 'USD');
  const transactionImbalance = (t: Transaction) => t.splits.reduce((sum, s) => sum + s.amount, 0);

  let {
    pagedTxs = [],
    accountsById,
    editing = null,
    expandedId = null,
    selectedRowIndex = null,
    totalCount = 0,
    onToggleExpand,
    onToggleEdit,
    onSave,
    onDelete,
    onSelectRow,
  }: {
    pagedTxs: Transaction[];
    accountsById: Map<string, Account>;
    editing?: Transaction | null;
    expandedId?: string | null;
    selectedRowIndex?: number | null;
    totalCount?: number;
    onToggleExpand: (id: string) => void;
    onToggleEdit: (tx: Transaction) => void;
    onSave: (tx: Transaction) => Promise<void>;
    onDelete: (id: string) => Promise<void>;
    onSelectRow: (index: number) => void;
  } = $props();
</script>

<div class="min-h-0 flex-1 overflow-y-auto">
  {#if pagedTxs.length === 0}
    <div class="flex h-full items-center justify-center p-8">
      <p class="text-text-muted font-aux text-small">
        {totalCount === 0 ? i18n.t.noTxRecorded : i18n.t.noTxMatchFilter}
      </p>
    </div>
  {:else}
    <table class="sharp-table">
      <thead class="sticky top-0 z-10">
        <tr>
          <th class="w-28 pl-3">{i18n.t.date}</th>
          <th class="px-3">{i18n.t.description}</th>
          <th class="numeric w-36 px-3">{i18n.t.amount}</th>
          <th class="center w-24 px-3">{i18n.t.status}</th>
          <th class="w-16 pr-3"></th>
        </tr>
      </thead>
      <tbody>
        {#each pagedTxs as tx, index (tx.id)}
          {@const imb = transactionImbalance(tx)}
          {@const total = tx.splits.filter((s) => s.amount > 0).reduce((a, c) => a + c.amount, 0)}
          {@const isEditing = editing?.id === tx.id}
          {@const expanded = expandedId === tx.id}
          {@const isSelected = selectedRowIndex === index}

          <tr
            id="tx-row-{tx.id}"
            class="cursor-pointer transition-colors {isSelected
              ? 'bg-bg-row-active border-l-2 border-teal'
              : ''} {isEditing ? 'bg-bg-row-active' : ''}"
            onclick={() => {
              onToggleExpand(tx.id);
              onSelectRow(index);
            }}
          >
            <td class="font-proto text-text-base py-2 pl-3 whitespace-nowrap tabular-nums">
              {tx.date}
            </td>
            <td class="px-3 py-2">
              <div class="flex items-center gap-2">
                <span class="font-aux text-text-strong truncate">{tx.description}</span>
                {#if tx.num}
                  <span
                    class="bg-bg-app border-line text-text-muted font-proto text-smaller border px-1"
                  >
                    {tx.num}
                  </span>
                {/if}
                {#if tx.splits.length > 2}
                  <span
                    class="bg-bg-card border-line text-text-dim font-proto text-[10px] border px-1 font-semibold uppercase"
                    title="{tx.splits.length} splits"
                  >
                    {tx.splits.length} SPLITS
                  </span>
                {/if}
              </div>
              {#if tx.notes}
                <p class="text-text-muted font-aux text-smaller mt-0.5 leading-tight">
                  {tx.notes}
                </p>
              {/if}
            </td>
            <td class="numeric font-proto text-text-base px-3 py-2 whitespace-nowrap">
              {tx.currency === 'USD' ? formatUSD(total) : formatIDR(total)}
            </td>
            <td class="center px-3 py-2">
              {#if imb !== 0}
                <Badge size="m" tone="err">{i18n.t.badgeImbal}</Badge>
              {:else}
                <Badge size="m" tone="ok">{i18n.t.badgeOk}</Badge>
              {/if}
            </td>
            <td class="py-2 pr-3 text-right">
              <Button
                variant="ghost"
                size="sm"
                onclick={(e) => {
                  e.stopPropagation();
                  onToggleEdit(tx);
                }}
              >
                {i18n.t.edit}
              </Button>
            </td>
          </tr>

          {#if expanded && !isEditing}
            <tr class="bg-bg-card/40 border-line border-b">
              <td colspan="5" class="p-3">
                <JournalSplitsDetail {tx} {accountsById} />
              </td>
            </tr>
          {/if}

          {#if isEditing}
            <tr class="bg-bg-card/60 border-line border-b">
              <td colspan="5" class="p-3">
                <svelte:boundary>
                  <JournalEntryForm
                    tx={editing}
                    {onSave}
                    onCancel={() => onToggleEdit(tx)}
                    {onDelete}
                  />
                  {#snippet failed()}
                    <div class="badge-err text-smaller px-1.5 py-2">
                      {i18n.t.quickTxLoadFailed}
                    </div>
                  {/snippet}
                </svelte:boundary>
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  {/if}
</div>
