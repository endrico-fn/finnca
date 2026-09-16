<script lang="ts">
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    currency = 'IDR',
    periodStats,
    filteredCount = 0,
    totalCount = 0,
  }: {
    currency?: string;
    periodStats: {
      debit: number;
      credit: number;
      net: number;
      cleared: number;
      count: number;
    };
    filteredCount?: number;
    totalCount?: number;
  } = $props();
</script>

<div
  class="font-proto border-line text-smaller flex shrink-0 flex-wrap items-center gap-4 border-t pt-2"
>
  <span class="text-text-muted">
    {i18n.t.debitLabel}
    <strong class="text-income ml-1 tabular-nums">
      {formatMinorToDisplay(periodStats.debit, currency)}
    </strong>
  </span>
  <span class="text-line">|</span>
  <span class="text-text-muted">
    {i18n.t.creditLabel}
    <strong class="text-text-base ml-1 tabular-nums">
      {formatMinorToDisplay(periodStats.credit, currency)}
    </strong>
  </span>
  <span class="text-line">|</span>
  <span class="text-text-muted">
    {i18n.t.netLabel}
    <strong class="{periodStats.net >= 0 ? 'text-income' : 'text-expense'} ml-1 tabular-nums">
      {formatMinorToDisplay(Math.abs(periodStats.net), currency)}
      {periodStats.net < 0 ? i18n.t.crSuffix : ''}
    </strong>
  </span>
  <span class="text-line">|</span>
  <span class="text-text-muted">
    {i18n.t.clearedLabel}
    <strong class="text-teal ml-1 tabular-nums">
      {periodStats.cleared}/{periodStats.count}
    </strong>
    {i18n.t.reconcileCountSuffix}
  </span>
  {#if filteredCount !== totalCount}
    <span class="text-text-dim text-smaller ml-auto">
      {filteredCount}/{totalCount}
      {i18n.t.shownSuffix}
    </span>
  {/if}
</div>
