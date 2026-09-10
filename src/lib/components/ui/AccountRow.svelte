<script lang="ts">
  import type { Account } from '$lib/accounting/types';
  import { formatIDR, formatUSD } from '$lib/accounting/finance';

  let {
    account,
    balance,
    valueClass = 'text-text-base',
    percentage,
  }: {
    account: Account;
    balance: number;
    valueClass?: string;
    percentage?: string;
  } = $props();

  const formattedBalance = $derived(
    account.currency === 'USD' ? formatUSD(Math.abs(balance)) : formatIDR(Math.abs(balance))
  );
</script>

<div class="text-text-base hover:bg-bg-row-active/30 flex items-center justify-between px-2 py-1.5 text-[11.5px] font-mono">
  <div class="flex items-center gap-2 truncate">
    <strong class="text-text-dim font-proto text-[10px]">{account.code}</strong>
    <span class="text-text-strong truncate">{account.name}</span>
  </div>
  <div class="ml-3 flex shrink-0 items-center gap-3">
    {#if percentage}
      <span class="text-text-dim font-proto w-10 text-right text-[9px]">{percentage}</span>
    {/if}
    <span class="font-proto shrink-0 {valueClass}">
      {formattedBalance}
    </span>
  </div>
</div>
