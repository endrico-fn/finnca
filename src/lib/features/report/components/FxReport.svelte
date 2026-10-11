<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { reportState } from '$lib/features/report/state/report.svelte';
  import { formatIDR, formatUSD } from '$lib/core/format/currency';
  import { EmptyState, KpiCard, AnimatedCounter, Card } from '$lib/components/ui';
  import { fxState } from '$lib/core/state/fx.svelte';

  const fxReport = $derived(reportState.fxRevaluation);
  const items = $derived(fxReport?.items ?? []);
  const totalCurrentVal = $derived(fxReport?.total_current_value_idr ?? 0);
  const totalCostBasis = $derived(fxReport?.total_cost_basis_idr ?? 0);
  const totalUnrealizedGain = $derived(fxReport?.total_unrealized_gain_idr ?? 0);
  const currentRate = $derived(fxReport?.current_fx_rate || fxState.rate);
  const hasData = $derived(items.length > 0);

  function formatGainSigned(n: number): string {
    const prefix = n >= 0 ? '+' : '';
    return `${prefix}${formatIDR(n)}`;
  }
</script>

<div class="flex min-h-0 flex-1 flex-col gap-3">
  <div class="grid shrink-0 grid-cols-2 gap-2 sm:grid-cols-4">
    <KpiCard
      label={i18n.t.totalUnrealizedFx}
      badge={totalUnrealizedGain >= 0 ? i18n.t.fxGainBadge : i18n.t.fxLossBadge}
      badgeTone={!hasData ? 'neutral' : totalUnrealizedGain >= 0 ? 'ok' : 'err'}
      subValue={`${i18n.t.currentRate}: ${formatIDR(currentRate)}`}
    >
      <AnimatedCounter
        value={totalUnrealizedGain}
        currency="IDR"
        formatFn={formatGainSigned}
        class="text-medium block leading-tight font-bold {totalUnrealizedGain >= 0
          ? 'text-income'
          : 'text-expense'}"
      />
    </KpiCard>

    <KpiCard
      label={i18n.t.colCurrentValue}
      subValue={`${items.length} ${i18n.t.accountsTitle.toLowerCase()}`}
    >
      <AnimatedCounter
        value={totalCurrentVal}
        currency="IDR"
        class="text-medium text-text-strong block leading-tight font-bold"
      />
    </KpiCard>

    <KpiCard label={i18n.t.colCostBasis} subValue={i18n.t.trendsBadgeTotal}>
      <AnimatedCounter
        value={totalCostBasis}
        currency="IDR"
        class="text-medium text-text-muted block leading-tight font-bold"
      />
    </KpiCard>

    <KpiCard
      label={i18n.t.activeFxRate}
      badge="USD/IDR"
      value={formatIDR(currentRate)}
      valueClass="text-text-base"
      subValue={i18n.t.fxRateDisplay.replace('{rate}', formatIDR(currentRate))}
    />
  </div>

  {#if !hasData}
    <div class="border-line bg-bg-card flex flex-1 items-center justify-center border p-8">
      <EmptyState title={i18n.t.noForeignAssets} hint={i18n.t.noForeignAssetsHint} icon="chart" />
    </div>
  {:else}
    <Card
      padding={false}
      divided={false}
      title={i18n.t.fxRevalTab}
      count={items.length}
      class="flex min-h-0 flex-1 flex-col"
    >
      <div class="min-h-0 flex-1 overflow-y-auto">
        <table class="sharp-table w-full">
          <thead class="sticky top-0 z-10">
            <tr>
              <th class="pl-3">{i18n.t.colAccount}</th>
              <th class="numeric w-40 px-3">{i18n.t.colNativeBalance}</th>
              <th class="numeric w-40 px-3">{i18n.t.colCostBasis}</th>
              <th class="numeric w-40 px-3">{i18n.t.colCurrentValue}</th>
              <th class="numeric w-44 pr-3">{i18n.t.colUnrealizedGain}</th>
            </tr>
          </thead>
          <tbody>
            {#each items as rev (rev.account_id)}
              <tr>
                <td class="pl-3">
                  <div class="text-text-strong font-proto text-small font-medium">{rev.name}</div>
                  <div class="text-text-dim font-proto text-smaller">
                    {rev.code} • {rev.currency}
                  </div>
                </td>
                <td
                  class="numeric font-proto text-text-strong text-small px-3 whitespace-nowrap tabular-nums"
                >
                  {rev.currency === 'USD' ? formatUSD(rev.native_balance) : rev.native_balance}
                </td>
                <td
                  class="numeric font-proto text-text-muted text-small px-3 whitespace-nowrap tabular-nums"
                >
                  {formatIDR(rev.cost_basis_idr)}
                </td>
                <td
                  class="numeric font-proto text-text-base text-small px-3 font-medium whitespace-nowrap tabular-nums"
                >
                  {formatIDR(rev.current_value_idr)}
                </td>
                <td
                  class="numeric font-proto text-small pr-3 font-bold whitespace-nowrap tabular-nums {rev.unrealized_gain_idr >=
                  0
                    ? 'text-income'
                    : 'text-expense'}"
                >
                  {rev.unrealized_gain_idr >= 0 ? '+' : ''}{formatIDR(rev.unrealized_gain_idr)}
                </td>
              </tr>
            {/each}
          </tbody>
          <tfoot
            class="border-line bg-bg-card font-proto sticky bottom-0 z-10 border-t-2 font-bold"
          >
            <tr>
              <td class="text-smaller text-text-muted py-2 pl-3 tracking-wider uppercase">
                {i18n.t.totals}
              </td>
              <td class="numeric text-smaller text-text-dim w-40 px-3 py-2"> — </td>
              <td
                class="numeric text-smaller text-text-muted w-40 px-3 py-2 whitespace-nowrap tabular-nums"
              >
                {formatIDR(totalCostBasis)}
              </td>
              <td
                class="numeric text-smaller text-text-strong w-40 px-3 py-2 whitespace-nowrap tabular-nums"
              >
                {formatIDR(totalCurrentVal)}
              </td>
              <td
                class="numeric text-smaller w-44 py-2 pr-3 font-bold whitespace-nowrap tabular-nums {totalUnrealizedGain >=
                0
                  ? 'text-income'
                  : 'text-expense'}"
              >
                {formatGainSigned(totalUnrealizedGain)}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </Card>
  {/if}
</div>
