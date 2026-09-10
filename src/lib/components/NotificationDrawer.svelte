<script lang="ts">
  import {
    notifStore,
    type AppNotification,
    type NotificationType,
  } from '$lib/notifications/store.svelte';
  import { goto } from '$app/navigation';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import { getNotificationAccent } from '$lib/ui/theme';
  import { i18n } from '$lib/i18n.svelte';
  import { Icon, Button, CloseButton } from '$lib/components/ui';

  let filterTab = $state<'ALL' | 'DUE' | 'FX' | 'ACTIVITY'>('ALL');
  let confirmClearOpen = $state(false);

  const filtered = $derived(
    notifStore.notifications.filter((n) => {
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

  function getAccent(type: NotificationType): string {
    return getNotificationAccent(type);
  }

  function handleAction(notif: AppNotification) {
    notifStore.markAsRead(notif.id);
    notifStore.closeDrawer();
    if (notif.actionHref) {
      goto(notif.actionHref);
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

{#if notifStore.drawerOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div onclick={() => notifStore.closeDrawer()} class="bg-overlay-soft fixed inset-0 z-40"></div>

  <!-- Drawer Panel -->
  <aside
    class="border-line bg-bg-card anim-drawer fixed top-0 right-0 z-50 flex h-full w-95 flex-col border-l font-mono"
  >
    <!-- Header -->
    <div class="bg-bg-app flex items-center justify-between px-4 py-3">
      <div class="flex items-center gap-2">
        <span class="label-title">{i18n.t.notifications}</span>
        {#if notifStore.unreadCount > 0}
          <span class="bg-expense text-text-white px-1.5 py-0.5 text-[10px] font-bold">
            {notifStore.unreadCount}
          </span>
        {/if}
      </div>
      <div class="flex items-center gap-3">
        {#if notifStore.notifications.length > 0}
          <button
            type="button"
            onclick={() => notifStore.markAllAsRead()}
            class="text-teal text-[10px] hover:underline"
          >
            {i18n.t.markAllRead}
          </button>
          <button
            type="button"
            onclick={() => (confirmClearOpen = true)}
            class="text-text-muted hover:text-expense text-[10px]"
          >
            {i18n.t.clearAllNotifs}
          </button>
        {/if}
        <CloseButton onclick={() => notifStore.closeDrawer()} label={i18n.t.notifClose} />
      </div>
    </div>

    <!-- Filter Tabs -->
    <div class="bg-bg-app flex text-[10px]">
      <button
        type="button"
        onclick={() => (filterTab = 'ALL')}
        class="flex-1 py-2 text-center transition-colors {filterTab === 'ALL'
          ? 'text-teal border-teal bg-bg-card border-b-2'
          : 'text-text-muted hover:text-text-base'}"
      >
        {i18n.t.notifAll} ({notifStore.notifications.length})
      </button>
      <button
        type="button"
        onclick={() => (filterTab = 'DUE')}
        class="flex-1 py-2 text-center transition-colors {filterTab === 'DUE'
          ? 'text-expense border-expense bg-bg-card border-b-2'
          : 'text-text-muted hover:text-text-base'}"
      >
        {i18n.t.notifDue}
      </button>
      <button
        type="button"
        onclick={() => (filterTab = 'FX')}
        class="flex-1 py-2 text-center transition-colors {filterTab === 'FX'
          ? 'text-income border-income bg-bg-card border-b-2'
          : 'text-text-muted hover:text-text-base'}"
      >
        {i18n.t.notifFx}
      </button>
      <button
        type="button"
        onclick={() => (filterTab = 'ACTIVITY')}
        class="flex-1 py-2 text-center transition-colors {filterTab === 'ACTIVITY'
          ? 'text-warning border-warning bg-bg-card border-b-2'
          : 'text-text-muted hover:text-text-base'}"
      >
        {i18n.t.notifActivity}
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 space-y-2.5 overflow-y-auto p-3">
      {#each filtered as n (n.id)}
        {@const accent = getAccent(n.type)}
        <div
          class="bg-bg-app border-line hover:border-text-muted/60 border p-3 transition-colors {n.read
            ? 'opacity-70'
            : ''}"
          style="border-left: 3px solid {accent};"
        >
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-1.5">
              {#if !n.read}
                <span class="bg-teal size-1.5"></span>
              {/if}
              <p class="text-text-strong text-[11px] font-medium uppercase">
                {n.title}
              </p>
            </div>
            <span class="text-text-muted font-proto shrink-0 text-[9px]">
              {formatTime(n.timestamp)}
            </span>
          </div>

          <p class="text-text-base mt-1 text-[11px] leading-relaxed">
            {n.message}
          </p>

          {#if n.detail}
            <p class="text-text-muted mt-1 text-[10px] leading-normal">
              {n.detail}
            </p>
          {/if}

          <div class="border-line/40 mt-2.5 flex items-center justify-between border-t pt-1">
            <button
              type="button"
              onclick={() => notifStore.snoozeNotification(n.id)}
              class="text-text-muted hover:text-text-base font-proto inline-flex cursor-pointer items-center gap-1 text-[10px] uppercase transition-colors"
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
          <p class="text-[11px]">{i18n.t.noNotifsCategory}</p>
        </div>
      {/each}
    </div>

    <!-- Footer info -->
    <div class="border-line bg-bg-app text-text-muted border-t px-4 py-2.5 text-[10px]">
      {i18n.t.notifLocalPrivacy}
    </div>
  </aside>

  <ConfirmModal
    bind:open={confirmClearOpen}
    title={i18n.t.clearAllNotifsTitle}
    message={i18n.t.confirmClearNotifsMsg}
    confirmLabel={i18n.t.clearAllBtn}
    cancelLabel={i18n.t.cancelModalBtn}
    onConfirm={() => notifStore.clearAll()}
  />
{/if}
