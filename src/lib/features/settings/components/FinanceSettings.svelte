<script lang="ts">
  import { onMount } from 'svelte';
  import { fxState } from '$lib/features/settings/state/settings.svelte';
  import { closingBooksState } from '$lib/core/state/ledgerLock.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import { fetchLiveFxRate } from '$lib/features/settings/fxSync';
  import { i18n } from '$lib/core/i18n.svelte';
  import { getPref, setPref } from '$lib/core/state/prefs';
  import { formatIDR } from '$lib/core/format/currency';
  import { todayString } from '$lib/core/format/date';
  import { Badge, Card, Button } from '$lib/components/ui';

  let syncing = $state(false);
  let syncMsg = $state<{ text: string; type: 'ok' | 'err' } | null>(null);
  let lastSyncStr = $state<string>('');
  let isOnline = $state(typeof navigator !== 'undefined' ? navigator.onLine : true);
  let selectedLockDate = $state(todayString());

  const currentVaultRate = $derived(fxState.rate);

  const previousRate = $derived.by(() => {
    const history = fxState.history;
    if (history.length >= 2) {
      return history[history.length - 2].rate;
    }
    return currentVaultRate;
  });

  const rateDiff = $derived(currentVaultRate - previousRate);
  const ratePctChange = $derived(previousRate > 0 ? (rateDiff / previousRate) * 100 : 0);

  const recentHistory = $derived.by(() => {
    return [...fxState.history].reverse().slice(0, 5);
  });

  function refreshSyncTime() {
    const stored = getPref('finnca_last_fx_sync', '');
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
      const liveRate = await fetchLiveFxRate();
      setPref('finnca_last_fx_sync', new Date().toISOString());
      refreshSyncTime();
      if (liveRate !== null) {
        fxState.setRate(liveRate);
        syncMsg = { text: i18n.t.fxSyncSuccess, type: 'ok' };
      } else {
        syncMsg = { text: i18n.t.fxSyncFailed, type: 'err' };
      }
    } catch (e: unknown) {
      syncMsg = {
        text: (e instanceof Error ? e.message : String(e)) || i18n.t.fxSyncFailed,
        type: 'err',
      };
    } finally {
      syncing = false;
      setTimeout(() => {
        syncMsg = null;
      }, 4000);
    }
  }

  onMount(() => {
    refreshSyncTime();
    closingBooksState.load();
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

  function promptLockBooks() {
    if (!selectedLockDate) return;
    modalState.confirm({
      title: i18n.t.confirmLockDateTitle,
      message: i18n.t.confirmLockDateMsg.replace('{date}', selectedLockDate),
      confirmLabel: i18n.t.lockDateAction,
      cancelLabel: i18n.t.cancelBtn,
      danger: true,
      onConfirm: async () => {
        await closingBooksState.setClosingDate(selectedLockDate);
        notificationState.addNotification({
          type: 'INFO',
          priority: 'low',
          title: i18n.t.closingBooksTitle,
          message: i18n.t.closingDateUpdatedSuccess,
        });
      },
    });
  }

  function promptUnlockBooks() {
    modalState.confirm({
      title: i18n.t.confirmUnlockTitle,
      message: i18n.t.confirmUnlockMsg,
      confirmLabel: i18n.t.unlockDateAction,
      cancelLabel: i18n.t.cancelBtn,
      danger: true,
      onConfirm: async () => {
        await closingBooksState.setClosingDate(null);
        notificationState.addNotification({
          type: 'INFO',
          priority: 'low',
          title: i18n.t.closingBooksTitle,
          message: i18n.t.closingDateUpdatedSuccess,
        });
      },
    });
  }
</script>

<div class="grid grid-cols-1 gap-4 select-none lg:grid-cols-2">
  <div class="flex flex-col gap-2">
    <Card title={i18n.t.usdExchangeRateTitle} class="gap-2">
      {#snippet header()}
        <Badge size="m" tone="neutral">{i18n.t.fxOneShotBadge}</Badge>
        <div class="font-proto text-small flex items-center gap-2 leading-none">
          <span class="size-2 {isOnline ? 'bg-income' : 'bg-warning'} animate-pulse"></span>
          <span
            class="{isOnline
              ? 'text-income'
              : 'text-warning'} text-smaller leading-none tracking-wider"
          >
            {isOnline ? i18n.t.fxOnlineStatus : i18n.t.fxOfflineStatus}
          </span>
        </div>
      {/snippet}

      <div class="mt-1 flex items-start justify-between gap-4">
        <div>
          <p class="font-proto text-text-dim text-smaller mb-1 tracking-widest uppercase">
            {i18n.t.fxOneShotTitle}
          </p>
          <div class="flex flex-wrap items-baseline gap-2.5">
            <span class="font-proto text-text-strong text-large font-bold tracking-tight">
              {i18n.t.fxRateDisplay.replace('{rate}', formatIDR(currentVaultRate))}
            </span>
            {#if rateDiff !== 0}
              <Badge size="m" tone={rateDiff > 0 ? 'ok' : 'err'}>
                {rateDiff > 0 ? '▲ +' : '▼ '}{formatIDR(Math.abs(rateDiff))} ({rateDiff > 0
                  ? '+'
                  : ''}{ratePctChange.toFixed(2)}%)
              </Badge>
            {/if}
          </div>
        </div>

        <Button
          variant="primary"
          class="font-proto text-small h-8 shrink-0 px-3 font-bold tracking-wider"
          disabled={syncing}
          onclick={handleOneShotSync}
        >
          {i18n.t.syncNowBtn}
        </Button>
      </div>

      <div
        class="border-line/40 font-proto text-text-muted text-smaller mt-auto flex items-center justify-between border-t pt-2.5"
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
          class="font-proto text-small border px-3 py-1.5 {syncMsg.type === 'ok'
            ? 'border-income/50 text-income bg-income/10'
            : 'border-expense/50 text-expense bg-expense/10'}"
        >
          {syncMsg.text}
        </div>
      {/if}
    </Card>

    <Card
      title={i18n.t.notifFxAlertsLabel}
      badge={i18n.t.activeStatusWord}
      badgeTone="ok"
      class="gap-2.5"
    >
      <p class="text-text-base text-small font-aux mt-1 leading-relaxed">
        {i18n.t.fxDailyNotificationInfo}
      </p>

      <p class="text-text-muted border-line/30 text-smaller font-aux border-t pt-2 leading-relaxed">
        {i18n.t.fxOneShotDesc}
      </p>
    </Card>
  </div>

  <Card title={i18n.t.fxHistoryTitle} badge={i18n.t.fxAuditLogBadge} class="justify-between">
    <div>
      {#if recentHistory.length > 1}
        <div class="font-proto text-small flex flex-col gap-1.5">
          {#each recentHistory as item, idx (item.date)}
            {@const next = recentHistory[idx + 1]}
            {@const diff = next ? item.rate - next.rate : 0}
            <div
              class="border-line/30 bg-bg-app hover:border-line flex items-center justify-between border p-2.5 transition-colors"
            >
              <span class="text-text-muted font-proto tabular-nums">{item.date}</span>
              <div class="flex items-center gap-2">
                <span class="text-text-white font-proto font-bold tabular-nums"
                  >{formatIDR(item.rate)}</span
                >
                {#if diff !== 0}
                  <span
                    class="text-smaller font-proto tabular-nums {diff > 0
                      ? 'text-income'
                      : 'text-expense'} w-24 text-right"
                  >
                    {diff > 0 ? '▲ +' : '▼ '}{formatIDR(Math.abs(diff))}
                  </span>
                {:else}
                  <span class="text-text-dim text-smaller font-proto w-24 text-right">—</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div
          class="border-line/60 bg-bg-app flex flex-col items-center gap-2 border border-dashed p-6 text-center"
        >
          <span class="font-proto text-text-dim text-smaller tracking-wider uppercase">
            {i18n.t.fxNoHistoryTitle}
          </span>
          <p class="text-text-muted text-smaller font-aux max-w-xs">
            {i18n.t.fxNoHistoryDesc}
          </p>
        </div>
      {/if}
    </div>

    <p class="text-text-dim border-line/30 text-smaller font-aux mt-auto border-t pt-2">
      {i18n.t.fxConversionNote}
    </p>
  </Card>

  <Card
    title={i18n.t.closingBooksTitle}
    description={i18n.t.closingBooksDesc}
    badge={closingBooksState.closingDate ? i18n.t.lockedPeriodBadge : i18n.t.openPeriodBadge}
    badgeTone={closingBooksState.closingDate ? 'warn' : 'ok'}
    class="gap-3 lg:col-span-2"
  >
    <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
      <div class="flex flex-col gap-1">
        <span class="font-proto text-text-dim text-smaller tracking-wider uppercase">
          {i18n.t.currentLockDate}
        </span>
        <div class="flex items-center gap-2">
          {#if closingBooksState.closingDate}
            <span class="font-proto text-text-white text-medium font-bold tabular-nums">
              {closingBooksState.closingDate}
            </span>
            <Badge size="m" tone="warn">
              {i18n.t.lockedPeriodBadge}
            </Badge>
          {:else}
            <span class="font-proto text-text-muted text-small">
              {i18n.t.noClosingDateSet}
            </span>
            <Badge size="m" tone="ok">
              {i18n.t.openPeriodBadge}
            </Badge>
          {/if}
        </div>
        <p class="font-aux text-text-muted text-smaller mt-0.5 max-w-lg">
          {i18n.t.confirmLockDateMsg.replace(
            '{date}',
            closingBooksState.closingDate || selectedLockDate
          )}
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <input
          type="date"
          bind:value={selectedLockDate}
          class="sharp-input font-proto text-small h-8 px-2"
        />
        <Button
          variant="primary"
          class="font-proto text-small h-8 px-3 font-bold tracking-wider"
          disabled={closingBooksState.saving || !selectedLockDate}
          onclick={promptLockBooks}
        >
          {i18n.t.lockDateAction}
        </Button>
        {#if closingBooksState.closingDate}
          <Button
            variant="ghost"
            class="font-proto text-small text-expense hover:bg-expense/10 h-8 px-3 tracking-wider"
            disabled={closingBooksState.saving}
            onclick={promptUnlockBooks}
          >
            {i18n.t.unlockDateAction}
          </Button>
        {/if}
      </div>
    </div>
  </Card>
</div>
