<script lang="ts">
  import type { Account } from '$lib/accounting/types';
  import { formatIDR, formatUSD } from '$lib/accounting/finance';
  import { i18n } from '$lib/i18n.svelte';
  import { resolve } from '$app/paths';

  let {
    account,
    balance,
    valueClass = 'text-text-base',
    percentage,
    clickable = true,
  }: {
    account: Account;
    balance: number;
    valueClass?: string;
    percentage?: string;
    clickable?: boolean;
  } = $props();

  const formattedBalance = $derived(
    account.currency === 'USD' ? formatUSD(Math.abs(balance)) : formatIDR(Math.abs(balance))
  );
</script>

{#if clickable && account.code}
  <a
    href={resolve('/app/accounts/[code]', { code: account.code })}
    class="text-text-base hover:bg-bg-row-active text-small group flex cursor-pointer items-center justify-between px-3 py-1.5 transition-colors"
    title={i18n.t.accountViewLedgerTitle
      .replace('{code}', account.code)
      .replace('{name}', account.name)}
  >
    <div class="flex items-center gap-2 truncate">
      <strong class="text-text-dim group-hover:text-teal font-proto text-smaller transition-colors">
        {account.code}
      </strong>
      <span class="text-text-strong group-hover:text-text-white truncate transition-colors">
        {account.name}
      </span>
    </div>
    <div class="ml-3 flex shrink-0 items-center gap-3">
      {#if percentage}
        <span class="text-text-dim font-proto text-smaller w-10 text-right tabular-nums">
          {percentage}
        </span>
      {/if}
      <span class="font-proto shrink-0 tabular-nums {valueClass}">
        {formattedBalance}
      </span>
    </div>
  </a>
{:else}
  <div
    class="text-text-base hover:bg-bg-row-active text-small flex items-center justify-between px-3 py-1.5"
  >
    <div class="flex items-center gap-2 truncate">
      <strong class="text-text-dim font-proto text-smaller">{account.code}</strong>
      <span class="text-text-strong truncate">{account.name}</span>
    </div>
    <div class="ml-3 flex shrink-0 items-center gap-3">
      {#if percentage}
        <span class="text-text-dim font-proto text-smaller w-10 text-right tabular-nums">
          {percentage}
        </span>
      {/if}
      <span class="font-proto shrink-0 tabular-nums {valueClass}">
        {formattedBalance}
      </span>
    </div>
  </div>
{/if}
