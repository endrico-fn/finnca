<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { Badge, Card, KpiCard, AnimatedCounter } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number, c = 'IDR') => formatMinorToDisplay(n, c);

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

<div class="flex min-h-0 w-full flex-1 flex-col gap-4">
  <div class="grid shrink-0 grid-cols-2 gap-4 sm:grid-cols-4">
    <KpiCard label={i18n.t.totalAssetsLabel} labelClass="text-asset">
      <AnimatedCounter
        value={totalAssets}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {assetRows.length}
        {i18n.t.accountsTitle}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.totalLiabilitiesLabel} labelClass="text-liability">
      <AnimatedCounter
        value={totalLiabilities}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {liabilityRows.length}
        {i18n.t.accountsTitle}
      </span>
    </KpiCard>

    <KpiCard label={i18n.t.totalEquityLabel} labelClass="text-equity">
      <AnimatedCounter
        value={totalEquity}
        currency="IDR"
        class="text-medium text-text-white block leading-tight font-bold"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {equityRows.length}
        {i18n.t.accountsTitle}
      </span>
    </KpiCard>

    <KpiCard
      label={i18n.t.discrepancyLabel}
      badge={isBalanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
      badgeTone={isBalanced ? 'ok' : 'err'}
    >
      <AnimatedCounter
        value={discrepancy}
        currency="IDR"
        class="text-medium block leading-tight font-bold {isBalanced
          ? 'text-text-white'
          : 'text-expense'}"
      />
      <span class="text-text-dim font-proto text-smaller block truncate leading-tight">
        {isBalanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
      </span>
    </KpiCard>
  </div>

  <div class="grid min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-2">
    <Card divided title={i18n.t.assetsTitle} class="flex min-h-0 flex-1 flex-col">
      {#snippet header()}
        <span class="font-proto text-text-white text-small font-bold tabular-nums">
          {fmt(totalAssets)}
        </span>
      {/snippet}

      {#if assetRows.length === 0}
        <div
          class="text-text-dim font-aux text-small flex flex-1 items-center justify-center py-8 text-center"
        >
          {i18n.t.noAssetAccounts}
        </div>
      {:else}
        <div class="flex-1 overflow-y-auto">
          <table class="sharp-table w-full">
            <thead>
              <tr>
                <th class="w-24 pl-3">{i18n.t.colCode}</th>
                <th class="px-3">{i18n.t.name}</th>
                <th class="numeric w-24 px-2">{i18n.t.colPctAssets}</th>
                <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
              </tr>
            </thead>
            <tbody>
              {#each assetRows as row (row.account_id)}
                {@const pct = totalAssets > 0 ? (row.amount / totalAssets) * 100 : 0}
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
                  <td
                    class="numeric font-proto text-text-dim text-smaller w-24 px-2 whitespace-nowrap tabular-nums"
                  >
                    {totalAssets > 0 ? `${pct.toFixed(1)}%` : '—'}
                  </td>
                  <td
                    class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums font-bold"
                  >
                    {fmt(row.amount, row.currency)}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </Card>

    <div class="grid min-h-0 flex-1 grid-rows-2 gap-4">
      <Card divided title={i18n.t.liabilitiesTitle} class="flex min-h-0 flex-1 flex-col">
        {#snippet header()}
          <span class="font-proto text-text-white text-small font-bold tabular-nums">
            {fmt(totalLiabilities)}
          </span>
        {/snippet}

        {#if liabilityRows.length === 0}
          <div
            class="text-text-dim font-aux text-small flex flex-1 items-center justify-center py-4 text-center"
          >
            {i18n.t.noLiabilityAccounts}
          </div>
        {:else}
          <div class="flex-1 overflow-y-auto">
            <table class="sharp-table w-full">
              <thead>
                <tr>
                  <th class="w-24 pl-3">{i18n.t.colCode}</th>
                  <th class="px-3">{i18n.t.name}</th>
                  <th class="numeric w-24 px-2">{i18n.t.colPctAssets}</th>
                  <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
                </tr>
              </thead>
              <tbody>
                {#each liabilityRows as row (row.account_id)}
                  {@const pct = totalAssets > 0 ? (row.amount / totalAssets) * 100 : 0}
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
                    <td
                      class="numeric font-proto text-text-dim text-smaller w-24 px-2 whitespace-nowrap tabular-nums"
                    >
                      {totalAssets > 0 ? `${pct.toFixed(1)}%` : '—'}
                    </td>
                    <td
                      class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums font-bold"
                    >
                      {fmt(row.amount, row.currency)}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </Card>

      <Card divided title={i18n.t.equityTitle} class="flex min-h-0 flex-1 flex-col">
        {#snippet header()}
          <span class="font-proto text-text-white text-small font-bold tabular-nums">
            {fmt(totalEquity + netIncome)}
          </span>
        {/snippet}

        {#if equityRows.length === 0 && netIncome === 0}
          <div
            class="text-text-dim font-aux text-small flex flex-1 items-center justify-center py-4 text-center"
          >
            {i18n.t.noEquityAccounts}
          </div>
        {:else}
          <div class="flex-1 overflow-y-auto">
            <table class="sharp-table w-full">
              <thead>
                <tr>
                  <th class="w-24 pl-3">{i18n.t.colCode}</th>
                  <th class="px-3">{i18n.t.name}</th>
                  <th class="numeric w-24 px-2">{i18n.t.colPctAssets}</th>
                  <th class="numeric w-36 pr-3">{i18n.t.amount}</th>
                </tr>
              </thead>
              <tbody>
                {#each equityRows as row (row.account_id)}
                  {@const pct = totalAssets > 0 ? (row.amount / totalAssets) * 100 : 0}
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
                    <td
                      class="numeric font-proto text-text-dim text-smaller w-24 px-2 whitespace-nowrap tabular-nums"
                    >
                      {totalAssets > 0 ? `${pct.toFixed(1)}%` : '—'}
                    </td>
                    <td
                      class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums font-bold"
                    >
                      {fmt(row.amount, row.currency)}
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
                  <td
                    class="numeric font-proto text-text-dim text-smaller w-24 px-2 whitespace-nowrap tabular-nums"
                  >
                    {totalAssets > 0
                      ? `${((netIncome / totalAssets) * 100).toFixed(1)}%`
                      : '—'}
                  </td>
                  <td
                    class="numeric font-proto text-text-white text-smaller w-36 pr-3 font-bold whitespace-nowrap tabular-nums"
                  >
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

  <Card class="border-line bg-bg-card/60 shrink-0 p-4">
    <div class="flex flex-wrap items-center justify-between gap-4">
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
