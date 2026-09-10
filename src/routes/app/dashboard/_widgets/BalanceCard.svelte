<script lang="ts">
  import { Card } from '$lib/components/ui';
  import { i18n } from '$lib/i18n.svelte';

  let {
    assets,
    usdAssets,
    accountCount = 0,
  }: {
    assets: string;
    usdAssets: string;
    accountCount?: number;
  } = $props();
</script>

<Card title={i18n.t.totalBalance} class="h-full">
  {#snippet header()}
    <span
      class="font-proto border-income/30 text-income bg-income/10 border px-1.5 py-0.5 text-[9px] leading-none tracking-wider"
    >
      IDR / USD
    </span>
  {/snippet}
  <div class="mt-1 flex h-full flex-col justify-between gap-2 overflow-hidden">
    <div>
      <p class="text-income font-proto text-[20px] leading-none font-bold tabular-nums">
        Rp {assets}
      </p>
    </div>

    <dl class="border-line/40 font-proto mt-auto flex flex-col gap-1 border-t pt-1.5 text-[11px]">
      <div class="flex items-center justify-between">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.usdEquivalent}
        </dt>
        <dd class="text-text-strong font-proto font-medium tabular-nums">
          $ {usdAssets}
        </dd>
      </div>
      <div class="flex items-center justify-between">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.totalAccounts}
        </dt>
        <dd class="text-text-dim font-proto tabular-nums">
          {accountCount}
        </dd>
      </div>
    </dl>
  </div>
</Card>
