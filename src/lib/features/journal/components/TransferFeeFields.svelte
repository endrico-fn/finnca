<script lang="ts">
  import type { Account } from '$lib/core/ipc/bindings';
  import { i18n } from '$lib/core/i18n.svelte';
  import { AccountSelectDropdown } from '$lib/components/ui';

  let {
    feeAccount = $bindable(''),
    feeAmount = $bindable(''),
    showFee = $bindable(false),
    currency,
    accounts,
    accountsById,
  }: {
    feeAccount: string;
    feeAmount: string;
    showFee: boolean;
    currency: string;
    accounts: Account[];
    accountsById: Map<string, Account>;
  } = $props();
</script>

<div class="border-line/40 mt-2 border-t pt-2">
  <label class="font-proto text-smaller inline-flex cursor-pointer items-center gap-2">
    <input type="checkbox" bind:checked={showFee} class="accent-teal" />
    <span>{i18n.t.adminFeeOpt}</span>
  </label>
  {#if showFee}
    <div class="mt-2 grid gap-2 md:grid-cols-[1fr_160px]">
      <div>
        <span class="label-xs">{i18n.t.adminFeeAccount}</span>
        <AccountSelectDropdown
          bind:value={feeAccount}
          {accounts}
          {accountsById}
          filterType="EXPENSE"
          placeholder={i18n.t.txSelectAccount}
          class="mt-1 w-full"
        />
      </div>
      <div>
        <span class="label-xs">{i18n.t.amount} ({currency})</span>
        <input
          bind:value={feeAmount}
          placeholder={i18n.t.txAmountExample}
          type="text"
          inputmode="decimal"
          autocomplete="off"
          class="sharp-input font-proto mt-1 w-full text-right"
        />
      </div>
    </div>
  {/if}
</div>
