<script lang="ts">
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';
  import { KpiCard, AnimatedCounter } from '$lib/components/ui';

  let {
    netWorth,
    assets,
    usdAssets,
    liabilities,
    debtRatio,
    thisMonthNet,
    savingsRate,
  }: {
    netWorth: number;
    assets: number;
    usdAssets: number;
    liabilities: number;
    debtRatio: string;
    thisMonthNet: number;
    savingsRate: string;
  } = $props();

  const fmtUsd = $derived(formatMinorToDisplay(usdAssets, 'USD'));
  const ratioNum = $derived.by(() => {
    const n = Number(String(debtRatio).replace(',', '.'));
    return Number.isFinite(n) ? n : 0;
  });
  const isHealthyDebt = $derived(ratioNum < 30);
</script>

<div class="grid shrink-0 grid-cols-12 gap-2">
  <div class="col-span-12 sm:col-span-6 lg:col-span-3">
    <KpiCard
      label={i18n.t.netWorth}
      badge={netWorth < 0 ? i18n.t.statusErr : undefined}
      badgeTone="err"
    >
      <AnimatedCounter
        value={netWorth}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {i18n.t.currencyBadgeIdrUsd}
      </span>
    </KpiCard>
  </div>

  <div class="col-span-12 sm:col-span-6 lg:col-span-3">
    <KpiCard label={i18n.t.totalAssetsLabel}>
      <AnimatedCounter
        value={assets}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {fmtUsd} USD
      </span>
    </KpiCard>
  </div>

  <div class="col-span-12 sm:col-span-6 lg:col-span-3">
    <KpiCard
      label={i18n.t.liabilitiesAndDebt}
      badge={!isHealthyDebt ? i18n.t.debtStatusCaution : undefined}
      badgeTone="warn"
    >
      <AnimatedCounter
        value={liabilities}
        currency="IDR"
        class="text-medium block leading-tight font-bold {liabilities > 0
          ? 'text-expense'
          : 'text-text-white'}"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {i18n.t.debtRatio}: {debtRatio}%
      </span>
    </KpiCard>
  </div>

  <div class="col-span-12 sm:col-span-6 lg:col-span-3">
    <KpiCard label="{i18n.t.thisMonth} {i18n.t.net}">
      <AnimatedCounter
        value={thisMonthNet}
        currency="IDR"
        class="text-medium block leading-tight font-bold {thisMonthNet >= 0
          ? 'text-income'
          : 'text-expense'}"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {i18n.t.savingsRate}: {savingsRate}%
      </span>
    </KpiCard>
  </div>
</div>
