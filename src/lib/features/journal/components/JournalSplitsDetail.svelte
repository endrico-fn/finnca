<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import type { Transaction } from '$lib/core/types';
  import { formatIDR, formatMinorToDisplay } from '$lib/core/format/currency';
  import { i18n } from '$lib/core/i18n.svelte';

  const formatUSD = (val: number) => formatMinorToDisplay(val, 'USD');

  let {
    tx,
    accountsById,
  }: {
    tx: Transaction;
    accountsById: Map<string, Account>;
  } = $props();
</script>

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
    {#each tx.splits as sp (sp.id)}
      {@const acc = accountsById.get(sp.accountId)}
      <tr>
        <td class="text-text-base px-3 py-1.5 font-aux">
          <span class="text-text-strong font-proto font-bold">{acc?.code ?? '?'}</span>
          <span class="ml-1.5 text-text-base">{acc?.name ?? sp.accountId.slice(0, 8)}</span>
        </td>
        <td
          class="numeric px-3 py-1.5 {sp.amount > 0
            ? 'text-income font-bold'
            : 'text-text-muted'}"
        >
          {sp.amount > 0
            ? tx.currency === 'USD'
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
            ? tx.currency === 'USD'
              ? formatUSD(-sp.amount)
              : formatIDR(-sp.amount)
            : '—'}
        </td>
        <td class="text-text-muted font-aux text-smaller px-3 py-1.5">{sp.memo ?? ''}</td>
      </tr>
    {/each}
  </tbody>
</table>
