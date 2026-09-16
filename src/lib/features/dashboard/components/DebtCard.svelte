<script lang="ts">
  import { Badge, Card } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';

  let {
    liabilities,
    debtRatio,
    netWorth,
  }: {
    liabilities: number;
    debtRatio: string;
    netWorth: number;
  } = $props();

  const fmtLiabilities = $derived(formatMinorToDisplay(liabilities, 'IDR'));
  const fmtNetWorth = $derived(formatMinorToDisplay(netWorth, 'IDR'));
  const ratioNum = $derived(parseFloat(debtRatio) || 0);
  const isHealthy = $derived(ratioNum < 30);
</script>

<Card title={i18n.t.liabilitiesAndDebt} class="h-full">
  {#snippet header()}
    {#if !isHealthy}
      <Badge size="m" tone="warn">
        {i18n.t.debtStatusCaution}
      </Badge>
    {/if}
  {/snippet}

  <div class="mt-1 flex min-h-0 flex-1 flex-col justify-between gap-2">
    <div>
      <p
        class="font-proto text-medium leading-none font-bold tracking-tight tabular-nums {liabilities >
        0
          ? 'text-expense'
          : 'text-text-strong'}"
      >
        {fmtLiabilities}
      </p>
    </div>

    <dl class="border-line/40 font-proto text-small mt-auto flex flex-col gap-1 border-t pt-1.5">
      <div class="flex items-center justify-between">
        <dt class="text-text-base font-proto text-smaller tracking-wider uppercase">
          {i18n.t.debtRatio}
        </dt>
        <dd class="font-proto text-text-strong text-smaller font-medium tabular-nums">
          {debtRatio}%
        </dd>
      </div>
      <div class="flex items-center justify-between">
        <dt class="text-text-base font-proto text-smaller tracking-wider uppercase">
          {i18n.t.netWorth}
        </dt>
        <dd class="font-proto text-income text-smaller font-medium tabular-nums">
          {fmtNetWorth}
        </dd>
      </div>
    </dl>
  </div>
</Card>
