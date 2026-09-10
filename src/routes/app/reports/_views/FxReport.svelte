<script lang="ts">
  import { i18n } from '$lib/i18n.svelte';
  import { ledger } from '$lib/accounting/store.svelte';
  import { calculateFxRevaluation, formatIDR, fromMinor, formatUSD } from '$lib/accounting/finance';
  import { EmptyState, Badge, Card } from '$lib/components/ui';

  const fxData = $derived(
    ledger.data ? calculateFxRevaluation(ledger.data) : { revaluations: [], totalUnrealizedGain: 0 }
  );
  const hasData = $derived(fxData.revaluations.length > 0);
</script>

<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
  <Card title={i18n.t.totalUnrealizedFx} badge="FX VALUATION">
    <div class="mt-1 flex items-start gap-4">
      <span
        class="font-proto text-3xl font-bold {fxData.totalUnrealizedGain >= 0
          ? 'text-income'
          : 'text-expense'}"
      >
        {fxData.totalUnrealizedGain >= 0 ? '+' : ''}{formatIDR(
          fromMinor('IDR', fxData.totalUnrealizedGain)
        )}
      </span>
      <Badge tone={fxData.totalUnrealizedGain >= 0 ? 'ok' : 'err'}>
        {fxData.totalUnrealizedGain >= 0 ? 'GAIN' : 'LOSS'}
      </Badge>
    </div>
    <div class="font-proto text-text-dim mt-2 flex items-center gap-2 text-[10px]">
      <span class="bg-line/30 border-line border px-1.5 py-0.5 leading-none">{i18n.t.currentRate}</span>
      <span class="leading-none">1 USD = {formatIDR(ledger.data?.fxRate || 15000)}</span>
    </div>
  </Card>

  {#if !hasData}
    <div class="border-line bg-bg-card flex flex-1 items-center justify-center border">
      <EmptyState title={i18n.t.noForeignAssets} hint={i18n.t.noForeignAssetsHint} icon="chart" />
    </div>
  {:else}
    <div class="border-line bg-bg-card border">
      <div
        class="border-line bg-line/20 font-proto text-text-muted grid grid-cols-5 border-b p-2 text-[10px] tracking-widest uppercase"
      >
        <div class="col-span-1">ACCOUNT</div>
        <div class="col-span-1 text-right">NATIVE BAL</div>
        <div class="col-span-1 text-right">COST BASIS (IDR)</div>
        <div class="col-span-1 text-right">CURRENT VAL (IDR)</div>
        <div class="col-span-1 text-right">UNREALIZED GAIN</div>
      </div>

      {#each fxData.revaluations as rev (rev.accountId)}
        {@const acc = ledger.accounts.find((a) => a.id === rev.accountId)}
        <div
          class="border-line font-proto hover:bg-bg-row-hover grid grid-cols-5 items-center border-b p-3 text-[11px] transition-colors"
        >
          <div class="col-span-1">
            <div class="text-text-strong">{acc?.name}</div>
            <div class="text-text-dim mt-0.5 text-[9px]">
              {acc?.code} • {acc?.currency}
            </div>
          </div>
          <div class="text-text-strong col-span-1 text-right font-medium">
            {acc?.currency === 'USD'
              ? formatUSD(fromMinor('USD', rev.nativeBalance))
              : rev.nativeBalance}
          </div>
          <div class="text-text-muted col-span-1 text-right">
            {formatIDR(fromMinor('IDR', rev.costBasisIdr))}
          </div>
          <div class="text-text-base col-span-1 text-right">
            {formatIDR(fromMinor('IDR', rev.currentValueIdr))}
          </div>
          <div
            class="col-span-1 text-right font-bold {rev.unrealizedGainIdr >= 0
              ? 'text-income'
              : 'text-expense'}"
          >
            {rev.unrealizedGainIdr >= 0 ? '+' : ''}{formatIDR(
              fromMinor('IDR', rev.unrealizedGainIdr)
            )}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
