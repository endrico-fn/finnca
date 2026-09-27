<script lang="ts">
  import { formatIDR, formatUSD } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Pagination, Badge } from '$lib/components/ui';

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
  class="border-line bg-bg-card font-proto text-smaller flex shrink-0 items-center justify-between gap-2 border-t px-3 py-2"
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
      <span class="text-text-strong ml-1 font-bold tabular-nums">
        {formatIDR(idrCredit)}{usdCredit > 0 ? ` + ${formatUSD(usdCredit)}` : ''}
      </span>
    </span>

    <div class="ml-1 hidden items-center sm:flex">
      {#if isBalanced}
        <Badge size="s" tone="ok">
          {i18n.t.badgeBalanced}
        </Badge>
      {:else}
        <Badge size="s" tone="err">
          {i18n.t.badgeImbalPrefix}
          {discrepancyIDR > 0 ? formatIDR(discrepancyIDR) : ''}{discrepancyUSD > 0
            ? ` ${formatUSD(discrepancyUSD)}`
            : ''}
        </Badge>
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
