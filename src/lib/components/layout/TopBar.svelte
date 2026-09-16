<script lang="ts">
  import { resolve } from '$app/paths';
  import { session } from '$lib/core/state/session.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Badge, Icon } from '$lib/components/ui';
  import { formatIDR } from '$lib/core/format/currency';
  import { APP_NAME } from '$lib/core/types';

  interface HealthStats {
    balanced: boolean;
    unbalancedTxCount: number;
    discrepancy: number;
  }

  let {
    healthStats = null,
    onOpenHealthModal = () => {},
    onLock = () => {},
  }: {
    healthStats?: HealthStats | null;
    onOpenHealthModal?: () => void;
    onLock?: () => void;
  } = $props();
</script>

<header
  class="border-line bg-bg-app flex h-(--layout-topbar) shrink-0 items-center justify-between border-b px-3"
>
  <div class="flex items-center gap-2.5">
    <div class="font-proto text-small flex items-center font-bold tracking-widest uppercase">
      <span class="text-text-strong">{APP_NAME}</span>
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
      <button
        type="button"
        onclick={onOpenHealthModal}
        class="flex cursor-pointer items-center transition-colors focus-visible:outline-teal outline-offset-1"
        title={i18n.t.ledgerHealthTitle}
      >
        <Badge size="m" tone={hbOk ? 'ok' : 'err'}>
          <span class="size-1.5 animate-pulse {hbOk ? 'bg-income' : 'bg-expense'}"></span>
          {hbOk
            ? `${i18n.t.ledgerOkPrefix}${i18n.t.ledgerBalanced}`
            : `${i18n.t.ledgerWarnPrefix}${i18n.t.ledgerImbalance}: ${formatIDR(Math.abs(healthStats.discrepancy))}`}
        </Badge>
      </button>
    {/if}
  </div>

  <!-- Command Palette Quick Trigger -->
  <button
    type="button"
    onclick={() => modalState.toggleCommandPalette()}
    class="border-line bg-bg-btn hover:border-text-dim text-text-dim hover:text-text-strong font-proto text-smaller hidden md:flex h-6.5 items-center gap-2 border px-2.5 uppercase transition-colors cursor-pointer focus-visible:outline-teal outline-offset-1"
    title="Command Palette (Ctrl+K)"
  >
    <Icon name="search" size={12} />
    <span class="tracking-wider">COMMAND PALETTE</span>
    <span class="border-line bg-bg-card text-text-muted border px-1 text-[9px] font-bold">CTRL+K</span>
  </button>

  <div class="flex items-center gap-3">
    <button
      type="button"
      onclick={onLock}
      title={i18n.t.lockVault}
      class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong font-proto text-smaller inline-flex h-7 cursor-pointer items-center gap-1.5 border px-2 uppercase transition-colors focus-visible:outline-teal outline-offset-1"
      aria-label={i18n.t.lockVault}
    >
      <Icon name="lock" size={13} />
      <span class="hidden sm:inline">LOCK</span>
    </button>

    <!-- Notification Bell Button -->
    <button
      type="button"
      onclick={() => notificationState.toggleDrawer()}
      class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong relative flex h-7 min-w-7 cursor-pointer items-center justify-center gap-1 border px-1.5 transition-colors focus-visible:outline-teal outline-offset-1"
      title={i18n.t.notifications}
      aria-label={i18n.t.notifications}
    >
      <Icon name="bell" size={14} />
      {#if notificationState.unreadCount > 0}
        <span class="font-proto text-smaller text-expense font-bold tabular-nums">
          {notificationState.unreadCount}
        </span>
      {/if}
    </button>

    <!-- Settings Gear -->
    <a
      href={resolve('/app/setting')}
      title={i18n.t.settings}
      class="border-line hover:border-text-dim bg-bg-btn hover:bg-bg-card text-text-base hover:text-text-strong inline-flex h-7 w-7 items-center justify-center border transition-colors focus-visible:outline-teal outline-offset-1"
    >
      <Icon name="gear" size={14} />
    </a>
  </div>
</header>
