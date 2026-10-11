import { check } from '@tauri-apps/plugin-updater';
import { notificationState } from '$lib/core/state/notification.svelte';
import { updaterState } from '$lib/core/updater/updaterState.svelte';
import { getPref } from '$lib/core/state/prefs';
import type { TranslationDict } from '$lib/core/i18n.svelte';

let sessionNotifiedVersion: string | null = null;
let hasCheckedOnStartup = false;
let lastCheckTime = 0;
const PERIODIC_CHECK_INTERVAL_MS = 4 * 60 * 60 * 1000;

export function resetUpdateSessionState(): void {
  sessionNotifiedVersion = null;
  hasCheckedOnStartup = false;
  lastCheckTime = 0;
}

export async function checkAppUpdates(t: TranslationDict, force = false): Promise<void> {
  const notifToggles = getPref('finnca_notif_toggles', { autoUpdate: true });
  if (!force && notifToggles.autoUpdate === false) {
    return;
  }

  const now = Date.now();
  if (hasCheckedOnStartup && !force && now - lastCheckTime < PERIODIC_CHECK_INTERVAL_MS) {
    return;
  }

  hasCheckedOnStartup = true;
  lastCheckTime = now;

  try {
    const update = await check();
    if (!update) return;

    const latestVersion = update.version;
    updaterState.update = update;
    updaterState.phase = 'available';

    if (sessionNotifiedVersion === latestVersion && !force) {
      return;
    }
    sessionNotifiedVersion = latestVersion;

    const msg = t.updateAvailableMsg.replace('{version}', `v${latestVersion}`);

    notificationState.addNotification({
      type: 'INFO',
      title: t.updateAvailableTitle,
      message: msg,
      detail: update.body || `v${latestVersion}`,
      priority: 'high',
      actionLabel: t.updateActionView,
      actionHref: 'app:update',
    });
  } catch (err) {
    // network failure, browser/test environment, or unreleased state silently caught
    console.debug('[updater] Update check failed or offline:', err);
  }
}
