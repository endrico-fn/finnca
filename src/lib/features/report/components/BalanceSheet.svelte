<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { Badge, Card } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number) => formatMinorToDisplay(n, 'IDR');

  let { asOf = '' }: { asOf?: string } = $props();

  $effect(() => {
    reportState.loadBalanceSheet(asOf || undefined);
  });

  const bs = $derived(reportState.balanceSheet);
  const assetRows = $derived(bs?.asset_rows ?? []);
  const liabilityRows = $derived(bs?.liability_rows ?? []);
  const equityRows = $derived(bs?.equity_rows ?? []);

  const totalAssets = $derived(bs?.total_assets ?? 0);
  const totalLiabilities = $derived(bs?.total_liabilities ?? 0);
  const totalEquity = $derived(bs?.total_equity ?? 0);
  const netIncome = $derived(bs?.net_income ?? 0);
  const discrepancy = $derived(bs?.discrepancy ?? 0);
  const isBalanced = $derived(bs?.is_balanced ?? true);
</script>

<div class="flex flex-1 flex-col gap-3 w-full min-h-0">
  <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-4 shrink-0">
    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.totalAssetsLabel}
      </div>
      <div class="font-proto text-text-white text-lg font-bold tabular-nums">
        {fmt(totalAssets)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.totalLiabilitiesLabel}
      </div>
      <div class="font-proto text-text-white text-lg font-bold tabular-nums">
        {fmt(totalLiabilities)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="font-proto text-text-dim text-smaller uppercase tracking-wider">
        {i18n.t.totalEquityLabel}
      </div>
      <div class="font-proto text-text-white text-lg font-bold tabular-nums">
        {fmt(totalEquity)}
      </div>
    </Card>

    <Card class="p-3">
      <div class="flex items-center justify-between gap-2">
        <span class="font-proto text-text-dim text-smaller uppercase tracking-wider truncate">
          {i18n.t.discrepancyLabel}
        </span>
        <Badge size="s" tone={isBalanced ? 'ok' : 'err'} class="shrink-0">
          {isBalanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
        </Badge>
      </div>
      <div class="font-proto text-lg font-bold tabular-nums {isBalanced ? 'text-text-white' : 'text-expense'}">
        {fmt(discrepancy)}
      </div>
    </Card>
  </div>

  <div class="grid grid-cols-1 gap-3 lg:grid-cols-2 flex-1 min-h-0">
    <Card divided title={i18n.t.assetsTitle} class="flex flex-col flex-1 min-h-0">
      {#snippet header()}
        <span class="font-proto text-text-white text-small font-bold tabular-nums">
          {fmt(totalAssets)}
        </span>
      {/snippet}

      {#if assetRows.length === 0}
        <div class="text-text-dim font-aux flex flex-1 items-center justify-center py-8 text-center text-small">
          {i18n.t.noAssetAccounts}
        </div>
      {:else}
        <div class="flex-1 overflow-y-auto">
          <table class="sharp-table w-full">
            <thead>
              <tr>
                <th class="w-24 pl-3">{i18n.t.colCode}</th>
                <th class="px-3">{i18n.t.name}</th>
                <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
              </tr>
            </thead>
            <tbody>
              {#each assetRows as row (row.account_id)}
                <tr>
                  <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
                    {row.code}
                  </td>
                  <td class="font-aux text-text-white text-small px-3">
                    <div class="flex items-center gap-2">
                      <span class="bg-asset size-1.5 shrink-0"></span>
                      <span class="truncate">{row.name}</span>
                    </div>
                  </td>
                  <td class="numeric font-proto text-smaller w-36 pr-3 whitespace-nowrap tabular-nums">
                    {fmt(row.amount)}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </Card>

    <div class="grid grid-rows-2 gap-3 flex-1 min-h-0">
      <Card divided title={i18n.t.liabilitiesTitle} class="flex flex-col flex-1 min-h-0">
        {#snippet header()}
          <span class="font-proto text-text-white text-small font-bold tabular-nums">
            {fmt(totalLiabilities)}
          </span>
        {/snippet}

        {#if liabilityRows.length === 0}
          <div class="text-text-dim font-aux flex flex-1 items-center justify-center py-4 text-center text-small">
            {i18n.t.noLiabilityAccounts}
          </div>
        {:else}
          <div class="flex-1 overflow-y-auto">
            <table class="sharp-table w-full">
              <thead>
                <tr>
                  <th class="w-24 pl-3">{i18n.t.colCode}</th>
                  <th class="px-3">{i18n.t.name}</th>
                  <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
                </tr>
              </thead>
              <tbody>
                {#each liabilityRows as row (row.account_id)}
                  <tr>
                    <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
                      {row.code}
                    </td>
                    <td class="font-aux text-text-white text-small px-3">
                      <div class="flex items-center gap-2">
                        <span class="bg-liability size-1.5 shrink-0"></span>
                        <span class="truncate">{row.name}</span>
                      </div>
                    </td>
                    <td class="numeric font-proto text-smaller w-36 pr-3 whitespace-nowrap tabular-nums">
                      {fmt(row.amount)}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </Card>

      <Card divided title={i18n.t.equityTitle} class="flex flex-col flex-1 min-h-0">
        {#snippet header()}
          <span class="font-proto text-text-white text-small font-bold tabular-nums">
            {fmt(totalEquity + netIncome)}
          </span>
        {/snippet}

        {#if equityRows.length === 0 && netIncome === 0}
          <div class="text-text-dim font-aux flex flex-1 items-center justify-center py-4 text-center text-small">
            {i18n.t.noEquityAccounts}
          </div>
        {:else}
          <div class="flex-1 overflow-y-auto">
            <table class="sharp-table w-full">
              <thead>
                <tr>
                  <th class="w-24 pl-3">{i18n.t.colCode}</th>
                  <th class="px-3">{i18n.t.name}</th>
                  <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
                </tr>
              </thead>
              <tbody>
                {#each equityRows as row (row.account_id)}
                  <tr>
                    <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
                      {row.code}
                    </td>
                    <td class="font-aux text-text-white text-small px-3">
                      <div class="flex items-center gap-2">
                        <span class="bg-equity size-1.5 shrink-0"></span>
                        <span class="truncate">{row.name}</span>
                      </div>
                    </td>
                    <td class="numeric font-proto text-smaller w-36 pr-3 whitespace-nowrap tabular-nums">
                      {fmt(row.amount)}
                    </td>
                  </tr>
                {/each}
                <tr class="bg-bg-card/40">
                  <td class="font-proto text-text-dim text-smaller w-24 pl-3 whitespace-nowrap">
                    Σ
                  </td>
                  <td class="font-proto text-text-white text-small px-3 font-medium">
                    {i18n.t.netIncomeLoss}
                  </td>
                  <td class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums font-bold">
                    {fmt(netIncome)}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        {/if}
      </Card>
    </div>
  </div>

  <Card class="border-line bg-bg-card/60 p-2.5 shrink-0">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="font-proto text-smaller text-text-dim flex items-center gap-2">
        <span class="text-text-base font-bold">A = L + E</span>
        <span>•</span>
        <span>{fmt(totalAssets)} = {fmt(totalLiabilities)} + {fmt(totalEquity + netIncome)}</span>
      </div>
      <div class="font-proto text-smaller flex items-center gap-2">
        <span class="text-text-dim">{i18n.t.status}:</span>
        <Badge size="s" tone={isBalanced ? 'ok' : 'err'}>
          {isBalanced ? i18n.t.statementBalanced : i18n.t.statementImbalance}
        </Badge>
      </div>
    </div>
  </Card>
</div>
