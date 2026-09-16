<script lang="ts">
  import BalanceCard from './BalanceCard.svelte';
  import DebtCard from './DebtCard.svelte';
  import HealthCard from './HealthCard.svelte';

  let {
    assets,
    usdAssets,
    accountCount,
    liabilities,
    debtRatio,
    netWorth,
    unbalanced,
    uncategorized,
    pending,
    lastRecon,
    runwayMonths,
    quickRatio,
  }: {
    assets: number;
    usdAssets: number;
    accountCount: number;
    liabilities: number;
    debtRatio: string;
    netWorth: number;
    unbalanced: number;
    uncategorized: number;
    pending: number;
    lastRecon: string;
    runwayMonths: number;
    quickRatio: number;
  } = $props();
</script>

<div class="grid shrink-0 grid-cols-12 gap-2">
  <div class="col-span-4">
    <BalanceCard {assets} {usdAssets} {accountCount} />
  </div>
  <div class="col-span-4">
    <DebtCard {liabilities} {debtRatio} {netWorth} />
  </div>
  <div class="col-span-4">
    <HealthCard {unbalanced} {uncategorized} {pending} {lastRecon} {runwayMonths} {quickRatio} />
  </div>
</div>
