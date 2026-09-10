<script lang="ts">
  import { onMount } from 'svelte';
  import { ledger } from '$lib/accounting/store.svelte';
  import { notifStore } from '$lib/notifications/store.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { DEFAULT_FX_RATE } from '$lib/accounting/types';
  import { formatIDR } from '$lib/accounting/finance';
  import { Card } from '$lib/components/ui';

  let syncing = $state(false);
  let syncMsg = $state<{ text: string; type: 'ok' | 'err' } | null>(null);
  let lastSyncStr = $state<string>('');
  let isOnline = $state(typeof navigator !== 'undefined' ? navigator.onLine : true);

  const currentVaultRate = $derived(ledger.data?.fxRate ?? DEFAULT_FX_RATE);

  // Previous rate from history if available
  const previousRate = $derived.by(() => {
    const history = ledger.data?.fxHistory ?? [];
    if (history.length >= 2) {
      return history[history.length - 2].rate;
    }
    return currentVaultRate;
  });

  const rateDiff = $derived(currentVaultRate - previousRate);
  const ratePctChange = $derived(previousRate > 0 ? (rateDiff / previousRate) * 100 : 0);

  // Recent snapshots (last 5)
  const recentHistory = $derived.by(() => {
    const history = ledger.data?.fxHistory ?? [];
    return [...history].reverse().slice(0, 5);
  });

  function refreshSyncTime() {
    if (typeof localStorage === 'undefined') return;
    const stored = localStorage.getItem('finnca_last_fx_sync');
    if (!stored) {
      lastSyncStr = i18n.t.fxNeverSynced;
      return;
    }
    const d = new Date(stored);
    lastSyncStr = d.toLocaleDateString(i18n.locale === 'id' ? 'id-ID' : 'en-US', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  async function handleOneShotSync() {
    if (syncing) return;
    syncing = true;
    syncMsg = null;
    try {
      const res = await notifStore.checkLiveFxRate(ledger.data, true);
      refreshSyncTime();
      if (res.success) {
        syncMsg = { text: i18n.t.fxSyncSuccess, type: 'ok' };
      } else {
        syncMsg = { text: i18n.t.fxSyncFailed, type: 'err' };
      }
    } catch (e: any) {
      syncMsg = { text: e.message || i18n.t.fxSyncFailed, type: 'err' };
    } finally {
      syncing = false;
      setTimeout(() => {
        syncMsg = null;
      }, 4000);
    }
  }

  onMount(() => {
    refreshSyncTime();
    const handleOnline = () => {
      isOnline = true;
    };
    const handleOffline = () => {
      isOnline = false;
    };
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  });
</script>

<div class="grid grid-cols-1 gap-2 select-none lg:grid-cols-2">
  <!-- LEFT COLUMN: LIVE FX & ONE-SHOT SYNC -->
  <div class="flex flex-col gap-2">
    <!-- HERO FX CARD -->
    <Card title={i18n.t.usdExchangeRateTitle} class="gap-2">
      {#snippet header()}
        <span
          class="font-proto text-text-muted border-line bg-bg-app border px-1.5 py-0.5 text-[9px] tracking-widest uppercase leading-none"
        >
          {i18n.t.fxOneShotBadge}
        </span>
        <div class="font-proto flex items-center gap-2 text-[11px] leading-none">
          <span class="size-2 rounded-none {isOnline ? 'bg-income' : 'bg-warn'} animate-pulse"
          ></span>
          <span class="{isOnline ? 'text-income' : 'text-warn'} text-[10px] tracking-wider leading-none">
            {isOnline ? i18n.t.fxOnlineStatus : i18n.t.fxOfflineStatus}
          </span>
        </div>
      {/snippet}

      <div class="mt-1 flex items-start justify-between gap-4">
        <div>
          <p class="font-proto text-text-dim mb-1 text-[10px] tracking-widest uppercase">
            {i18n.t.fxOneShotTitle}
          </p>
          <div class="flex flex-wrap items-baseline gap-2.5">
            <span class="font-proto text-text-white text-xl font-bold tracking-tight">
              1 USD = {formatIDR(currentVaultRate)}
            </span>
            {#if rateDiff !== 0}
              <span
                class="font-proto border px-2 py-0.5 text-[11px] font-semibold {rateDiff > 0
                  ? 'border-income/40 text-income bg-income/10'
                  : 'border-expense/40 text-expense bg-expense/10'}"
              >
                {rateDiff > 0 ? '▲ +' : '▼ '}{formatIDR(Math.abs(rateDiff))} ({rateDiff > 0
                  ? '+'
                  : ''}{ratePctChange.toFixed(2)}%)
              </span>
            {/if}
          </div>
        </div>

        <button
          type="button"
          onclick={handleOneShotSync}
          disabled={syncing}
          class="sharp-btn btn-primary font-proto inline-flex shrink-0 items-center gap-2 px-3.5 py-2 text-[10px] disabled:opacity-50"
        >
          {#if syncing}
            <span class="spinner-sm"></span>
            <span>{i18n.t.processingBtn}</span>
          {:else}
            <span class="text-[12px] leading-none">↻</span>
            <span>{i18n.t.syncNowBtn}</span>
          {/if}
        </button>
      </div>

      <!-- Metadata Row -->
      <div
        class="border-line/40 font-proto text-text-muted mt-auto flex items-center justify-between border-t pt-2.5 text-[10px]"
      >
        <span>
          {i18n.t.fxLastSynced}: <strong class="text-text-base">{lastSyncStr}</strong>
        </span>
        <span class="text-text-dim">
            {i18n.t.fxSyncFrequency}
          </span>
      </div>

      {#if syncMsg}
        <div
          class="font-proto border px-3 py-1.5 text-[11px] {syncMsg.type === 'ok'
            ? 'border-income/50 text-income bg-income/10'
            : 'border-expense/50 text-expense bg-expense/10'}"
        >
          {syncMsg.text}
        </div>
      {/if}
    </Card>

    <!-- FX ALERTS & POLICY CARD -->
    <Card title={i18n.t.notifFxAlertsLabel} badge="ACTIVE" badgeTone="ok" class="gap-2.5">
      <p class="text-text-base font-mono mt-1 text-[11px] leading-relaxed">
        {i18n.t.fxDailyNotificationInfo}
      </p>

      <p class="text-text-muted border-line/30 border-t pt-2 font-mono text-[10px] leading-relaxed">
        {i18n.t.fxOneShotDesc}
      </p>
    </Card>
  </div>

  <!-- RIGHT COLUMN: AUDIT LOG & RECENT MOVEMENTS -->
  <Card title={i18n.t.fxHistoryTitle} badge="AUDIT LOG" class="justify-between">
    <div>
      {#if recentHistory.length > 1}
        <div class="font-proto flex flex-col gap-1.5 text-[11px]">
          {#each recentHistory as item, idx (item.date)}
            {@const next = recentHistory[idx + 1]}
            {@const diff = next ? item.rate - next.rate : 0}
            <div
              class="border-line/30 bg-bg-app hover:border-line flex items-center justify-between border p-2.5 transition-colors"
            >
              <span class="text-text-muted font-mono">{item.date}</span>
              <div class="flex items-center gap-2">
                <span class="text-text-white font-bold">{formatIDR(item.rate)}</span>
                {#if diff !== 0}
                  <span
                    class="text-[10px] {diff > 0 ? 'text-income' : 'text-expense'} w-24 text-right"
                  >
                    {diff > 0 ? '▲ +' : '▼ '}{formatIDR(Math.abs(diff))}
                  </span>
                {:else}
                  <span class="text-text-dim w-24 text-right text-[10px]">—</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div
          class="border-line/60 bg-bg-app flex flex-col items-center gap-2 border border-dashed p-6 text-center"
        >
          <span class="font-proto text-text-dim text-[10px] tracking-wider uppercase">
            {i18n.t.fxNoHistoryTitle}
          </span>
          <p class="text-text-muted max-w-xs font-mono text-[10px]">
            {i18n.t.fxNoHistoryDesc}
          </p>
        </div>
      {/if}
    </div>

    <p class="text-text-dim border-line/30 mt-auto border-t pt-2 font-mono text-[9px]">
      {i18n.t.fxConversionNote}
    </p>
  </Card>
</div>
