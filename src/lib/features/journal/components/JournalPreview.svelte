<script lang="ts">
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';

  export interface PreviewLine {
    label: string;
    amount: number;
  }

  let {
    currency,
    debits,
    creditLabel,
    creditAmount,
  }: {
    currency: string;
    debits: PreviewLine[];
    creditLabel: string;
    creditAmount: number;
  } = $props();
</script>

<div class="border-line/60 bg-bg-card font-proto text-smaller mt-2 border px-3 py-2 tabular-nums">
  <p class="text-text-muted mb-1.5 tracking-wider">{i18n.t.previewJournal}</p>
  {#each debits as line (line.label)}
    <div class="text-income flex justify-between">
      <span>{line.label}</span>
      <span>{formatMinorGrouping(line.amount, currency)}</span>
    </div>
  {/each}
  <div class="text-text-base border-line/40 mt-1 flex justify-between border-t pt-1">
    <span>{creditLabel}</span>
    <span>{formatMinorGrouping(creditAmount, currency)}</span>
  </div>
</div>
