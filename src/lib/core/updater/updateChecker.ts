import { check } from '@tauri-apps/plugin-updater';
import { notificationState } from '$lib/core/state/notification.svelte';
import { getPref, setPref } from '$lib/core/state/prefs';
import type { TranslationDict } from '$lib/core/i18n.svelte';

const PREF_LAST_CHECK = 'finnca_last_update_check';
const PREF_NOTIFIED_VERSION = 'finnca_last_notified_version';
const CHECK_INTERVAL_MS = 4 * 60 * 60 * 1000;

export async function checkAppUpdates(t: TranslationDict, force = false): Promise<void> {
  const now = Date.now();
  const lastCheck = Number(getPref(PREF_LAST_CHECK, 0));

  if (!force && now - lastCheck < CHECK_INTERVAL_MS) {
    return;
  }

  setPref(PREF_LAST_CHECK, now);

  try {
    const update = await check();
    if (!update) return;

    const latestVersion = update.version;
    const lastNotified = getPref(PREF_NOTIFIED_VERSION, '');
    if (lastNotified === latestVersion && !force) return;

    setPref(PREF_NOTIFIED_VERSION, latestVersion);

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
  } catch {
    // network, browser environment, or unreleased state silently caught
  }
}
