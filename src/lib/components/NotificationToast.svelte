<script lang="ts">
  import { notifStore, type AppNotification } from '$lib/notifications/store.svelte';
  import { goto } from '$app/navigation';
  import { getNotificationAccent } from '$lib/ui/theme';
  import { i18n } from '$lib/i18n.svelte';
  import { Icon, Button, CloseButton } from '$lib/components/ui';

  function getAccentColor(type: AppNotification['type']): string {
    return getNotificationAccent(type);
  }
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
      default:
        return 'bell';
    }
  }

  function handleAction(notif: AppNotification) {
    notifStore.markAsRead(notif.id);
    notifStore.dismissToast(notif.id);
    if (notif.actionHref) {
      goto(notif.actionHref);
    }
  }
</script>

<div class="pointer-events-none fixed top-12 right-6 z-50 flex max-w-95 flex-col gap-2.5 font-mono">
  {#each notifStore.activeToasts as notif (notif.id)}
    {@const accent = getAccentColor(notif.type)}
    <div
      class="anim-toast bg-bg-card border-line pointer-events-auto border p-3.5"
      style="border-left: 3px solid {accent};"
    >
      <div class="flex items-start justify-between gap-2.5">
        <div class="flex items-center gap-2">
          <span class="text-text-strong inline-flex"
            ><Icon name={getIconName(notif.type)} size={14} /></span
          >
          <p class="text-text-strong text-[12px] font-medium tracking-wide uppercase">
            {notif.title}
          </p>
        </div>
        <CloseButton
          onclick={() => notifStore.dismissToast(notif.id)}
          label={i18n.t.notifClose}
        />
      </div>

      <p class="text-text-base mt-1 text-[11px] leading-relaxed">
        {notif.message}
      </p>

      {#if notif.detail}
        <p class="text-text-muted mt-1 text-[10px] leading-normal">
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
