<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import {
    balanceSheet,
    fromMinor,
    formatIDR,
    formatUSD,
    accountBalanceMinor,
    buildChildrenMap,
  } from '$lib/accounting/finance';
  import { todayString } from '$lib/accounting/core/date';
  import { generateGhostTransactions } from '$lib/accounting/features/planning';
  import { ReportCard, AccountRow } from '$lib/components/ui';

  let { asOf = '' }: { asOf?: string } = $props();

  const filteredVault = $derived.by(() => {
    if (!ledger.data) return null;
    let ghostTxs: import('$lib/accounting/types').Transaction[] = [];
    const today = todayString();
    if (asOf && asOf > today) {
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 1);
      const tomorrowStr = todayString(tomorrow);
      ghostTxs = generateGhostTransactions(ledger.data, tomorrowStr, asOf);
    }
    return asOf
      ? { ...ledger.data, transactions: [...ledger.data.transactions.filter(t => t.date <= asOf), ...ghostTxs] }
      : ledger.data;
  });

  const bs = $derived(
    filteredVault
      ? balanceSheet(ledger.data!, asOf, ledger.childrenMap)
      : { assets: 0, liabilities: 0, equity: 0, netIncome: 0, unrealizedFx: 0, balanced: false }
  );

  const childrenMap = $derived(ledger.data ? buildChildrenMap(ledger.data.accounts) : new Map());

  const assetAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'ASSET' && !a.placeholder)
  );
  const liabilityAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'LIABILITY' && !a.placeholder)
  );
  const equityAccounts = $derived(
    ledger.accounts.filter((a) => a.type === 'EQUITY' && !a.placeholder)
  );
</script>

<div class="grid min-h-0 flex-1 grid-cols-12 gap-2">
  <!-- Assets (Col 6) -->
  <ReportCard 
    title={i18n.t.assetsTitle} 
    description={i18n.t.bsAssetsDesc} 
    value={formatIDR(fromMinor('IDR', bs.assets))} 
    valueClass="text-income"
    class="col-span-6 flex-1"
  >
    {#each assetAccounts as acc (acc.id)}
      {@const bal = filteredVault ? accountBalanceMinor(acc.id, filteredVault, childrenMap) : 0}
      <AccountRow account={acc} balance={bal} />
    {:else}
      <p class="text-text-muted py-2 text-[11px] px-3">
        {i18n.t.noAssetAccounts}
      </p>
    {/each}
  </ReportCard>

  <!-- Liabilities & Equity (Col 6) -->
  <ReportCard
    title={`${i18n.t.liabilitiesTitle} & ${i18n.t.equityTitle}`}
    description={i18n.t.bsLiabEquityDesc}
    class="col-span-6 flex-1"
  >
    {#snippet headerRight()}
      <span class="px-2 py-0.5 text-[11px] {bs.balanced ? 'badge-ok' : 'badge-warn'}">
        {bs.balanced ? i18n.t.pasivaOk : i18n.t.pasivaMismatch}
      </span>
    {/snippet}

    <div class="min-h-0 flex-1 space-y-3 pr-1 text-[12px]">
      <div class="sharp-card p-3">
        <p class="text-expense mb-2 pb-2 text-[11px] font-bold">
          {i18n.t.liabilitiesTitle}
        </p>
        <div class="space-y-1">
          {#each liabilityAccounts as acc (acc.id)}
            {@const bal = filteredVault ? accountBalanceMinor(acc.id, filteredVault, childrenMap) : 0}
            <AccountRow account={acc} balance={bal} valueClass="text-expense" />
          {:else}
            <p class="text-text-muted text-[11px] px-2">
              {i18n.t.noLiabilityAccounts}
            </p>
          {/each}
        </div>
      </div>

      <div class="sharp-card p-3">
        <p class="text-teal mb-2 pb-2 text-[11px] font-bold">
          {i18n.t.equityTitle}
        </p>
        <div class="space-y-1">
          {#each equityAccounts as acc (acc.id)}
            {@const bal = filteredVault ? accountBalanceMinor(acc.id, filteredVault, childrenMap) : 0}
            <AccountRow account={acc} balance={bal} />
          {:else}
            <p class="text-text-muted text-[11px] px-2">
              {i18n.t.noEquityAccounts}
            </p>
          {/each}

          <div
            class="text-text-base border-line mt-2 flex justify-between border-t px-2 pb-1 pt-2 text-[11.5px] font-mono"
          >
            <span class="text-text-dim">{i18n.t.netIncomeCurrentPeriod}</span>
            <span class="font-proto {bs.netIncome < 0 ? 'text-expense' : 'text-income'}"
              >{formatIDR(bs.netIncome)}</span
            >
          </div>

          <div class="text-text-base flex justify-between px-2 py-1 text-[11.5px] font-mono">
            <span class="text-text-dim">{i18n.t.unrealizedFxGainLoss}</span>
            <span class="font-proto {bs.unrealizedFx < 0 ? 'text-expense' : 'text-income'}"
              >{formatIDR(bs.unrealizedFx)}</span
            >
          </div>
        </div>
      </div>
    </div>
  </ReportCard>
</div>
