<script lang="ts">
  import '../app.css';
  import { APP_NAME } from '$lib/core/types';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { session } from '$lib/core/state/session.svelte';
  import { eventBus } from '$lib/core/events/eventBus.svelte';
  import { i18n } from '$lib/core/i18n.svelte';
  import { checkAppUpdates } from '$lib/core/updater/updateChecker';
  import { updaterState } from '$lib/core/updater/updaterState.svelte';
  import { diagnosticsState } from '$lib/core/state/diagnostics.svelte';
  import { getRollbackNoticeCmd } from '$lib/core/ipc/bindings';
  import { notificationState } from '$lib/core/state/notification.svelte';
  import NotificationToast from '$lib/components/feedback/NotificationToast.svelte';
  import UpdateScreen from '$lib/features/updater/components/UpdateScreen.svelte';

  let { children } = $props();

  onMount(() => {
    // 1. Immediately evaluate application updates on startup
    checkAppUpdates(i18n.t);

    let unlistenImport: (() => void) | undefined;
    listen<string>('finnca:import-file', (event) => {
      const filePath = event.payload;
      if (filePath) {
        session.setPendingImport(filePath);
        eventBus.emit('vault:import_file', { path: filePath });
      }
    })
      .then((un) => {
        unlistenImport = un;
      })
      .catch((err) => {
        console.error('Failed to listen to finnca:import-file event:', err);
      });

    let unlistenVaultLocked: (() => void) | undefined;
    listen('vault:locked', async () => {
      await session.refresh();
      eventBus.emit('vault:locked', undefined);
    })
      .then((un) => {
        unlistenVaultLocked = un;
      })
      .catch((err) => {
        console.error('Failed to listen to vault:locked event:', err);
      });

    let unlistenRollback: (() => void) | undefined;
    listen<string>('finnca:rollback-restored', (event) => {
      const ver = event.payload || 'unknown';
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        title: i18n.t.rollbackNoticeTitle,
        message: i18n.t.rollbackNotice.replace('{version}', ver),
        priority: 'high',
      });
    })
      .then((un) => {
        unlistenRollback = un;
      })
      .catch(() => {});

    // Check if a rollback occurred prior to launch
    getRollbackNoticeCmd()
      .then((failedVer) => {
        if (failedVer) {
          notificationState.addNotification({
            type: 'LEDGER_INTEGRITY',
            title: i18n.t.rollbackNoticeTitle,
            message: i18n.t.rollbackNotice.replace('{version}', failedVer),
            priority: 'high',
          });
        }
      })
      .catch(() => {});

    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      const reason = event.reason;
      const message = reason instanceof Error ? reason.message : String(reason);
      const stack = reason instanceof Error ? reason.stack : undefined;
      diagnosticsState.captureError('unhandledrejection', message, stack);
    };

    const handleWindowError = (event: ErrorEvent) => {
      const message = event.message || 'Unknown window error';
      const stack = event.error instanceof Error ? event.error.stack : undefined;
      diagnosticsState.captureError('error', message, stack);
    };

    window.addEventListener('unhandledrejection', handleUnhandledRejection);
    window.addEventListener('error', handleWindowError);
    const preventWheelZoom = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        e.stopPropagation();
      }
    };

    const preventKeyZoom = (e: KeyboardEvent) => {
      if (
        (e.ctrlKey || e.metaKey) &&
        (e.key === '+' ||
          e.key === '-' ||
          e.key === '=' ||
          e.key === '_' ||
          e.key === '0' ||
          e.code === 'NumpadAdd' ||
          e.code === 'NumpadSubtract')
      ) {
        e.preventDefault();
        e.stopPropagation();
      }
    };

    // 3. Block multi-touch pinch gestures
    const preventTouch = (e: TouchEvent) => {
      if (e.touches && e.touches.length > 1) {
        e.preventDefault();
      }
    };

    // 4. Block WebKit gesture events (pinch-to-zoom on touchpad)
    const preventGesture = (e: Event) => {
      e.preventDefault();
      e.stopPropagation();
    };

    document.addEventListener('wheel', preventWheelZoom, {
      capture: true,
      passive: false,
    });
    window.addEventListener('wheel', preventWheelZoom, {
      capture: true,
      passive: false,
    });
    document.addEventListener('keydown', preventKeyZoom, {
      capture: true,
      passive: false,
    });
    window.addEventListener('keydown', preventKeyZoom, {
      capture: true,
      passive: false,
    });
    document.addEventListener('touchstart', preventTouch, {
      capture: true,
      passive: false,
    });
    document.addEventListener('touchmove', preventTouch, {
      capture: true,
      passive: false,
    });
    document.addEventListener('gesturestart', preventGesture, {
      capture: true,
      passive: false,
    });
    document.addEventListener('gesturechange', preventGesture, {
      capture: true,
      passive: false,
    });
    document.addEventListener('gestureend', preventGesture, {
      capture: true,
      passive: false,
    });

    const handleOnline = () => {
      checkAppUpdates(i18n.t);
    };
    window.addEventListener('online', handleOnline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
      window.removeEventListener('error', handleWindowError);
      document.removeEventListener('wheel', preventWheelZoom);
      window.removeEventListener('wheel', preventWheelZoom);
      document.removeEventListener('keydown', preventKeyZoom);
      window.removeEventListener('keydown', preventKeyZoom);
      document.removeEventListener('touchstart', preventTouch);
      document.removeEventListener('touchmove', preventTouch);
      document.removeEventListener('gesturestart', preventGesture);
      document.removeEventListener('gesturechange', preventGesture);
      document.removeEventListener('gestureend', preventGesture);
      unlistenImport?.();
      unlistenVaultLocked?.();
      unlistenRollback?.();
    };
  });
</script>

<svelte:head>
  <title>{APP_NAME}</title>
</svelte:head>

{@render children()}

<NotificationToast />

{#if updaterState.screenOpen}
  <UpdateScreen onClose={() => updaterState.closeScreen()} />
{/if}
