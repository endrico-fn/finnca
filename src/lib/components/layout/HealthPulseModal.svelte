<script lang="ts">
  import { ModalShell, Button, Badge } from '$lib/components/ui';
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatIDR } from '$lib/core/format/currency';
  import { diagnoseVaultHealthCmd, type VaultHealthReport } from '$lib/core/ipc/bindings';

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

  let healthReport = $state<VaultHealthReport | null>(null);
  let diagnosing = $state(false);

  async function runDiagnostics() {
    diagnosing = true;
    try {
      healthReport = await diagnoseVaultHealthCmd();
    } catch (e) {
      healthReport = {
        is_healthy: false,
        sqlite_integrity: String(e),
        foreign_keys_ok: false,
        unbalanced_entries_count: 0,
        orphan_postings_count: 0,
        placeholder_postings_count: 0,
        issues: [String(e)],
      };
    } finally {
      diagnosing = false;
    }
  }
</script>

{#if open && healthStats}
  <ModalShell bind:open title={i18n.t.ledgerHealthTitle} size="lg">
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

      <div class="border-line/60 border-t pt-3">
        <div class="mb-2 flex items-center justify-between">
          <span class="text-text-dim font-proto text-smaller font-bold uppercase">
            {i18n.t.vaultHealthDiagnosticsTitle}
          </span>
          <Button variant="ghost" size="sm" disabled={diagnosing} onclick={runDiagnostics}>
            {diagnosing ? '...' : i18n.t.runDiagnosticsBtn}
          </Button>
        </div>

        {#if healthReport}
          <div
            class="font-proto text-smaller border p-2.5 {healthReport.is_healthy
              ? 'border-income/40 bg-income/5'
              : 'border-expense/40 bg-expense/5'}"
          >
            <div class="mb-1.5 flex items-center justify-between">
              <span class="font-bold {healthReport.is_healthy ? 'text-income' : 'text-expense'}">
                {healthReport.is_healthy
                  ? i18n.t.vaultHealthHealthy
                  : i18n.t.vaultHealthIssuesFound}
              </span>
              <Badge size="s" tone={healthReport.is_healthy ? 'ok' : 'err'}>
                SQLite: {healthReport.sqlite_integrity}
              </Badge>
            </div>
            {#if healthReport.issues.length > 0}
              <ul class="text-expense text-smaller mt-2 list-inside list-disc space-y-1">
                {#each healthReport.issues as issue, idx (idx)}
                  <li class="truncate">{issue}</li>
                {/each}
              </ul>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </ModalShell>
{/if}
