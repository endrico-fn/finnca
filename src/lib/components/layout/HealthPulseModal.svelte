<script lang="ts">
  import { ModalShell, Button } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatIDR } from '$lib/core/format/currency';

  export interface HealthStats {
    balanced: boolean;
    unbalancedTxCount: number;
    discrepancy: number;
    assets: number;
    liabilities: number;
    equity: number;
    netIncome: number;
    unrealizedFx?: number;
  }

  let {
    open = $bindable(false),
    healthStats,
    onFixInJournal = () => {},
  }: {
    open: boolean;
    healthStats: HealthStats;
    onFixInJournal?: () => void;
  } = $props();
</script>

{#if open && healthStats}
  <ModalShell bind:open title={i18n.t.ledgerHealthTitle} maxWidth="max-w-lg">
    <div class="space-y-4 p-4 select-none">
      <div
        class="flex items-center justify-between border p-3 {healthStats.balanced &&
        healthStats.unbalancedTxCount === 0
          ? 'border-income/40 bg-income/5'
          : 'border-expense/40 bg-expense/5'}"
      >
        <div class="flex items-center gap-2">
          <span
            class="size-2 {healthStats.balanced && healthStats.unbalancedTxCount === 0
              ? 'bg-income'
              : 'bg-expense animate-ping'}"
          ></span>
          <span
            class="font-proto text-small font-bold tracking-wider {healthStats.balanced &&
            healthStats.unbalancedTxCount === 0
              ? 'text-income'
              : 'text-expense'}"
          >
            {healthStats.balanced && healthStats.unbalancedTxCount === 0
              ? i18n.t.ledgerBalanced
              : i18n.t.ledgerImbalance}
          </span>
        </div>
        <span class="font-proto text-text-muted text-small">
          {i18n.t.ledgerFormula}
        </span>
      </div>

      <div class="font-proto text-small space-y-2">
        <div class="border-line flex items-start justify-between border-b py-1.5">
          <span class="text-text-muted">{i18n.t.ledgerAssetsLabel} (A)</span>
          <span class="text-text-strong font-proto tabular-nums"
            >{formatIDR(healthStats.assets)}</span
          >
        </div>
        <div class="border-line flex items-start justify-between border-b py-1.5">
          <span class="text-text-muted">{i18n.t.liabilities} (L)</span>
          <span class="text-text-strong font-proto tabular-nums"
            >{formatIDR(healthStats.liabilities)}</span
          >
        </div>
        <div class="border-line flex items-start justify-between border-b py-1.5">
          <span class="text-text-muted">{i18n.t.equity} (E)</span>
          <span class="text-text-strong font-proto tabular-nums"
            >{formatIDR(healthStats.equity)}</span
          >
        </div>
        <div class="border-line flex items-start justify-between border-b py-1.5">
          <span class="text-text-muted">{i18n.t.net} (I - X)</span>
          <span class="text-text-strong font-proto tabular-nums"
            >{formatIDR(healthStats.netIncome)}</span
          >
        </div>
        {#if healthStats.unrealizedFx}
          <div class="border-line flex items-start justify-between border-b py-1.5">
            <span class="text-text-muted">{i18n.t.unrealizedFxGainLoss}</span>
            <span class="text-text-strong font-proto tabular-nums"
              >{formatIDR(healthStats.unrealizedFx)}</span
            >
          </div>
        {/if}
        <div class="border-line bg-bg-card flex items-start justify-between border-b px-2 py-1.5">
          <span class="text-text-muted">{i18n.t.ledgerLiabEquityLabel}</span>
          <span class="text-text-strong font-proto font-bold tabular-nums">
            {formatIDR(
              healthStats.liabilities +
                healthStats.equity +
                healthStats.netIncome +
                (healthStats.unrealizedFx || 0)
            )}
          </span>
        </div>
        <div
          class="flex items-center justify-between px-2 py-1.5 {healthStats.discrepancy === 0
            ? 'text-income'
            : 'text-expense font-bold'}"
        >
          <span>{i18n.t.difference}</span>
          <span class="font-proto tabular-nums">{formatIDR(healthStats.discrepancy)}</span>
        </div>
      </div>

      {#if healthStats.unbalancedTxCount > 0 || !healthStats.balanced}
        <div class="pt-2">
          <Button variant="danger" onclick={onFixInJournal} class="w-full">
            {i18n.t.viewJournalFix}
          </Button>
        </div>
      {/if}
    </div>
  </ModalShell>
{/if}
