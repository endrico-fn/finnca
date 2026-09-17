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

<Card divided title={i18n.t.trialBalanceTitle} class="flex flex-1 min-h-0 flex-col w-full overflow-hidden">
  {#snippet header()}
    <div class="flex min-w-0 items-center justify-end gap-4">
      <Badge size="m" tone={balanced ? 'ok' : 'warn'}>
        {balanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
      </Badge>
      <div
        class="font-proto text-smaller flex shrink-0 items-center gap-4 whitespace-nowrap tabular-nums"
      >
        <span class="shrink-0 text-text-dim">
          {i18n.t.totalDebit}:
          <strong class="text-text-white font-bold">{fmt(totalDebit)}</strong>
        </span>
        <span class="shrink-0 text-text-dim">
          {i18n.t.totalCredit}:
          <strong class="text-text-white font-bold">{fmt(totalCredit)}</strong>
        </span>
      </div>
    </div>
  {/snippet}

  <div class="flex-1 overflow-y-auto">
    <table class="sharp-table w-full">
      <thead>
        <tr>
          <th class="w-24 pl-3">{i18n.t.colCode}</th>
          <th class="px-3">{i18n.t.name}</th>
          <th class="w-32 px-3">{i18n.t.type}</th>
          <th class="numeric w-36 px-3">{i18n.t.totalDebit}</th>
          <th class="numeric w-36 pr-3">{i18n.t.totalCredit}</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as r (r.account_id)}
          <tr>
            <td class="font-proto text-text-muted text-smaller w-24 pl-3 whitespace-nowrap">
              {r.code}
            </td>
            <td class="font-aux text-text-white text-small px-3">
              <span class="truncate">{r.name}</span>
            </td>
            <td class="font-proto text-text-dim text-smaller w-32 px-3 whitespace-nowrap">
              {accountTypeLabel(r.account_type as AccountType)}
            </td>
            <td class="numeric font-proto text-text-white text-smaller w-36 px-3 whitespace-nowrap tabular-nums">
              {r.debit ? fmt(r.debit) : '—'}
            </td>
            <td class="numeric font-proto text-text-white text-smaller w-36 pr-3 whitespace-nowrap tabular-nums">
              {r.credit ? fmt(r.credit) : '—'}
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="5" class="text-text-muted text-small py-8 text-center">
              {i18n.t.noTbRecords}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</Card>
