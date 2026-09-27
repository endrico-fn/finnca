<script lang="ts">
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { JournalEntryView, Account } from '$lib/core/ipc/bindings';
  import { formatIDR, formatUSD } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button, Badge, Icon } from '$lib/components/ui';
  import { closingBooksState } from '$lib/core/state/ledgerLock.svelte';
  import { parseNoteTags } from '../state/journalNoteTags';
  import JournalSplitsDetail from './JournalSplitsDetail.svelte';

  const transactionImbalance = (t: JournalEntryView) =>
    t.postings.reduce((sum, s) => sum + s.amount, 0);

  function getDrCrPostings(tx: JournalEntryView, accountsMap: Map<string, Account>) {
    const drList = tx.postings.filter((p) => p.amount > 0);
    const crList = tx.postings.filter((p) => p.amount < 0);
    const drTotal = drList.reduce((acc, p) => acc + p.amount, 0);
    const crTotal = crList.reduce((acc, p) => acc + Math.abs(p.amount), 0);

    const getAccLabel = (p: (typeof drList)[0]) => {
      const acc = accountsMap.get(p.account_id);
      return p.account_code || acc?.code || p.account_name || acc?.name || '?';
    };

    return {
      isSimple: drList.length === 1 && crList.length === 1,
      drLabel:
        drList.length === 1 ? getAccLabel(drList[0]) : `${drList.length} ${i18n.t.splitsUnit}`,
      crLabel:
        crList.length === 1 ? getAccLabel(crList[0]) : `${crList.length} ${i18n.t.splitsUnit}`,
      drTotal,
      crTotal,
      isMulti: drList.length > 1 || crList.length > 1,
    };
  }

  let {
    pagedEntries = [],
    accountsById,
    editing = null,
    expandedId = null,
    selectedRowIndex = null,
    totalCount = 0,
    onToggleExpand,
    onToggleEdit,
    onSelectRow,
    onSettle,
  }: {
    pagedEntries?: JournalEntryView[];
    accountsById: Map<string, Account>;
    editing?: JournalEntryView | null;
    expandedId?: string | null;
    selectedRowIndex?: number | null;
    totalCount?: number;
    onToggleExpand: (id: string) => void;
    onToggleEdit: (tx: JournalEntryView) => void;
    onSelectRow: (index: number) => void;
    onSettle?: (tx: JournalEntryView) => void;
  } = $props();

  const entries = $derived(pagedEntries);

  const ROW_HEIGHT = 44;
  let scrollContainer = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let containerHeight = $state(600);

  const isVirtualized = $derived(entries.length > 30);
  const startIndex = $derived(
    isVirtualized ? Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - 5) : 0
  );
  const visibleCount = $derived(
    isVirtualized ? Math.ceil(containerHeight / ROW_HEIGHT) + 10 : entries.length
  );
  const endIndex = $derived(
    isVirtualized ? Math.min(entries.length, startIndex + visibleCount) : entries.length
  );
  const visibleEntries = $derived(
    isVirtualized ? entries.slice(startIndex, endIndex) : entries
  );
  const topPadding = $derived(isVirtualized ? startIndex * ROW_HEIGHT : 0);
  const bottomPadding = $derived(
    isVirtualized ? Math.max(0, (entries.length - endIndex) * ROW_HEIGHT) : 0
  );
</script>

<div
  bind:this={scrollContainer}
  bind:clientHeight={containerHeight}
  onscroll={(e) => {
    scrollTop = (e.currentTarget as HTMLElement).scrollTop;
  }}
  class="min-h-0 flex-1 overflow-y-auto"
