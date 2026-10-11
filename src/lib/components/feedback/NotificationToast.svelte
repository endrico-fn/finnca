<script lang="ts">
  import { onMount } from 'svelte';
  import {
    notificationState,
    type AppNotification,
    getNotificationBorderClass,
  } from '$lib/core/state/notification.svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { i18n } from '$lib/core/i18n.svelte';
  import { Icon, Button, CloseButton } from '$lib/components/ui';

  let dismissingIds = $state<Set<string>>(new Set());

  function scheduleDismiss(id: string) {
    dismissingIds = new Set([...dismissingIds, id]);
    setTimeout(() => {
      notificationState.dismissToast(id);
      dismissingIds = new Set([...dismissingIds].filter((x) => x !== id));
    }, 90);
  }

  onMount(() => {
    notificationState.dismissCallback = scheduleDismiss;
    return () => {
      notificationState.dismissCallback = null;
    };
  });

  function getIconName(
    type: AppNotification['type']
  ): 'calendar' | 'chart' | 'alert' | 'wallet' | 'bell' {
    switch (type) {
      case 'DUE_DATE':
        return 'calendar';
      case 'FX_ALERT':
        return 'chart';
      case 'EXPENSE_SPIKE':
        return 'alert';
      case 'CASHFLOW_DEFICIT':
        return 'wallet';
      case 'LEDGER_INTEGRITY':
        return 'alert';
      case 'GREETING':
        return 'bell';
      default:
        return 'bell';
    }
  }

  import { openUrl } from '@tauri-apps/plugin-opener';
  import { updaterState } from '$lib/core/updater/updaterState.svelte';

  async function handleAction(notif: AppNotification) {
    notificationState.markAsRead(notif.id);
    scheduleDismiss(notif.id);
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
</script>

<div
  class="pointer-events-none fixed top-12 right-6 z-[var(--z-toast)] flex max-w-95 flex-col gap-2.5"
>
  {#each notificationState.activeToasts as notif (notif.id)}
    {@const borderClass = getNotificationBorderClass(notif.type)}
    <div
      class="{dismissingIds.has(notif.id)
        ? 'anim-toast-exit'
        : 'anim-toast'} bg-bg-card border-line {borderClass} pointer-events-auto border border-l-2 p-3.5"
    >
      <div class="flex items-start justify-between gap-2.5">
        <div class="flex items-center gap-2">
          <span class="text-text-strong inline-flex"
            ><Icon name={getIconName(notif.type)} size={14} /></span
          >
          <p class="text-text-strong text-small font-proto font-medium tracking-wide uppercase">
            {notif.title}
          </p>
        </div>
        <CloseButton onclick={() => scheduleDismiss(notif.id)} label={i18n.t.notifClose} />
      </div>

      <p class="text-text-base text-small mt-1 leading-relaxed tabular-nums">
        {notif.message}
      </p>

      {#if notif.detail}
        <p class="text-text-muted text-smaller mt-1 leading-normal tabular-nums">
          {notif.detail}
        </p>
      {/if}

      {#if notif.actionHref && notif.actionLabel}
        <div class="mt-2.5 flex justify-end">
          <Button variant="ghost" onclick={() => handleAction(notif)}>
            {notif.actionLabel}
          </Button>
        </div>
      {/if}
    </div>
  {/each}
</div>
