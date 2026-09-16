<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { reportState } from '$lib/features/report/state/report.svelte';
  import { formatIDR, formatUSD } from '$lib/core/format/currency';
  import { EmptyState, KpiCard } from '$lib/components/ui';
  import { fxState } from '$lib/features/settings/state/settings.svelte';

  const fxReport = $derived(reportState.fxRevaluation);
  const items = $derived(fxReport?.items ?? []);
  const totalCurrentVal = $derived(fxReport?.total_current_value_idr ?? 0);
  const totalCostBasis = $derived(fxReport?.total_cost_basis_idr ?? 0);
  const totalUnrealizedGain = $derived(fxReport?.total_unrealized_gain_idr ?? 0);
  const currentRate = $derived(fxReport?.current_fx_rate || fxState.rate || 16000);
  const hasData = $derived(items.length > 0);
</script>

<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
  <div class="grid shrink-0 grid-cols-4 gap-2">
    <!-- Card 1: Unrealized FX Gain / Loss -->
    <KpiCard
      label={i18n.t.totalUnrealizedFx}
      badge={totalUnrealizedGain >= 0 ? i18n.t.fxGainBadge : i18n.t.fxLossBadge}
      badgeTone={!hasData ? 'neutral' : totalUnrealizedGain >= 0 ? 'ok' : 'err'}
      value={`${totalUnrealizedGain >= 0 ? '+' : ''}${formatIDR(totalUnrealizedGain)}`}
      valueClass={totalUnrealizedGain >= 0 ? 'text-income' : 'text-expense'}
      subValue={`${i18n.t.currentRate}: ${formatIDR(currentRate)}`}
    />

    <!-- Card 2: Current Valuation (IDR) -->
    <KpiCard
      label={i18n.t.colCurrentValue}
      value={formatIDR(totalCurrentVal)}
      valueClass="text-text-strong"
      subValue={`${items.length} ${i18n.t.account}`}
    />

    <!-- Card 3: Total Cost Basis (IDR) -->
    <KpiCard
      label={i18n.t.colCostBasis}
      value={formatIDR(totalCostBasis)}
      valueClass="text-text-muted"
      subValue={i18n.t.trendsBadgeTotal}
    />

    <!-- Card 4: Active Benchmark Rate -->
    <KpiCard
      label={i18n.t.activeFxRate}
      badge="USD/IDR"
      value={formatIDR(currentRate)}
      valueClass="text-text-base"
      subValue={i18n.t.fxRateDisplay.replace('{rate}', formatIDR(currentRate))}
    />
  </div>

  {#if !hasData}
    <div class="border-line bg-bg-card flex flex-1 items-center justify-center border p-3">
      <EmptyState title={i18n.t.noForeignAssets} hint={i18n.t.noForeignAssetsHint} icon="chart" />
    </div>
  {:else}
    <div class="border-line bg-bg-card border">
      <div
        class="border-line bg-line/20 font-proto text-text-muted text-smaller grid grid-cols-5 border-b px-3 py-1.5 tracking-widest uppercase"
      >
        <div class="col-span-1">{i18n.t.colAccount}</div>
        <div class="col-span-1 text-right">{i18n.t.colNativeBalance}</div>
        <div class="col-span-1 text-right">{i18n.t.colCostBasis}</div>
        <div class="col-span-1 text-right">{i18n.t.colCurrentValue}</div>
        <div class="col-span-1 text-right">{i18n.t.colUnrealizedGain}</div>
      </div>

      {#each items as rev (rev.account_id)}
        <div
          class="border-line font-proto hover:bg-bg-btn text-smaller grid grid-cols-5 items-center border-b px-3 py-1.5 tabular-nums transition-colors"
        >
          <div class="col-span-1">
            <div class="text-text-strong">{rev.name}</div>
            <div class="text-text-dim text-smaller mt-0.5">
              {rev.code} • {rev.currency}
            </div>
          </div>
          <div class="text-text-strong col-span-1 text-right font-medium">
            {rev.currency === 'USD' ? formatUSD(rev.native_balance) : rev.native_balance}
          </div>
          <div class="text-text-muted col-span-1 text-right">
            {formatIDR(rev.cost_basis_idr)}
          </div>
          <div class="text-text-base col-span-1 text-right">
            {formatIDR(rev.current_value_idr)}
          </div>
          <div
            class="col-span-1 text-right font-bold {rev.unrealized_gain_idr >= 0
              ? 'text-income'
              : 'text-expense'}"
          >
            {rev.unrealized_gain_idr >= 0 ? '+' : ''}{formatIDR(rev.unrealized_gain_idr)}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
