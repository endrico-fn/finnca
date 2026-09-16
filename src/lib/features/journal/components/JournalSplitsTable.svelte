<script lang="ts">
  import type { Split, Currency } from '../state/journalDraft.svelte';
  import type { Account } from '$lib/core/ipc/bindings';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button } from '$lib/components/ui';
  import JournalLineRow from './JournalLineRow.svelte';

  let {
    splits = $bindable(),
    currency,
    accounts,
    isBalanced,
    imbalance,
    debitTotal,
    creditTotal,
    onCollapseToSimple,
    onAddSplit,
    onRemoveSplit,
    onAmountChange,
    onAutoBalance,
  }: {
    splits: Split[];
    currency: Currency;
    accounts: Account[];
    isBalanced: boolean;
    imbalance: number;
    debitTotal: number;
    creditTotal: number;
    onCollapseToSimple: () => void;
    onAddSplit: () => void;
    onRemoveSplit: (id: string) => void;
    onAmountChange: (split: Split, field: 'debit' | 'credit', val: string) => void;
    onAutoBalance: () => void;
  } = $props();
</script>

<div class="mt-4">
  <div class="flex items-center justify-between pb-2">
    <div>
      <p class="label-xs">
        {splits.length}
        {i18n.t.splitsLabel}
      </p>
      <p class="text-text-muted font-proto text-smaller mt-0.5">
        {i18n.t.debitCreditHelp}
      </p>
    </div>
    <div class="flex shrink-0 items-center gap-1.5">
      {#if splits.length === 2}
        <Button
          variant="ghost"
          size="sm"
          onclick={onCollapseToSimple}
          class="shrink-0 whitespace-nowrap"
        >
          <span class="inline-flex items-center gap-1.5">
            <Icon name="chev-left" size={12} />
            {i18n.t.txSimpleView}
          </span>
        </Button>
      {/if}
      <Button variant="ghost" size="sm" onclick={onAddSplit} class="shrink-0 whitespace-nowrap">
        {i18n.t.txAddRow}
      </Button>
    </div>
  </div>

  <div class="border-line mt-2 overflow-x-auto border">
    <table class="font-proto text-smaller w-full tabular-nums">
      <thead class="bg-bg-app text-text-base">
        <tr>
          <th class="label-xs px-3 py-1.5 text-left font-normal whitespace-nowrap">
            {i18n.t.code} &amp; {i18n.t.name}
          </th>
          <th class="label-xs w-28 px-2 py-1.5 text-right font-normal whitespace-nowrap">
            {i18n.t.totalDebit} ({currency})
          </th>
          <th class="label-xs w-28 px-2 py-1.5 text-right font-normal whitespace-nowrap">
            {i18n.t.totalCredit} ({currency})
          </th>
          <th class="label-xs px-3 py-1.5 text-left font-normal">{i18n.t.note}</th>
          <th class="w-8"></th>
        </tr>
      </thead>
      <tbody class="divide-line/40 bg-bg-app divide-y">
        {#each splits as sp, idx (sp.id)}
          <JournalLineRow
            bind:split={splits[idx]}
            {currency}
            {accounts}
            {onAmountChange}
            onRemove={onRemoveSplit}
          />
        {/each}
      </tbody>
      <tfoot class="border-line bg-bg-app border-t">
        <tr>
          <td class="label-xs px-3 py-1 text-right">{i18n.t.totals}</td>
          <td
            class="font-proto text-small px-2 py-1 text-right font-bold whitespace-nowrap tabular-nums {debitTotal
              ? 'text-income'
              : 'text-text-muted'}"
          >
            {formatMinorGrouping(debitTotal, currency)}
          </td>
          <td
            class="font-proto text-small px-2 py-1 text-right font-bold whitespace-nowrap tabular-nums {creditTotal
              ? 'text-text-base'
              : 'text-text-muted'}"
          >
            {formatMinorGrouping(creditTotal, currency)}
          </td>
          <td
            colspan="2"
            class="font-proto text-smaller px-3 py-1 {isBalanced ? 'text-income' : 'text-warning'}"
          >
            {isBalanced
              ? i18n.t.balanced
              : `${i18n.t.imbalance}: ${formatMinorGrouping(Math.abs(imbalance), currency)}`}
            {#if !isBalanced && imbalance !== 0}
              <Button
                variant="tactical"
                size="sm"
                onclick={onAutoBalance}
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
