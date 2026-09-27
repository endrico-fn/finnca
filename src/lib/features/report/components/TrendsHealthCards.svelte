<script lang="ts">
  import { formatIDR } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Card, ProgressBar } from '$lib/components/ui';
  import type { Account } from '$lib/core/ipc/bindings';

  let {
    topAssetAccounts = [],
    debtRatio,
    debtStatus,
    runwayMonths,
  }: {
    topAssetAccounts: Array<{ account: Account; balIdr: number; percent: string }>;
    debtRatio: string;
    debtStatus: string;
    runwayMonths: string;
  } = $props();

  const numRatio = $derived(Number(debtRatio) || 0);
</script>

<div class="grid grid-cols-12 gap-2 pb-2">
  <Card title={i18n.t.topHoldings} class="col-span-12 lg:col-span-7">
    {#snippet header()}
      <span class="text-text-dim font-proto text-smaller leading-none">
        {i18n.t.trendsTop5}
      </span>
    {/snippet}

    <div class="space-y-2.5">
      {#each topAssetAccounts as item (item.account.id)}
        <div class="space-y-1">
          <div class="text-small flex items-center justify-between">
            <div class="flex items-center gap-1.5 truncate">
              <span class="text-text-dim font-proto text-smaller">{item.account.code}</span>
              <span class="text-text-strong truncate font-medium">{item.account.name}</span>
            </div>
            <div class="font-proto shrink-0 text-right">
              <span class="text-text-base">{formatIDR(item.balIdr)}</span>
              <span class="text-text-dim text-smaller ml-1">({item.percent}%)</span>
            </div>
          </div>
          <ProgressBar value={Number(item.percent)} tone="income" track="app" size="s" />
        </div>
      {:else}
        <div class="text-text-dim text-small p-4 text-center">{i18n.t.trendsNoHoldings}</div>
      {/each}
    </div>
  </Card>

  <Card title={i18n.t.financialHealthRatios} class="col-span-12 lg:col-span-5">
    {#snippet header()}
      <span class="text-text-dim font-proto text-smaller leading-none">
        {i18n.t.trendsMetricsLabel}
      </span>
    {/snippet}

    <div class="text-small space-y-3">
      <div class="border-line bg-bg-app space-y-1 border p-2.5">
        <div class="flex items-center justify-between">
          <span class="text-text-muted text-smaller font-proto">{i18n.t.debtToAssetRatio}</span>
          <span
            class="border-line text-smaller font-proto border px-1.5 py-0.5 font-bold uppercase
            {numRatio < 30
              ? 'text-income border-income/40'
              : numRatio < 60
                ? 'text-text-base'
                : 'text-expense border-expense/40'}"
          >
            {debtStatus}
          </span>
        </div>
        <div class="text-text-strong text-medium font-proto font-bold tabular-nums">
          {debtRatio}%
        </div>
        <div class="text-text-dim text-smaller font-proto">{i18n.t.debtRatioFormula}</div>
      </div>

      <div class="border-line bg-bg-app space-y-1 border p-2.5">
        <div class="flex items-center justify-between">
          <span class="text-text-muted text-smaller font-proto">{i18n.t.cashRunwayMonths}</span>
          <span class="text-text-dim font-proto text-smaller uppercase">{i18n.t.reserves}</span>
        </div>
        <div class="text-text-strong text-medium font-proto font-bold tabular-nums">
          {runwayMonths}
          <span class="text-text-muted text-small font-normal">{i18n.t.months}</span>
        </div>
        <div class="text-text-dim text-smaller font-proto">{i18n.t.runwayFormula}</div>
      </div>
    </div>
  </Card>
</div>
