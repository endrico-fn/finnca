<script lang="ts">
  import type { Account, JournalEntryView } from '$lib/core/ipc/bindings';
  import { formatIDR, formatUSD } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';

  let {
    entry,
    accountsById,
  }: {
    entry: JournalEntryView;
    accountsById?: Map<string, Account>;
  } = $props();

  const activeEntry = $derived(entry);
</script>

{#if activeEntry}
  <table class="sharp-table">
    <thead>
      <tr>
        <th class="px-3 py-1.5 text-left">{i18n.t.account}</th>
        <th class="numeric px-3 py-1.5">{i18n.t.debit}</th>
        <th class="numeric px-3 py-1.5">{i18n.t.credit}</th>
        <th class="px-3 py-1.5 text-left">{i18n.t.note}</th>
      </tr>
    </thead>
    <tbody>
      {#each activeEntry.postings as sp (sp.id)}
        {@const acc = accountsById?.get(sp.account_id)}
        <tr>
          <td class="text-text-base font-aux px-3 py-1.5">
            <span class="text-text-strong font-proto font-bold"
              >{sp.account_code || acc?.code || '?'}</span
            >
            <span class="text-text-base ml-1.5"
              >{sp.account_name || acc?.name || sp.account_id.slice(0, 8)}</span
            >
          </td>
          <td
            class="numeric px-3 py-1.5 {sp.amount > 0
              ? 'text-income font-bold'
              : 'text-text-muted'}"
          >
            {sp.amount > 0
              ? activeEntry.currency === 'USD'
                ? formatUSD(sp.amount)
                : formatIDR(sp.amount)
              : '—'}
          </td>
          <td
            class="numeric px-3 py-1.5 {sp.amount < 0
              ? 'text-text-strong font-bold'
              : 'text-text-muted'}"
          >
            {sp.amount < 0
              ? activeEntry.currency === 'USD'
                ? formatUSD(-sp.amount)
                : formatIDR(-sp.amount)
              : '—'}
          </td>
          <td class="text-text-muted font-aux text-smaller px-3 py-1.5">{sp.memo ?? ''}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
