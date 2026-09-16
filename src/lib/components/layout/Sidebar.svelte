<script lang="ts">
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import { i18n } from '$lib/core/i18n.svelte';
  import VaultPicker from '$lib/features/vault/components/VaultPicker.svelte';

  interface HealthStats {
    balanced: boolean;
    unbalancedTxCount: number;
    discrepancy: number;
  }

  let {
    healthStats = null,
    onOpenHealthModal = () => {},
  }: {
    healthStats?: HealthStats | null;
    onOpenHealthModal?: () => void;
  } = $props();

  const nav = $derived([
    { href: '/app', label: i18n.t.dashboard },
    { href: '/app/accounts', label: i18n.t.account },
    { href: '/app/journal', label: i18n.t.journal },
    { href: '/app/budget', label: i18n.t.budget },
    { href: '/app/reports', label: i18n.t.report },
    { href: '/app/reconcile', label: i18n.t.reconcile },
    { href: '/app/plan', label: i18n.t.plan },
    { href: '/app/audit', label: i18n.t.auditLogTitle },
    { href: '/app/setting', label: i18n.t.settings },
  ]);

  const isActive = (href: string) =>
    page.url.pathname === href || (href !== '/app' && page.url.pathname.startsWith(href));
</script>

<aside class="border-line bg-bg-app flex w-(--layout-sidebar) shrink-0 flex-col border-r">
  <div class="border-line relative z-30 shrink-0 border-b p-2">
    <VaultPicker />
  </div>

  <nav class="flex-1 space-y-1 overflow-y-auto px-2.5 py-2.5">
    {#each nav as item, idx (item.href)}
      {@const active = isActive(item.href)}
      <a
        href={resolve(item.href as '/app')}
        class="btn-nav text-small flex h-[32px] w-full items-center justify-between px-3 tracking-[0.06em] uppercase {active
          ? 'active font-medium'
          : ''}"
      >
        <span class="truncate">{item.label}</span>
        <span
          class="font-proto text-smaller {active
            ? 'text-teal font-bold'
            : 'text-text-muted/60'} tabular-nums transition-colors"
        >
          {idx + 1}
        </span>
      </a>
    {/each}
  </nav>

  {#if healthStats}
    <div class="border-line bg-bg-card/40 shrink-0 border-t p-3">
      <button
        type="button"
        onclick={onOpenHealthModal}
        class="group w-full cursor-pointer text-left focus-visible:outline-teal outline-offset-1"
      >
        <div class="mb-1 flex items-center justify-between">
          <span class="font-proto text-text-dim text-smaller font-bold tracking-wider uppercase">
            {i18n.t.ledgerHealthTitle}
          </span>
          <span
            class="size-1.5 {healthStats.balanced && healthStats.unbalancedTxCount === 0
              ? 'bg-income'
              : 'bg-expense'}"
          ></span>
        </div>
        <div class="font-proto text-text-muted text-smaller flex items-center justify-between">
          <span>{i18n.t.ledgerFormula}</span>
          {#if healthStats.unbalancedTxCount > 0}
            <span class="text-expense text-smaller font-bold">
              {i18n.t.unbalancedTransactionsCount.replace(
                '{count}',
                String(healthStats.unbalancedTxCount)
              )}
            </span>
          {/if}
        </div>
      </button>
    </div>
  {/if}
</aside>
