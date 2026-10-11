<script lang="ts">
  import { formatIDR } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { KpiCard } from '$lib/components/ui';
  import type { DailyDataPoint } from '../state/report.svelte';
  import type { TrendsMetric } from '../state/trendsChartUtils';

  let {
    bs,
    activePoint = null,
    deltaNominal,
    deltaPercent,
    isNewCapital,
    trendsMetric = $bindable(),
  }: {
    bs: {
      assets: number;
      liabilities: number;
      equity: number;
    };
    activePoint?: DailyDataPoint | null;
    deltaNominal: number;
    deltaPercent: number;
    isNewCapital: boolean;
    trendsMetric: TrendsMetric;
  } = $props();
</script>

<div class="grid shrink-0 grid-cols-4 gap-2">
  <KpiCard
    label={`[01] ${i18n.t.assetsTitle}`}
    badge={i18n.t.trendsBadgeTotal}
    value={formatIDR(bs.assets)}
    subValue={i18n.t.allAccountsDesc}
    valueClass="text-text-strong"
    selected={trendsMetric === 'assets'}
    onclick={() => (trendsMetric = 'assets')}
  />

  <KpiCard
    label={`[02] ${i18n.t.liabilitiesTitle}`}
    badge={i18n.t.trendsBadgeDebt}
    value={formatIDR(bs.liabilities)}
    subValue={i18n.t.liabilitiesDesc}
    valueClass="text-expense"
    selected={trendsMetric === 'liabilities'}
    onclick={() => (trendsMetric = 'liabilities')}
  />

  <KpiCard
    label={`[03] ${i18n.t.metricLiquidCash}`}
    badge={i18n.t.trendsBadgeLiquid}
    value={formatIDR(activePoint ? activePoint.liquidCash : 0)}
    subValue={i18n.t.liquidCashDesc}
    valueClass="text-text-strong"
    selected={trendsMetric === 'liquidCash'}
    onclick={() => (trendsMetric = 'liquidCash')}
  />

  <KpiCard
    label={`[04] ${i18n.t.metricNetWorth}`}
    badge={i18n.t.trendsBadgeNet}
    value={formatIDR(activePoint ? activePoint.netWorth : bs.equity)}
    subValue={`Δ ${deltaNominal >= 0 ? '+' : ''}${formatIDR(deltaNominal)} (${isNewCapital ? i18n.t.trendsNewBadge.replace(/[()]/g, '') : `${deltaPercent >= 0 ? '+' : ''}${deltaPercent.toFixed(2)}%`})`}
    valueClass="text-text-strong"
    selected={trendsMetric === 'netWorth'}
    onclick={() => (trendsMetric = 'netWorth')}
  />
</div>