>
  {#if entries.length === 0}
    <div class="flex h-full items-center justify-center p-8">
      <p class="text-text-muted font-aux text-small">
        {totalCount === 0 ? i18n.t.noTxRecorded : i18n.t.noTxMatchFilter}
      </p>
    </div>
  {:else}
    <table class="sharp-table">
      <thead class="sticky top-0 z-10">
        <tr>
          <th class="w-24 pl-3">{i18n.t.date}</th>
          <th class="w-20 px-2">{i18n.t.colRef}</th>
          <th class="px-2.5">{i18n.t.description}</th>
          <th class="w-36 px-2">{i18n.t.debit} {i18n.t.account}</th>
          <th class="numeric w-28 px-2">{i18n.t.debit}</th>
          <th class="w-36 px-2">{i18n.t.credit} {i18n.t.account}</th>
          <th class="numeric w-28 px-2">{i18n.t.credit}</th>
          <th class="center w-16 px-2">{i18n.t.status}</th>
          <th class="w-16 pr-3"></th>
        </tr>
      </thead>
      <tbody>
        {#snippet journalRow(tx: JournalEntryView, index: number)}
          {@const imb = transactionImbalance(tx)}
          {@const drcr = getDrCrPostings(tx, accountsById)}
          {@const isLocked = closingBooksState.isDateLocked(tx.date)}
          {@const isEditing = editing?.id === tx.id}
          {@const expanded = expandedId === tx.id}
          {@const isSelected = selectedRowIndex === index}
          {@const tags = parseNoteTags(tx)}
          {@const num = tags.ref}
          {@const isSettled = tags.settled}
          {@const cleanDue = tags.dueDate || null}
          {@const today = new Date().toISOString().slice(0, 10)}
          {@const isOverdue = cleanDue ? cleanDue < today && !isSettled : false}
          {@const displayNotes = tags.cleanNotes || null}

          <tr
            id="tx-row-{tx.id}"
            class="cursor-pointer transition-colors {isSelected
              ? 'bg-bg-row-active border-teal border-l-2'
              : ''} {isEditing ? 'bg-bg-row-active' : ''}"
            onclick={() => {
              onToggleExpand(tx.id);
              onSelectRow(index);
            }}
          >
            <td class="font-proto text-text-base py-2 pl-3 whitespace-nowrap tabular-nums">
              {tx.date}
            </td>
            <td class="font-proto text-text-muted text-smaller px-2 py-2 whitespace-nowrap">
              {#if num}
                <span class="bg-bg-app border-line text-text-dim border px-1">
                  {num}
                </span>
              {:else}
                —
              {/if}
            </td>
            <td class="px-2.5 py-2">
              <div class="flex items-center gap-1.5">
                <span class="font-aux text-text-strong truncate">{tx.description}</span>
                {#if cleanDue}
                  {#if isSettled}
                    <Badge size="s" tone="ok">{i18n.t.badgeSettled}</Badge>
                  {:else if isOverdue}
                    <Badge size="s" tone="err">{i18n.t.badgeOverdue}: {cleanDue}</Badge>
                  {:else}
                    <Badge size="s" tone="warn">{i18n.t.badgeDue}: {cleanDue}</Badge>
                  {/if}
                {/if}
                {#if drcr.isMulti}
                  <span
                    class="bg-bg-card border-line text-text-dim font-proto text-smaller border px-1 font-semibold uppercase"
                    title="{tx.postings.length} {i18n.t.splitsUnit.toLowerCase()}"
                  >
                    {tx.postings.length}
                    {i18n.t.splitsUnit}
                  </span>
                {/if}
              </div>
              {#if displayNotes}
                <p class="text-text-muted font-aux text-smaller mt-0.5 leading-tight">
                  {displayNotes}
                </p>
              {/if}
            </td>
            <td class="font-proto text-text-base text-smaller max-w-36 truncate px-2 py-2">
              <span class="text-text-strong font-medium">{drcr.drLabel}</span>
            </td>
            <td
              class="numeric font-proto text-income text-smaller px-2 py-2 font-bold whitespace-nowrap tabular-nums"
            >
              {tx.currency === 'USD' ? formatUSD(drcr.drTotal) : formatIDR(drcr.drTotal)}
            </td>
            <td class="font-proto text-text-base text-smaller max-w-36 truncate px-2 py-2">
              <span class="text-text-base">{drcr.crLabel}</span>
            </td>
            <td
              class="numeric font-proto text-text-strong text-smaller px-2 py-2 font-bold whitespace-nowrap tabular-nums"
            >
              {tx.currency === 'USD' ? formatUSD(drcr.crTotal) : formatIDR(drcr.crTotal)}
            </td>
            <td class="center px-2 py-2">
              {#if imb !== 0}
                <Badge size="s" tone="err">{i18n.t.badgeImbal}</Badge>
              {:else if isLocked}
                <span title={i18n.t.periodLockedNotice}>
                  <Badge size="s" tone="warn"
                    ><span class="inline-flex"><Icon name="lock" size={10} /></span>
                    {i18n.t.lockedPeriodBadge}</Badge
                  >
                </span>
              {:else}
                <Badge size="s" tone="ok">{i18n.t.badgeOk}</Badge>
              {/if}
            </td>
            <td class="py-2 pr-3 text-right">
              <div class="flex items-center justify-end gap-1">
                {#if cleanDue && !isSettled && onSettle && !isLocked}
                  <Button
                    variant="outline"
                    size="sm"
                    class="font-proto text-smaller border-success text-success hover:bg-success-bg px-1.5 py-0.5"
                    title={i18n.t.settleInvoiceTitle}
                    onclick={(e) => {
                      e.stopPropagation();
                      onSettle(tx);
                    }}
                  >
                    {i18n.t.settleInvoiceAction}
                  </Button>
                {/if}
                {#if isLocked}
                  <span
                    class="text-text-dim font-proto text-smaller inline-flex items-center gap-1 px-1.5 py-0.5 tracking-wider uppercase select-none"
                    title={i18n.t.periodLockedEditDisabled}
                  >
                    <Icon name="lock" size={10} />
                    {i18n.t.lockedPeriodBadge}
                  </span>
                {:else}
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
                {/if}
              </div>
            </td>
          </tr>

          {#if expanded && !isEditing}
            <tr class="bg-bg-card/40 border-line border-b">
              <td colspan="9" class="p-0">
                <div
                  transition:slide={{ duration: 120, easing: cubicOut }}
                  class="border-line/60 bg-bg-card/50 border-t p-3"
                >
                  <JournalSplitsDetail entry={tx} {accountsById} />
                </div>
              </td>
            </tr>
          {/if}
        {/snippet}

        {#if topPadding > 0}
          <tr style="height: {topPadding}px"><td colspan="9" class="p-0 border-none"></td></tr>
        {/if}
        {#each visibleEntries as tx, i (tx.id)}
          {@render journalRow(tx, startIndex + i)}
        {/each}
        {#if bottomPadding > 0}
          <tr style="height: {bottomPadding}px"><td colspan="9" class="p-0 border-none"></td></tr>
        {/if}
      </tbody>
    </table>
  {/if}
</div>
