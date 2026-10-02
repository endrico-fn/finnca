<script lang="ts">
  import type { AccountBalanceView } from '$lib/core/ipc/bindings';
  import { listAccountsCmd } from '$lib/core/ipc/bindings';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Card, Tabs, Badge, EmptyState } from '$lib/components/ui';
  import { formatIDR } from '$lib/core/format/currency';
  import { suggestInstallmentOptions } from '$lib/features/plan/planUtils';
  import { onMount } from 'svelte';

  let { accounts = [] }: { accounts?: AccountBalanceView[] } = $props();

  let localAccounts = $state<AccountBalanceView[]>([]);

  onMount(async () => {
    if (accounts.length === 0) {
      localAccounts = await listAccountsCmd().catch(() => []);
    }
  });

  const effectiveAccounts = $derived(accounts.length > 0 ? accounts : localAccounts);

  const debtStats = $derived.by(() => {
    return effectiveAccounts
      .filter((item) => item.account.account_type === 'LIABILITY' && !item.account.placeholder)
      .map((item) => {
        return {
          id: item.account.id,
          name: item.account.name,
          balance: Math.abs(item.direct_balance),
          interestRate: 0,
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

<Card divided title={i18n.t.debtSimTitle}>
  {#snippet header()}
    <Tabs
      variant="outline"
      tabs={[
        { id: 'AVALANCHE', label: i18n.t.avalanche },
        { id: 'SNOWBALL', label: i18n.t.snowball },
      ]}
      active={strategy}
      onSelect={(id) => (strategy = id as 'AVALANCHE' | 'SNOWBALL')}
    />
  {/snippet}

  {#if debtStats.length === 0}
    <EmptyState title={i18n.t.noActiveLiabilityAccounts} hint={i18n.t.debtSimHint} />
  {:else}
    <div class="px-3 pt-2 pb-3">
      <div class="font-proto text-text-dim bg-bg-app border-line text-small mb-4 border p-3">
        {#if strategy === 'AVALANCHE'}
          <strong>{i18n.t.avalanche}:</strong> {i18n.t.debtSimAvalancheDesc}
        {:else}
          <strong>{i18n.t.snowball}:</strong> {i18n.t.debtSimSnowballDesc}
        {/if}
      </div>

      <div class="space-y-3">
        {#each selectedStrategy as debt, i (debt.id || debt)}
          <div
            class="border-line bg-bg-app relative flex flex-col gap-2 overflow-hidden border px-3 pt-2 pb-3"
          >
            {#if i === 0}
              <div class="bg-income absolute top-0 right-0 bottom-0 w-1"></div>
              <Badge size="m" tone="ok" class="absolute top-3 right-3">
                {i18n.t.targetDebt}
              </Badge>
            {/if}

            <div class="flex items-center gap-2">
              <div
                class="bg-expense/10 border-expense/30 text-expense font-proto text-smaller flex h-5 w-5 items-center justify-center border font-bold"
              >
                {i + 1}
              </div>
              <span class="font-proto text-text-strong text-small font-bold">{debt.name}</span>
            </div>

            <div class="mt-2 grid grid-cols-3 gap-2">
              <div>
                <span class="text-text-muted font-proto text-smaller block uppercase"
                  >{i18n.t.currentBalance}</span
                >
                <span class="font-proto text-expense text-small block font-bold"
                  >{formatIDR(debt.balance)}</span
                >
              </div>
              <div>
                <span class="text-text-muted font-proto text-smaller block uppercase"
                  >{i18n.t.interestRateApr}</span
                >
                <span
                  class="font-proto text-small block font-bold {debt.interestRate >= 15
                    ? 'text-expense'
                    : 'text-text-base'}">{debt.interestRate}%</span
                >
              </div>
              <div>
                <span class="text-text-muted font-proto text-smaller block uppercase"
                  >{i18n.t.debtSimEstMin}</span
                >
                <span
                  class="font-proto text-small block font-bold {i === 0
                    ? 'text-income'
                    : 'text-text-dim'}"
                >
                  {suggestInstallmentOptions(debt.balance, debt.interestRate).find(
                    (o) => o.count === 12
                  )
                    ? formatIDR(
                        suggestInstallmentOptions(debt.balance, debt.interestRate).find(
                          (o) => o.count === 12
                        )!.amount
                      )
                    : 0}
                </span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</Card>
