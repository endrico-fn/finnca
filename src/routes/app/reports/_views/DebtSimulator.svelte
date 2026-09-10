<script lang="ts">
  import { ledger } from '$lib/accounting/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { ReportCard } from '$lib/components/ui';
  import {
    accountBalanceMinor,
    formatIDR,
    fromMinor,
    buildChildrenMap,
    suggestInstallmentOptions,
  } from '$lib/accounting/finance';

  let extraPaymentStr = $state('500000');

  const debtStats = $derived.by(() => {
    if (!ledger.data) return [];
    const cmap = buildChildrenMap(ledger.accounts);
    return ledger.accounts
      .filter((a) => a.type === 'LIABILITY' && !a.placeholder)
      .map((a) => {
        const bal = accountBalanceMinor(a.id, ledger.data!, cmap);
        const balanceIdrMinor = Math.abs(bal);
        return {
          id: a.id,
          name: a.name,
          balance: balanceIdrMinor,
          interestRate: a.interestRate || 0,
        };
      })
      .filter((d) => d.balance > 0);
  });

  const avalanche = $derived.by(() => {
    return [...debtStats].sort((a, b) => b.interestRate - a.interestRate);
  });

  const snowball = $derived.by(() => {
    return [...debtStats].sort((a, b) => a.balance - b.balance);
  });

  let strategy = $state<'AVALANCHE' | 'SNOWBALL'>('AVALANCHE');
  const selectedStrategy = $derived(strategy === 'AVALANCHE' ? avalanche : snowball);
</script>

<ReportCard
  title={i18n.t.debtSimTitle}
  description={i18n.t.debtSimDesc}
>
  {#snippet headerRight()}
    <div class="flex items-center gap-4">
      <div class="bg-bg-app border-line flex h-7 border p-0.5">
        <button
          type="button"
          onclick={() => (strategy = 'AVALANCHE')}
          class="font-proto px-3 text-[10px] font-bold uppercase transition-colors {strategy ===
          'AVALANCHE'
            ? 'bg-teal text-bg-app'
            : 'text-text-muted hover:text-text-base'}"
        >
          AVALANCHE
        </button>
        <button
          type="button"
          onclick={() => (strategy = 'SNOWBALL')}
          class="font-proto px-3 text-[10px] font-bold uppercase transition-colors {strategy ===
          'SNOWBALL'
            ? 'bg-teal text-bg-app'
            : 'text-text-muted hover:text-text-base'}"
        >
          SNOWBALL
        </button>
      </div>
    </div>
  {/snippet}

  {#if debtStats.length === 0}
    <div
      class="text-text-dim font-proto border-line m-4 border border-dashed p-10 text-center text-[11px]"
    >
      {i18n.t.noActiveLiabilityAccounts}
      <br /><span class="mt-2 block text-[9px]">{i18n.t.debtSimHint}</span>
    </div>
  {:else}
    <div class="overflow-y-auto p-4">
      <div class="font-proto text-text-dim bg-bg-app border-line mb-4 border p-3 text-[11px]">
        {#if strategy === 'AVALANCHE'}
          <strong>AVALANCHE METHOD:</strong> Mathematically saves the most money by targeting the highest
          interest rates first.
        {:else}
          <strong>SNOWBALL METHOD:</strong> Psychologically rewarding by clearing the smallest debts first
          to build momentum.
        {/if}
      </div>

      <div class="space-y-3">
        {#each selectedStrategy as debt, i (debt.id || debt)}
          <div
            class="border-line bg-bg-app relative flex flex-col gap-2 overflow-hidden border p-3"
          >
            {#if i === 0}
              <div class="bg-income absolute top-0 right-0 bottom-0 w-1"></div>
              <div
                class="bg-income/10 text-income font-proto border-income/30 absolute top-3 right-3 border px-1.5 py-0.5 text-[9px] font-bold"
              >
                {i18n.t.targetDebt}
              </div>
            {/if}

            <div class="flex items-center gap-2">
              <div
                class="bg-expense/10 border-expense/30 text-expense font-proto flex h-5 w-5 items-center justify-center border text-[10px] font-bold rounded-none"
              >
                {i + 1}
              </div>
              <span class="font-proto text-text-strong text-xs font-bold">{debt.name}</span>
            </div>

            <div class="mt-2 grid grid-cols-3 gap-4">
              <div>
                <span class="text-text-muted font-proto block text-[9px] uppercase"
                  >{i18n.t.currentBalance}</span
                >
                <span class="font-proto text-expense block text-[11px] font-bold"
                  >Rp {formatIDR(fromMinor('IDR', debt.balance))}</span
                >
              </div>
              <div>
                <span class="text-text-muted font-proto block text-[9px] uppercase"
                  >{i18n.t.interestRateApr}</span
                >
                <span
                  class="font-proto block text-[11px] font-bold {debt.interestRate >= 15
                    ? 'text-expense'
                    : 'text-text-base'}">{debt.interestRate}%</span
                >
              </div>
              <div>
                <span class="text-text-muted font-proto block text-[9px] uppercase"
                  >Est. 12mo Min</span
                >
                <span
                  class="font-proto block text-[11px] font-bold {i === 0
                    ? 'text-income'
                    : 'text-text-dim'}"
                >
                  {suggestInstallmentOptions(debt.balance, debt.interestRate).find(o => o.count === 12) 
                    ? formatIDR(fromMinor('IDR', suggestInstallmentOptions(debt.balance, debt.interestRate).find(o => o.count === 12)!.amount)) 
                    : 0}
                </span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</ReportCard>
