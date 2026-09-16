<script lang="ts">
  import type { AccountBalanceView, AccountType } from '$lib/core/ipc/bindings';
  import { formatMinorGrouping } from '$lib/core/format/currency';
  import { ACCOUNT_TYPES, accountTypeLabel } from '$lib/core/format/account';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Tabs, EmptyState, Card, Pagination, Badge } from '$lib/components/ui';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  let {
    searchQuery,
    searchScope,
    accounts = [],
  }: {
    searchQuery: string;
    searchScope: 'ALL' | 'ACCOUNT' | 'RECENT';
    accounts?: AccountBalanceView[];
  } = $props();

  function handleAccountClick(item: AccountBalanceView) {
    if (!item.account.placeholder) {
      goto(resolve('/app/accounts/[code]', { code: item.account.code }));
    } else {
      goto(resolve('/app/accounts'));
    }
  }

  let categoryFilter = $state<string>('ALL');
  let accPage = $state(1);
  const PAGE_SIZE = 10;

  const visibleAccounts = $derived(accounts.filter((i) => !i.account.hidden));

  const filteredAccounts = $derived(
    visibleAccounts
      .filter((i) => categoryFilter === 'ALL' || i.account.account_type === categoryFilter)
      .filter((i) => {
        if (!searchQuery.trim() || searchScope === 'RECENT') return true;
        const q = searchQuery.toLowerCase();
        return i.account.code.toLowerCase().includes(q) || i.account.name.toLowerCase().includes(q);
      })
  );

  const totalAccPages = $derived(Math.max(1, Math.ceil(filteredAccounts.length / PAGE_SIZE)));

  $effect(() => {
    void searchQuery;
    void categoryFilter;
    void searchScope;
    accPage = 1;
  });

  const pagedAccounts = $derived(
    filteredAccounts.slice((accPage - 1) * PAGE_SIZE, accPage * PAGE_SIZE)
  );
</script>

<Card title={i18n.t.account} count={filteredAccounts.length} class="col-span-8 h-full">
  {#snippet header()}
    <div class="flex min-w-0 flex-1 items-center justify-end gap-3">
      <Tabs
        variant="outline"
        tabs={[
          { id: 'ALL', label: i18n.t.filterAllLabel },
          ...ACCOUNT_TYPES.map((t: AccountType) => ({
            id: t,
            label: accountTypeLabel(t).toUpperCase(),
          })),
        ]}
        active={categoryFilter}
        onSelect={(id) => {
          categoryFilter = id;
        }}
        class="max-w-full overflow-x-auto"
      />
      <span
        class="text-text-dim font-proto text-smaller ml-auto shrink-0 tracking-widest uppercase tabular-nums"
      >
        {i18n.t.showingOf
          .replace('{shown}', String(pagedAccounts.length))
          .replace('{total}', String(filteredAccounts.length))}
      </span>
    </div>
  {/snippet}

  <div class="mt-2 min-h-0 flex-1 overflow-y-auto pr-3">
    {#if pagedAccounts.length === 0}
      <EmptyState
        title={i18n.t.noMatchingAccounts}
        actionLabel={i18n.t.newAccount}
        actionHref="/app/accounts"
      />
    {:else}
      <div class="flex flex-col gap-1.5 pr-1">
        {#each pagedAccounts as item (item.account.id)}
          <div
            role="button"
            tabindex="0"
            onclick={() => handleAccountClick(item)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleAccountClick(item);
              }
            }}
            class="group hover:bg-bg-btn flex cursor-pointer items-center gap-2 px-1.5 py-0.5 transition-colors first:pt-0"
          >
            <span class="font-proto text-text-muted text-smaller w-8 shrink-0"
              >{item.account.code}</span
            >
            <span class="font-proto text-text-base text-smaller shrink-0">{item.account.name}</span>
            {#if item.account.placeholder}
              <Badge size="s" tone="neutral" class="shrink-0">{i18n.t.badgePh}</Badge>
            {/if}

            <div
              class="border-line/80 group-hover:border-line mb-0.5 flex-1 border-b border-dotted transition-colors"
            ></div>

            <span
              class="font-proto text-text-strong text-smaller shrink-0 font-medium tabular-nums"
            >
              {formatMinorGrouping(Math.abs(item.recursive_balance), item.account.currency)}
            </span>
            <span class="font-proto text-text-dim text-smaller w-7 shrink-0 text-right"
              >{item.account.currency}</span
            >
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="border-line/40 mt-auto flex shrink-0 items-center border-t pt-1.5">
    <Pagination
      bind:currentPage={accPage}
      totalPages={totalAccPages}
      layout="spread"
      class="flex-1"
    />
  </div>
</Card>
