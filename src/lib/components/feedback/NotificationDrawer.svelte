<script lang="ts">
  import {
    notificationState,
    type AppNotification,
    getNotificationBorderClass,
  } from '$lib/core/state/notification.svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import ConfirmDialog from '$lib/components/feedback/ConfirmDialog.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button, Badge, CloseButton, Tabs } from '$lib/components/ui';

  let filterTab = $state<'ALL' | 'DUE' | 'FX' | 'ACTIVITY'>('ALL');
  let confirmClearOpen = $state(false);

  const tabCounts = $derived.by(() => {
    const all = notificationState.notifications;
    return {
      ALL: all.length,
      DUE: all.filter((n) => n.type === 'DUE_DATE').length,
      FX: all.filter((n) => n.type === 'FX_ALERT').length,
      ACTIVITY: all.filter((n) =>
        ['EXPENSE_SPIKE', 'CASHFLOW_DEFICIT', 'INACTIVITY', 'LEDGER_INTEGRITY'].includes(n.type)
      ).length,
    };
  });

  const filtered = $derived(
    notificationState.notifications.filter((n) => {
      if (filterTab === 'DUE') return n.type === 'DUE_DATE';
      if (filterTab === 'FX') return n.type === 'FX_ALERT';
      if (filterTab === 'ACTIVITY')
        return (
          n.type === 'EXPENSE_SPIKE' ||
          n.type === 'CASHFLOW_DEFICIT' ||
          n.type === 'INACTIVITY' ||
          n.type === 'LEDGER_INTEGRITY'
        );
      return true;
    })
  );

  import { openUrl } from '@tauri-apps/plugin-opener';
  import { updaterState } from '$lib/core/updater/updaterState.svelte';

  async function handleAction(notif: AppNotification) {
    notificationState.markAsRead(notif.id);
    notificationState.closeDrawer();
    if (notif.actionHref) {
      if (notif.actionHref === 'app:update') {
        updaterState.openScreen();
        updaterState.checkForUpdate();
      } else if (
        notif.actionHref.startsWith('http://') ||
        notif.actionHref.startsWith('https://')
      ) {
        try {
          await openUrl(notif.actionHref);
        } catch {
          window.open(notif.actionHref, '_blank');
        }
      } else {
        goto(resolve(notif.actionHref as '/app'));
      }
    }
  }

  function formatTime(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return '';
    }
  }
</script>

{#if notificationState.drawerOpen}
  <button
    type="button"
    aria-label={i18n.t.closeBtn}
    onclick={() => notificationState.closeDrawer()}
    class="bg-overlay fixed inset-0 z-40 cursor-default"
  ></button>

  <aside
    class="border-line bg-bg-card anim-drawer fixed top-0 right-0 z-50 flex h-full w-95 flex-col border-l"
  >
    <div class="bg-bg-app flex items-center justify-between px-4 py-3">
      <div class="flex items-center gap-2">
        <span class="label-title">{i18n.t.notifications}</span>
        {#if notificationState.unreadCount > 0}
          <Badge size="m" tone="err">{notificationState.unreadCount}</Badge>
        {/if}
      </div>
      <div class="flex items-center gap-3">
        {#if notificationState.notifications.length > 0}
          <button
            type="button"
            onclick={() => notificationState.markAllAsRead()}
            class="text-teal text-smaller font-proto hover:underline"
          >
            {i18n.t.markAllRead}
          </button>
          <button
            type="button"
            onclick={() => (confirmClearOpen = true)}
            class="text-text-muted hover:text-expense text-smaller font-proto"
          >
            {i18n.t.clearAllNotifs}
          </button>
        {/if}
        <CloseButton onclick={() => notificationState.closeDrawer()} label={i18n.t.notifClose} />
      </div>
    </div>

    <div class="bg-bg-app border-line border-b px-3 py-2">
      <Tabs
        variant="pill"
        fullWidth
        tabs={[
          { id: 'ALL', label: i18n.t.notifAll, count: tabCounts.ALL },
          { id: 'DUE', label: i18n.t.notifDue, count: tabCounts.DUE },
          { id: 'FX', label: i18n.t.notifFx, count: tabCounts.FX },
          { id: 'ACTIVITY', label: i18n.t.notifActivity, count: tabCounts.ACTIVITY },
        ]}
        active={filterTab}
        onSelect={(id) => (filterTab = id as typeof filterTab)}
      />
    </div>

    <div class="flex-1 space-y-2.5 overflow-y-auto p-3">
      {#each filtered as n (n.id)}
        {@const borderClass = getNotificationBorderClass(n.type)}
        <div
          class="bg-bg-app border-line {borderClass} hover:border-text-muted/60 border border-l-2 p-3 transition-colors {n.read
            ? 'opacity-70'
            : ''}"
        >
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-1.5">
              {#if !n.read}
                <span class="bg-teal size-1.5"></span>
              {/if}
              <p class="text-text-strong text-small font-proto font-medium uppercase">
                {n.title}
              </p>
            </div>
            <span class="text-text-muted font-proto text-smaller shrink-0">
              {formatTime(n.timestamp)}
            </span>
          </div>

          <p class="text-text-base text-small mt-1 leading-relaxed">
            {n.message}
          </p>

          {#if n.detail}
            <p class="text-text-muted text-smaller mt-1 leading-normal">
              {n.detail}
            </p>
          {/if}

          <div class="border-line/40 mt-2.5 flex items-center justify-between border-t pt-1">
            <button
              type="button"
              onclick={() => notificationState.snoozeNotification(n.id)}
              class="text-text-muted hover:text-text-base font-proto text-smaller inline-flex cursor-pointer items-center gap-1 uppercase transition-colors"
              title={i18n.t.snoozeNotifBtn}
            >
              <Icon name="clock" size={10} />
              <span>{i18n.t.snoozeNotifBtn}</span>
            </button>
            {#if n.actionHref && n.actionLabel}
              <Button variant="ghost" size="sm" onclick={() => handleAction(n)}>
                {n.actionLabel}
              </Button>
            {/if}
          </div>
        </div>
      {:else}
        <div
          class="text-text-muted flex h-full flex-col items-center justify-center gap-2 p-8 text-center"
        >
          <span class="text-text-muted inline-flex"><Icon name="bell" size={22} /></span>
          <p class="text-small font-proto">{i18n.t.noNotifsCategory}</p>
        </div>
      {/each}
    </div>

    <!-- Footer info -->
    <div class="border-line bg-bg-app text-text-muted text-smaller font-proto border-t px-4 py-2.5">
      {i18n.t.notifLocalPrivacy}
    </div>
  </aside>

  <ConfirmDialog
    bind:open={confirmClearOpen}
    title={i18n.t.clearAllNotifsTitle}
    message={i18n.t.confirmClearNotifsMsg}
    confirmLabel={i18n.t.clearAllBtn}
    cancelLabel={i18n.t.cancelModalBtn}
    onConfirm={() => notificationState.clearAll()}
  />
{/if}
