<script lang="ts">
  import { resolve } from '$app/paths';
  import { session } from '$lib/core/state/session.svelte';
  import { privacyState } from '$lib/core/state/privacy.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, Icon, AppBrand, Tooltip } from '$lib/components/ui';
  import { formatIDR } from '$lib/core/format/currency';

  interface HealthStats {
    balanced: boolean;
    unbalancedTxCount: number;
    discrepancy: number;
    assets: number;
    liabilities: number;
    equity: number;
    netIncome: number;
  }

  let {
    healthStats = null,
    onLock = () => {},
  }: {
    healthStats?: HealthStats | null;
    onLock?: () => void;
  } = $props();
</script>

<header
  class="border-line bg-bg-app relative z-40 flex h-(--layout-topbar) shrink-0 items-center justify-between border-b px-3"
>
  <div class="flex items-center gap-2.5">
    <div class="font-proto text-small flex items-center font-bold tracking-widest uppercase">
      <AppBrand variant="bar" />
      <span class="text-text-muted text-smaller mx-2">/</span>
      {#if session.raw?.vault_path}
        {@const parts = session.raw.vault_path
          .split(/[/\\]/)
          .filter((s) => !!s)
          .slice(-2)}
        {#each parts as p, i (i)}
          <span class="text-text-dim text-smaller">{p}</span>
          {#if i < parts.length - 1}
            <span class="text-text-muted text-smaller mx-1.5">/</span>
          {/if}
        {/each}
      {:else}
        <span class="text-text-dim text-smaller">
          vault-{session.currentVault?.toLowerCase().replace(/\s+/g, '-') ?? 'default'}
        </span>
      {/if}
    </div>

    {#if healthStats}
      {@const hbOk = healthStats.balanced && healthStats.unbalancedTxCount === 0}
      {@const tooltipContent = [
        `${i18n.t.totalAssetsLabel}: ${formatIDR(healthStats.assets)}`,
        `${i18n.t.liabilitiesAndDebt}: ${formatIDR(healthStats.liabilities)}`,
        `${i18n.t.equity}: ${formatIDR(healthStats.equity)}`,
        `${i18n.t.net}: ${formatIDR(healthStats.netIncome)}`,
      ].join(' · ')}
      <Tooltip
        content={privacyState.enabled ? i18n.t.ledgerHealthTitle : tooltipContent}
        placement="bottom"
        delay={200}
        class="flex items-center select-none"
      >
        <Badge size="m" tone={hbOk ? 'ok' : 'err'} class={hbOk ? '' : 'tabular-nums'}>
          <span class="size-1.5 animate-pulse {hbOk ? 'bg-income' : 'bg-expense'}"></span>
          {hbOk
            ? `${i18n.t.ledgerOkPrefix}${i18n.t.ledgerBalanced}`
            : `${i18n.t.ledgerWarnPrefix}${i18n.t.ledgerImbalance}: ${formatIDR(Math.abs(healthStats.discrepancy))}`}
        </Badge>
      </Tooltip>
    {/if}
  </div>

  <div class="flex items-center gap-3">
    <Tooltip content="{i18n.t.privacyMode} (Alt+P)" placement="bottom" delay={200}>
      <button
        type="button"
        onclick={() => privacyState.toggle()}
        class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong focus-visible:outline-teal inline-flex h-7 w-7 cursor-pointer items-center justify-center border outline-offset-1 transition-colors"
        aria-label={i18n.t.privacyMode}
      >
        <Icon name={privacyState.enabled ? 'eye-off' : 'eye'} size={14} />
      </button>
    </Tooltip>

    <Tooltip content={i18n.t.lockVault} placement="bottom" delay={200}>
      <button
        type="button"
        onclick={onLock}
        class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong font-proto text-smaller focus-visible:outline-teal inline-flex h-7 cursor-pointer items-center gap-1.5 border px-2 uppercase outline-offset-1 transition-colors"
        aria-label={i18n.t.lockVault}
      >
        <Icon name="lock" size={13} />
        <span class="hidden sm:inline">{i18n.t.lockVault}</span>
      </button>
    </Tooltip>

    <Tooltip content={i18n.t.notifications} placement="bottom" delay={200}>
      <button
        type="button"
        onclick={() => notificationState.toggleDrawer()}
        class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong focus-visible:outline-teal relative flex h-7 min-w-7 cursor-pointer items-center justify-center gap-1 border px-1.5 outline-offset-1 transition-colors"
        aria-label={i18n.t.notifications}
      >
        <Icon name="bell" size={14} />
        {#if notificationState.unreadCount > 0}
          <span class="font-proto text-smaller text-expense font-bold tabular-nums">
            {notificationState.unreadCount}
          </span>
        {/if}
      </button>
    </Tooltip>

    <Tooltip content={i18n.t.settings} placement="bottom" delay={200}>
      <a
        href={resolve('/app/setting')}
        class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong focus-visible:outline-teal inline-flex h-7 w-7 items-center justify-center border outline-offset-1 transition-colors"
        aria-label={i18n.t.settings}
      >
        <Icon name="gear" size={14} />
      </a>
    </Tooltip>
  </div>
</header>
