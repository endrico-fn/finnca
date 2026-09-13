<script lang="ts">
  import { Card } from '$lib/components/ui';
  import { i18n } from '$lib/i18n.svelte';

  let {
    liabilities,
    debtRatio,
    netWorth,
  }: {
    liabilities: string;
    debtRatio: string;
    netWorth: string;
  } = $props();

  const ratioNum = $derived(parseFloat(debtRatio) || 0);
  const isHealthy = $derived(ratioNum < 30);
</script>

<Card title={i18n.t.liabilitiesAndDebt} class="h-full">
  {#snippet header()}
    <span
      class="font-proto text-smaller border px-1.5 py-0.5 leading-none tracking-wider {isHealthy
        ? 'border-income/30 text-income bg-income/10'
        : 'border-expense/30 text-expense bg-expense/10'}"
    >
      {isHealthy ? i18n.t.debtStatusHealthy : i18n.t.debtStatusCaution}
    </span>
  {/snippet}

  <div class="mt-1 flex min-h-0 flex-1 flex-col justify-between gap-2">
    <div>
      <p
        class="font-proto text-large leading-none font-bold tracking-tight tabular-nums {liabilities !==
          '0' && liabilities !== '0.00'
          ? 'text-expense'
          : 'text-text-strong'}"
      >
        Rp {liabilities}
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
          Rp {netWorth}
        </dd>
      </div>
    </dl>
  </div>
</Card>
