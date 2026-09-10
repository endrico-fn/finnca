<script lang="ts">
  import { Card } from '$lib/components/ui';
  import { i18n } from '$lib/i18n.svelte';

  let {
    unbalanced,
    uncategorized,
    pending,
    lastRecon,
    runwayMonths = 0,
    quickRatio = 0,
  }: {
    unbalanced: number;
    uncategorized: number;
    pending: number;
    lastRecon: string;
    runwayMonths?: number;
    quickRatio?: number;
  } = $props();
</script>

<Card title={i18n.t.ledgerHealth} class="h-full">
  {#snippet header()}
    <span
      class="font-proto border px-1.5 py-0.5 text-[9px] leading-none tracking-wider {unbalanced ===
      0
        ? 'border-income/30 text-income bg-income/10'
        : 'border-expense/30 text-expense bg-expense/10'}"
    >
      {unbalanced === 0 ? 'OK' : `${unbalanced} ERR`}
    </span>
  {/snippet}

  <div class="mt-1 flex h-full flex-col justify-between gap-2 overflow-hidden">
    <dl class="font-proto flex flex-col gap-1 text-[11px]">
      <div class="flex items-center justify-between">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.balancedEntry}
        </dt>
        <dd
          class="{unbalanced === 0
            ? 'text-income font-bold'
            : 'text-expense font-bold'} font-proto tabular-nums"
        >
          {unbalanced === 0 ? 'YES' : `${unbalanced} ERR`}
        </dd>
      </div>
      <div class="flex items-center justify-between">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.uncategorized}
        </dt>
        <dd
          class="{uncategorized === 0
            ? 'text-text-dim'
            : 'text-warning font-bold'} font-proto tabular-nums"
        >
          {uncategorized}
        </dd>
      </div>
      <div class="flex items-center justify-between">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.pendingEntry}
        </dt>
        <dd
          class="{pending === 0 ? 'text-text-dim' : 'text-teal font-bold'} font-proto tabular-nums"
        >
          {pending}
        </dd>
      </div>
      <div class="flex items-center justify-between pb-1">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.lastReconciliation}
        </dt>
        <dd class="text-text-dim font-proto tabular-nums">{lastRecon}</dd>
      </div>
      <div class="border-line/30 mt-auto flex items-center justify-between border-t pt-2">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.cashRunwayTitle}
        </dt>
        <dd class="text-income font-proto text-[10px] font-bold tabular-nums">
          {runwayMonths >= 999
            ? i18n.t.cashRunwayInfinite
            : i18n.t.cashRunwayMonthsFormat.replace('{months}', String(runwayMonths))}
        </dd>
      </div>
      <div class="flex items-center justify-between">
        <dt class="text-text-icon font-proto text-[10px] tracking-wider uppercase">
          {i18n.t.quickRatioTitle}
        </dt>
        <dd
          class="font-proto text-[10px] font-bold tabular-nums {quickRatio >= 1.5
            ? 'text-income'
            : quickRatio >= 1.0
              ? 'text-warning'
              : 'text-expense'}"
        >
          {quickRatio >= 999 ? '∞' : `${quickRatio}x`}
          <span
            class="py-0.2 ml-1 border px-1 text-[8px] {quickRatio >= 1.5
              ? 'border-income/40 text-income'
              : quickRatio >= 1.0
                ? 'border-warning/40 text-warning'
                : 'border-expense/40 text-expense'}"
          >
            {quickRatio >= 1.5
              ? i18n.t.quickRatioHealthy
              : quickRatio >= 1.0
                ? i18n.t.quickRatioTight
                : i18n.t.quickRatioCritical}
          </span>
        </dd>
      </div>
    </dl>
  </div>
</Card>
