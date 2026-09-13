<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import { ledger } from '$lib/accounting/store.svelte';
  import { calculateFxRevaluation, formatIDR, formatUSD } from '$lib/accounting/finance';
  import { EmptyState, Badge, Card } from '$lib/components/ui';

  const fxData = $derived(
    ledger.data ? calculateFxRevaluation(ledger.data) : { revaluations: [], totalUnrealizedGain: 0 }
  );
  const hasData = $derived(fxData.revaluations.length > 0);
</script>

<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
  <Card title={i18n.t.totalUnrealizedFx} badge={i18n.t.fxValuationBadge}>
    <div class="mt-1 flex items-start gap-4">
      <span
        class="font-proto text-largest font-bold tabular-nums {fxData.totalUnrealizedGain >= 0
          ? 'text-income'
          : 'text-expense'}"
      >
        {fxData.totalUnrealizedGain >= 0 ? '+' : ''}{formatIDR(fxData.totalUnrealizedGain)}
      </span>
      <Badge tone={fxData.totalUnrealizedGain >= 0 ? 'ok' : 'err'}>
        {fxData.totalUnrealizedGain >= 0 ? i18n.t.fxGainBadge : i18n.t.fxLossBadge}
      </Badge>
    </div>
    <div class="font-proto text-text-dim text-smaller mt-2 flex items-center gap-2">
      <span class="bg-line/30 border-line border px-1.5 py-0.5 leading-none"
        >{i18n.t.currentRate}</span
      >
      <span class="leading-none"
        >{i18n.t.fxRateDisplay.replace('{rate}', formatIDR(ledger.data?.fxRate || 15000))}</span
      >
    </div>
  </Card>

  {#if !hasData}
    <div class="border-line bg-bg-card flex flex-1 items-center justify-center border p-3">
      <EmptyState title={i18n.t.noForeignAssets} hint={i18n.t.noForeignAssetsHint} icon="chart" />
    </div>
  {:else}
    <div class="border-line bg-bg-card border">
      <div
        class="border-line bg-line/20 font-proto text-text-muted text-smaller grid grid-cols-5 border-b px-3 py-2 tracking-widest uppercase"
      >
        <div class="col-span-1">{i18n.t.colAccount}</div>
        <div class="col-span-1 text-right">{i18n.t.colNativeBalance}</div>
        <div class="col-span-1 text-right">{i18n.t.colCostBasis}</div>
        <div class="col-span-1 text-right">{i18n.t.colCurrentValue}</div>
        <div class="col-span-1 text-right">{i18n.t.colUnrealizedGain}</div>
      </div>

      {#each fxData.revaluations as rev (rev.accountId)}
        {@const acc = ledger.accounts.find((a) => a.id === rev.accountId)}
        <div
          class="border-line font-proto hover:bg-bg-btn text-small grid grid-cols-5 items-center border-b px-3 py-2.5 transition-colors"
        >
          <div class="col-span-1">
            <div class="text-text-strong">{acc?.name}</div>
            <div class="text-text-dim text-smaller mt-0.5">
              {acc?.code} • {acc?.currency}
            </div>
          </div>
          <div class="text-text-strong col-span-1 text-right font-medium">
            {acc?.currency === 'USD' ? formatUSD(rev.nativeBalance) : rev.nativeBalance}
          </div>
          <div class="text-text-muted col-span-1 text-right">
            {formatIDR(rev.costBasisIdr)}
          </div>
          <div class="text-text-base col-span-1 text-right">
            {formatIDR(rev.currentValueIdr)}
          </div>
          <div
            class="col-span-1 text-right font-bold {rev.unrealizedGainIdr >= 0
              ? 'text-income'
              : 'text-expense'}"
          >
            {rev.unrealizedGainIdr >= 0 ? '+' : ''}{formatIDR(rev.unrealizedGainIdr)}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
