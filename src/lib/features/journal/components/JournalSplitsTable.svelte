<script lang="ts">
  import type { PostingInput, Account } from '$lib/core/ipc/bindings';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Button } from '$lib/components/ui';
  import JournalLineRow from './JournalLineRow.svelte';

  let {
    splits = $bindable(),
    currency = 'IDR',
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
    splits: PostingInput[];
    currency?: string;
    accounts: Account[];
    isBalanced: boolean;
    imbalance: number;
    debitTotal: number;
    creditTotal: number;
    onCollapseToSimple?: () => void;
    onAddSplit: () => void;
    onRemoveSplit: (id?: string | null) => void;
    onAmountChange: (split: PostingInput, field: 'debit' | 'credit', val: string) => void;
    onAutoBalance: () => void;
  } = $props();
</script>

<div class="space-y-2">
  <div class="flex items-center justify-between pb-1">
    <div>
      <p class="label-xs font-bold">
        {splits.length}
        {i18n.t.splitsLabel}
      </p>
      <p class="text-text-muted font-proto text-smaller">
        {i18n.t.debitCreditHelp}
      </p>
    </div>
    <div class="flex shrink-0 items-center gap-1.5">
      {#if onCollapseToSimple && splits.length === 2}
        <Button
          variant="ghost"
          size="sm"
          onclick={onCollapseToSimple}
          class="h-7 shrink-0 px-2 whitespace-nowrap"
        >
          <span class="font-proto text-smaller inline-flex items-center gap-1">
            <span>‹ {i18n.t.txSimpleView}</span>
          </span>
        </Button>
      {/if}
      {#if imbalance !== 0}
        <Button variant="tactical" size="sm" onclick={onAutoBalance} class="h-7 px-2">
          <span class="font-proto text-smaller inline-flex items-center gap-1">
            <span>{i18n.t.autoBalanceBtn}</span>
            <span class="border-line/80 text-text-dim py-0.2 text-smaller border px-1">Alt+A</span>
          </span>
        </Button>
      {/if}
      <Button variant="ghost" size="sm" onclick={onAddSplit} class="h-7 px-2 whitespace-nowrap">
        <span class="font-proto text-smaller inline-flex items-center gap-1">
          <span>{i18n.t.txAddRow}</span>
          <span class="border-line/80 text-text-dim py-0.2 text-smaller border px-1">Alt+N</span>
        </span>
      </Button>
    </div>
  </div>

  <div class="border-line border">
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
            {#if isBalanced}
              <span class="text-income font-bold">{i18n.t.balancedZero}</span>
            {:else}
              <span>{i18n.t.imbalance}: {formatMinorGrouping(Math.abs(imbalance), currency)}</span>
              {#if imbalance !== 0}
                <Button
                  variant="tactical"
                  size="sm"
                  onclick={onAutoBalance}
                  class="text-smaller ml-2 inline-flex h-6 items-center gap-1.5 py-0.5"
                >
                  <span>{i18n.t.autoBalanceBtn}</span>
                  <span class="border-line/80 text-text-dim text-smaller border px-1 py-0.5"
                    >Alt+A</span
                  >
                </Button>
              {/if}
            {/if}
          </td>
        </tr>
      </tfoot>
    </table>
  </div>
</div>
