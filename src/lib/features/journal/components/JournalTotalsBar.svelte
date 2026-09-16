<script lang="ts">
  import { formatIDR, formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Pagination } from '$lib/components/ui';

  const formatUSD = (val: number) => formatMinorToDisplay(val, 'USD');

  let {
    idrDebit = 0,
    idrCredit = 0,
    usdDebit = 0,
    usdCredit = 0,
    txPage = $bindable(1),
    totalTxPages = 1,
    filteredCount = 0,
  }: {
    idrDebit?: number;
    idrCredit?: number;
    usdDebit?: number;
    usdCredit?: number;
    txPage?: number;
    totalTxPages?: number;
    filteredCount?: number;
  } = $props();

  const isBalanced = $derived(idrDebit === idrCredit && usdDebit === usdCredit);
  const discrepancyIDR = $derived(Math.abs(idrDebit - idrCredit));
  const discrepancyUSD = $derived(Math.abs(usdDebit - usdCredit));
</script>

<div
  class="border-line/40 font-proto text-medium flex shrink-0 items-center justify-between gap-2 border-t px-3 py-1.5"
>
  <div class="flex items-center gap-3">
    <span>
      {i18n.t.totalDebit}:
      <span class="text-income ml-1 font-bold tabular-nums">
        {formatIDR(idrDebit)}{usdDebit > 0 ? ` + ${formatUSD(usdDebit)}` : ''}
      </span>
    </span>
    <span class="text-line">|</span>
    <span>
      {i18n.t.totalCredit}:
      <span class="text-text-base ml-1 font-bold tabular-nums">
        {formatIDR(idrCredit)}{usdCredit > 0 ? ` + ${formatUSD(usdCredit)}` : ''}
      </span>
    </span>

    <div class="ml-1 hidden items-center sm:flex">
      {#if isBalanced}
        <span
          class="border-income/30 bg-income/10 text-income font-proto text-smaller inline-flex items-center gap-1 border px-1.5 py-0.5 font-bold tracking-wider uppercase"
        >
          <span class="bg-income size-1.5"></span>
          BALANCED
        </span>
      {:else}
        <span
          class="border-expense/30 bg-expense/10 text-expense font-proto text-smaller inline-flex items-center gap-1 border px-1.5 py-0.5 font-bold tracking-wider uppercase"
        >
          <span class="bg-expense size-1.5 animate-pulse"></span>
          IMBAL: {discrepancyIDR > 0 ? formatIDR(discrepancyIDR) : ''}{discrepancyUSD > 0
            ? ` ${formatUSD(discrepancyUSD)}`
            : ''}
        </span>
      {/if}
    </div>
  </div>

  <Pagination
    bind:currentPage={txPage}
    totalPages={totalTxPages}
    totalItems={filteredCount}
    itemLabel={i18n.t.txUnit}
  />
</div>
