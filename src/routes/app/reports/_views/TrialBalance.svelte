<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { trialBalance, totalDebitsCredits, fromMinor } from '$lib/accounting/finance';
  import { ReportCard } from '$lib/components/ui';

  const fmt = (n: number) => n.toLocaleString('en-US');

  let { asOf = '' }: { asOf?: string } = $props();

  const tb = $derived(ledger.data ? trialBalance(ledger.data, undefined, asOf) : []);
  const totals = $derived(totalDebitsCredits(tb));
  const balanced = $derived(totals.debit === totals.credit);
</script>

<ReportCard
  title={i18n.t.trialBalanceTitle}
  class="flex-1"
>
  {#snippet headerRight()}
    <div class="flex items-center gap-4">
      <span class="px-2 py-0.5 text-[10px] font-bold {balanced ? 'badge-ok' : 'badge-warn'}">
        {balanced ? i18n.t.auditBalanced : i18n.t.auditUnbalanced}
      </span>
      <div class="font-proto flex items-center gap-4 text-[11px]">
        <span>
          {i18n.t.totalDebit}:
          <strong class="text-income font-bold">Rp {fmt(fromMinor('IDR', totals.debit))}</strong>
        </span>
        <span>
          {i18n.t.totalCredit}:
          <strong class="text-text-base font-bold">Rp {fmt(fromMinor('IDR', totals.credit))}</strong>
        </span>
      </div>
    </div>
  {/snippet}
    <table class="w-full border-collapse">
      <thead class="bg-bg-card sticky top-0 z-10">
        <tr class="border-line text-text-base border-b">
          <th class="label-xs py-2 text-left font-normal">{i18n.t.code} &amp; {i18n.t.name}</th>
          <th class="label-xs w-30 py-2 text-left font-normal">{i18n.t.type}</th>
          <th class="label-xs w-35 py-2 text-right font-normal">{i18n.t.totalDebit}</th>
          <th class="label-xs w-35 py-2 text-right font-normal">{i18n.t.totalCredit}</th>
        </tr>
      </thead>
      <tbody class="divide-line/40 divide-y">
        {#each tb as r (r.account.id)}
          <tr class="hover:bg-bg-row-active/40">
            <td class="text-text-base py-2">
              <span class="text-text-dim font-proto mr-1.5">{r.account.code}</span>
              <span>{r.account.name}</span>
            </td>
            <td class="text-text-icon py-2 text-[11px]">{r.account.type}</td>
            <td class="font-proto py-2 text-right {r.debit ? 'text-income' : 'text-text-muted'}">
              {r.debit ? fmt(fromMinor('IDR', r.debit)) : '—'}
            </td>
            <td
              class="font-proto py-2 text-right {r.credit ? 'text-text-base' : 'text-text-muted'}"
            >
              {r.credit ? fmt(fromMinor('IDR', r.credit)) : '—'}
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="4" class="text-text-muted py-8 text-center text-[12px]">
              {i18n.t.noTbRecords}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
</ReportCard>
