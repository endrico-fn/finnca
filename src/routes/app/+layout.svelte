<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { session } from '$lib/core/state/session.svelte';
  import { accountsState } from '$lib/features/accounts/state/accounts.svelte';
  import { journalState } from '$lib/features/journal/state/journalDraft.svelte';
  import { reportState } from '$lib/features/report/state/report.svelte';
  import NotificationToast from '$lib/components/feedback/NotificationToast.svelte';
  import NotificationDrawer from '$lib/components/feedback/NotificationDrawer.svelte';
  import { APP_NAME } from '$lib/core/types';
  import { lockPolicy } from '$lib/features/security/state/lockPolicy.svelte';
  import { modalState } from '$lib/core/state/modal.svelte';

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

  async function doLock() {
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
        ]);
        lockPolicy.mode = session.settings?.auto_lock_mode || 'always';
        const isBootMatch = await lockPolicy.checkBootIdFastUnlock(session.settings?.boot_id);
        if (!isBootMatch) {
          await doLock();
        }
      }
    })();

    const cleanupWatcher = lockPolicy.initInactivityWatcher(doLock);

    const handleGlobalKey = (e: KeyboardEvent) => {
      lockPolicy.touchActivity();
      if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault();
        modalState.toggleCommandPalette();
      } else if (e.key === 'Escape' && modalState.commandPaletteOpen) {
        modalState.closeCommandPalette();
      }
    };

    window.addEventListener('keydown', handleGlobalKey);

    return () => {
      cleanupWatcher();
      window.removeEventListener('keydown', handleGlobalKey);
    };
  });
</script>

<svelte:head>
  <title>{APP_NAME}</title>
</svelte:head>

<div class="bg-bg-app text-text-base flex h-screen flex-col overflow-hidden">
  <TopBar {healthStats} onOpenHealthModal={() => modalState.openHealthPulse()} onLock={doLock} />

  <div class="relative flex flex-1 overflow-hidden">
    <Sidebar {healthStats} onOpenHealthModal={() => modalState.openHealthPulse()} />

    <main class="bg-bg-app flex flex-1 flex-col overflow-hidden">
      {@render children()}
    </main>

    <NotificationToast />
    <NotificationDrawer />
  </div>

  <ModalHost {healthStats} onLock={doLock} />
</div>
