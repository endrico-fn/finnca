<script lang="ts">
  import { i18n } from '$lib/core/i18n.svelte';
  import { formatMinorToDisplay } from '$lib/core/format/currency';
  import { accountTypeLabel } from '$lib/core/format/account';
  import type { AccountType } from '$lib/core/ipc/bindings';
  import { Badge, Card } from '$lib/components/ui';
  import { reportState } from '../state/report.svelte';

  const fmt = (n: number) => formatMinorToDisplay(n, 'IDR');

  let { asOf = '' }: { asOf?: string } = $props();

  $effect(() => {
    reportState.loadTrialBalance(asOf || undefined);
  });

  const tbReport = $derived(reportState.trialBalance);
  const rows = $derived(tbReport?.rows ?? []);
  const totalDebit = $derived(tbReport?.total_debit ?? 0);
  const totalCredit = $derived(tbReport?.total_credit ?? 0);
  const balanced = $derived(tbReport?.is_balanced ?? true);
</script>

<Card divided title={i18n.t.trialBalanceTitle} class="flex-1">
  {#snippet header()}
    <div class="flex min-w-0 items-center justify-end gap-4">
      <Badge size="m" tone={balanced ? 'ok' : 'warn'}>
        {balanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
      </Badge>
      <div
        class="font-proto text-smaller flex shrink-0 items-center gap-4 whitespace-nowrap tabular-nums"
      >
        <span class="shrink-0">
          {i18n.t.totalDebit}:
          <strong class="text-income font-bold">{fmt(totalDebit)}</strong>
        </span>
        <span class="shrink-0">
          {i18n.t.totalCredit}:
          <strong class="text-text-base font-bold">{fmt(totalCredit)}</strong>
        </span>
      </div>
    </div>
  {/snippet}
  <table class="w-full border-collapse">
    <thead class="bg-bg-card sticky top-0 z-10">
      <tr class="border-line text-text-base border-b">
        <th class="label-xs py-1.5 text-left font-normal whitespace-nowrap"
          >{i18n.t.code} &amp; {i18n.t.name}</th
        >
        <th class="label-xs w-32 py-1.5 text-left font-normal whitespace-nowrap">{i18n.t.type}</th>
        <th class="label-xs w-28 py-1.5 text-right font-normal whitespace-nowrap"
          >{i18n.t.totalDebit}</th
        >
        <th class="label-xs w-28 py-1.5 text-right font-normal whitespace-nowrap"
          >{i18n.t.totalCredit}</th
        >
      </tr>
    </thead>
    <tbody class="divide-line/40 divide-y">
      {#each rows as r (r.account_id)}
        <tr class="hover:bg-bg-row-active transition-colors">
          <td class="text-text-base text-smaller py-1">
            <span class="text-text-dim font-proto mr-1.5">{r.code}</span>
            <span>{r.name}</span>
          </td>
          <td class="text-text-base text-smaller font-proto py-1 whitespace-nowrap"
            >{accountTypeLabel(r.account_type as AccountType)}</td
          >
          <td
            class="font-proto text-smaller py-1 text-right whitespace-nowrap tabular-nums {r.debit
              ? 'text-income'
              : 'text-text-muted'}"
          >
            {r.debit ? fmt(r.debit) : '—'}
          </td>
          <td
            class="font-proto text-smaller py-1 text-right whitespace-nowrap tabular-nums {r.credit
              ? 'text-text-base'
              : 'text-text-muted'}"
          >
            {r.credit ? fmt(r.credit) : '—'}
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="4" class="text-text-muted text-small py-6 text-center">
            {i18n.t.noTbRecords}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</Card>
