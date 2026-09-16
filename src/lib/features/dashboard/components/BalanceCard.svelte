<script lang="ts">
  import { Badge, Card } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';

  let {
    assets,
    usdAssets,
    accountCount = 0,
  }: {
    assets: number;
    usdAssets: number;
    accountCount?: number;
  } = $props();

  const fmtIDR = $derived(formatMinorToDisplay(assets, 'IDR'));
  const fmtUSD = $derived(formatMinorToDisplay(usdAssets, 'USD'));
</script>

<Card title={i18n.t.totalBalance} class="h-full">
  {#snippet header()}
    <Badge size="m" tone="neutral">{i18n.t.currencyBadgeIdrUsd}</Badge>
  {/snippet}
  <div class="mt-1 flex min-h-0 flex-1 flex-col justify-between gap-2">
    <div>
      <p class="text-income font-proto text-medium leading-none font-bold tabular-nums">
        {fmtIDR}
      </p>
    </div>

    <dl class="border-line/40 font-proto text-small mt-auto flex flex-col gap-1 border-t pt-1.5">
      <div class="flex items-center justify-between">
        <dt class="text-text-base font-proto text-smaller tracking-wider uppercase">
          {i18n.t.usdEquivalent}
        </dt>
        <dd class="text-text-strong font-proto text-smaller font-medium tabular-nums">
          {fmtUSD}
        </dd>
      </div>
      <div class="flex items-center justify-between">
        <dt class="text-text-base font-proto text-smaller tracking-wider uppercase">
          {i18n.t.totalAccounts}
        </dt>
        <dd class="text-text-dim font-proto text-smaller tabular-nums">
          {accountCount}
        </dd>
      </div>
    </dl>
  </div>
</Card>
