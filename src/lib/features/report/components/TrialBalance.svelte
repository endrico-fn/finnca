<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { accountTypeLabel } from '$lib/core/format/account';
  import type { AccountType } from '$lib/core/ipc/bindings';
  import { Badge } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number, c = 'IDR') => formatMinorToDisplay(n, c);

  let { asOf = '' }: { asOf?: string } = $props();

  $effect(() => {
    reportState.loadTrialBalance(asOf || undefined);
  });

  const tbReport = $derived(reportState.trialBalance);
  const rows = $derived(tbReport?.rows ?? []);
  const reportCurrency = $derived(rows[0]?.currency ?? 'IDR');
  const totalDebit = $derived(tbReport?.total_debit ?? 0);
  const totalCredit = $derived(tbReport?.total_credit ?? 0);
  const balanced = $derived(tbReport?.is_balanced ?? true);
</script>

<div class="border-line bg-bg-card flex min-h-0 w-full flex-1 flex-col overflow-hidden border">
  <div class="border-line/60 flex shrink-0 items-center justify-between border-b px-3 py-1.5">
    <div class="font-proto text-smaller flex shrink-0 items-center gap-2">
      <Badge size="s" tone={balanced ? 'ok' : 'err'}>
        {balanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
      </Badge>
      <span class="text-line mx-1">|</span>
      <span class="text-text-dim shrink-0">
        {i18n.t.totalDebit}:
        <strong class="text-text-strong ml-1 font-bold tabular-nums"
          >{fmt(totalDebit, reportCurrency)}</strong
        >
      </span>
      <span class="text-line mx-1">|</span>
      <span class="text-text-dim shrink-0">
        {i18n.t.totalCredit}:
        <strong class="text-text-strong ml-1 font-bold tabular-nums"
          >{fmt(totalCredit, reportCurrency)}</strong
        >
      </span>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto">
    <table class="sharp-table w-full border-x-0 border-t-0" spellcheck="false">
      <thead class="bg-bg-card sticky top-0 z-10">
        <tr class="border-line border-b">
          <th class="w-24 py-2 pl-3 whitespace-nowrap">{i18n.t.colCode}</th>
          <th class="px-3 py-2">{i18n.t.name}</th>
          <th class="w-32 px-3 py-2 whitespace-nowrap">{i18n.t.type}</th>
          <th class="numeric w-36 px-3 py-2 text-right whitespace-nowrap">{i18n.t.totalDebit}</th>
          <th class="numeric w-36 py-2 pr-3 text-right whitespace-nowrap">{i18n.t.totalCredit}</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as r (r.account_id)}
          <tr class="hover:bg-bg-row-active transition-colors">
            <td
              class="font-proto text-text-muted text-smaller w-24 py-2 pl-3 whitespace-nowrap tabular-nums"
            >
              {r.code}
            </td>
            <td class="font-aux text-text-strong text-small truncate px-3 py-2 normal-case">
              {r.name}
            </td>
            <td class="font-proto text-text-dim text-smaller w-32 px-3 py-2 whitespace-nowrap">
              {accountTypeLabel(r.account_type as AccountType)}
            </td>
            <td
              class="numeric font-proto text-smaller w-36 px-3 py-2 text-right whitespace-nowrap tabular-nums {r.debit
                ? 'text-income font-bold'
                : 'text-text-dim'}"
            >
              {r.debit ? fmt(r.debit, r.currency) : '—'}
            </td>
            <td
              class="numeric font-proto text-smaller w-36 py-2 pr-3 text-right whitespace-nowrap tabular-nums {r.credit
                ? 'text-text-strong font-bold'
                : 'text-text-dim'}"
            >
              {r.credit ? fmt(r.credit, r.currency) : '—'}
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="5" class="text-text-muted font-aux text-small py-8 text-center">
              {i18n.t.noTbRecords}
            </td>
          </tr>
        {/each}
      </tbody>
      {#if rows.length > 0}
        <tfoot class="border-line bg-bg-card font-proto sticky bottom-0 z-10 border-t-2 font-bold">
          <tr>
            <td
              colspan="3"
              class="text-smaller text-text-strong py-2 pl-3 font-bold tracking-widest uppercase"
            >
              {i18n.t.totals}
            </td>
            <td
              class="numeric text-smaller text-income w-36 px-3 py-2 text-right font-bold whitespace-nowrap tabular-nums"
            >
              {fmt(totalDebit, reportCurrency)}
            </td>
            <td
              class="numeric text-smaller text-text-strong w-36 py-2 pr-3 text-right font-bold whitespace-nowrap tabular-nums"
            >
              {fmt(totalCredit, reportCurrency)}
            </td>
          </tr>
        </tfoot>
      {/if}
    </table>
  </div>
</div>
