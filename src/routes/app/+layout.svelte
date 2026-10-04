<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { session } from '$lib/core/state/session.svelte';
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import { journalState } from '$lib/features/journal/state/journalDraft.svelte';
  import { reportState } from '$lib/features/report/state/report.svelte';
  import { closingBooksState } from '$lib/core/state/ledgerLock.svelte';
  import NotificationToast from '$lib/components/feedback/NotificationToast.svelte';
  import NotificationDrawer from '$lib/components/feedback/NotificationDrawer.svelte';
  import { APP_NAME } from '$lib/core/types';
  import { i18n } from '$lib/core/i18n.svelte';
  import { lockPolicy } from '$lib/features/security/state/lockPolicy.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';
  import { evaluateSmartNotifications } from '$lib/core/notification/smartEvaluator';
  import { runDailyFxSync } from '$lib/features/settings/fxSync';
  import { checkAppUpdates } from '$lib/core/updater/updateChecker';
  import { privacyState } from '$lib/core/state/privacy.svelte';

  import TopBar from '$lib/components/layout/TopBar.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import ModalHost from '$lib/components/layout/ModalHost.svelte';

  let { children } = $props();

  const healthStats = $derived.by(() => {
    const bs = reportState.balanceSheet;
    if (!bs) return null;
    return {
      balanced: bs.is_balanced,
      unbalancedTxCount: 0,
      discrepancy: bs.discrepancy,
      assets: bs.total_assets,
      liabilities: bs.total_liabilities,
      equity: bs.total_equity,
      netIncome: bs.net_income,
    };
  });

  async function lockActiveVault() {
    await session.lock();
    goto(resolve('/login'));
  }

  onMount(() => {
    (async () => {
      try {
        await session.refresh();
      } catch (err) {
        console.error('Failed to refresh app state:', err);
      }
      if (!session.raw) {
        goto(resolve('/login'), { replaceState: true });
        return;
      }
      if (!session.isConfigured) {
        goto(resolve('/setup'), { replaceState: true });
      } else if (!session.isUnlocked) {
        goto(resolve('/login'), { replaceState: true });
      } else {
        await Promise.all([
          accountsState.load(),
          journalState.loadEntries(),
          reportState.loadBalanceSheet(),
          closingBooksState.load(),
        ]);
        lockPolicy.mode = session.settings?.auto_lock_mode || 'always';
        const isBootMatch = await lockPolicy.checkBootIdFastUnlock(session.settings?.boot_id);
        if (!isBootMatch) {
          await lockActiveVault();
          return;
        }

        evaluateSmartNotifications(i18n.t);
        runDailyFxSync(i18n.t);
        checkAppUpdates(i18n.t);
      }
    })();

    const cleanupWatcher = lockPolicy.initInactivityWatcher(lockActiveVault);

    const handleGlobalKey = (e: KeyboardEvent) => {
      lockPolicy.touchActivity();
      if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault();
        modalState.toggleCommandPalette();
      } else if (e.altKey && (e.key === 'p' || e.key === 'P')) {
        e.preventDefault();
        privacyState.toggle();
      } else if (e.key === 'Escape') {
        if (modalState.commandPaletteOpen) {
          modalState.closeCommandPalette();
        } else if (modalState.inspectorOpen) {
          modalState.closeInspector();
        }
      } else if (
        !modalState.isAnyModalOpen &&
        !(
          e.target instanceof HTMLInputElement ||
          e.target instanceof HTMLTextAreaElement ||
          e.target instanceof HTMLSelectElement
        )
      ) {
        if (e.key === 'n' || e.key === 'N') {
          e.preventDefault();
          modalState.openInspector({ mode: 'journal' });
        } else if (e.key === 't' || e.key === 'T') {
          e.preventDefault();
          modalState.openInspector({ mode: 'transfer' });
        }
      }
    };

    const handleOnline = () => {
      runDailyFxSync(i18n.t);
    };

    window.addEventListener('keydown', handleGlobalKey);
    window.addEventListener('online', handleOnline);

    return () => {
      cleanupWatcher();
      window.removeEventListener('keydown', handleGlobalKey);
      window.removeEventListener('online', handleOnline);
    };
  });

  $effect(() => {
    if (privacyState.enabled) {
      document.body.classList.add('privacy-blur-enabled');
    } else {
      document.body.classList.remove('privacy-blur-enabled');
    }
  });
</script>

<svelte:head>
  <title>{APP_NAME}</title>
</svelte:head>

<div class="bg-bg-app text-text-base flex h-screen flex-col overflow-hidden">
  <TopBar {healthStats} onLock={lockActiveVault} />

  <div class="relative flex flex-1 overflow-hidden">
    <Sidebar {healthStats} onOpenHealthModal={() => modalState.openHealthPulse()} />

    <main class="bg-bg-app relative flex flex-1 flex-col overflow-hidden">
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        {@render children()}
      </div>
    </main>

    <NotificationToast />
    <NotificationDrawer />
  </div>

  <ModalHost {healthStats} onLock={lockActiveVault} />
</div>
