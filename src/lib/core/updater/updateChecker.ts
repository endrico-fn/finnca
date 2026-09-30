import { APP_VERSION } from '$lib/core/types';
import { notificationState } from '$lib/core/state/notification.svelte';
import { getPref, setPref } from '$lib/core/state/prefs';
import type { TranslationDict } from '$lib/core/i18n.svelte';

const REPO = 'endrico-fn/finnca';
const PREF_LAST_CHECK = 'finnca_last_update_check';
const PREF_NOTIFIED_VERSION = 'finnca_last_notified_version';
const CHECK_INTERVAL_MS = 4 * 60 * 60 * 1000;

interface GitHubReleaseResponse {
  tag_name: string;
  html_url: string;
  name?: string;
  body?: string;
}

function parseSemver(ver: string): [number, number, number] {
  const clean = ver.replace(/^[vV]/, '').trim();
  const parts = clean.split('.').map((p) => parseInt(p, 10) || 0);
  return [parts[0] ?? 0, parts[1] ?? 0, parts[2] ?? 0];
}

function isNewer(latest: string, current: string): boolean {
  const [lMajor, lMinor, lPatch] = parseSemver(latest);
  const [cMajor, cMinor, cPatch] = parseSemver(current);

  if (lMajor !== cMajor) return lMajor > cMajor;
  if (lMinor !== cMinor) return lMinor > cMinor;
  return lPatch > cPatch;
}

export async function checkAppUpdates(t: TranslationDict, force = false): Promise<void> {
  const now = Date.now();
  const lastCheck = Number(getPref(PREF_LAST_CHECK, 0));

  if (!force && now - lastCheck < CHECK_INTERVAL_MS) {
    return;
  }

  setPref(PREF_LAST_CHECK, now);

  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers: {
        Accept: 'application/vnd.github.v3+json',
      },
      signal: AbortSignal.timeout(4000),
    });

    if (!res.ok) return;

    const release = (await res.json()) as GitHubReleaseResponse;
    if (!release.tag_name) return;

    const latestTag = release.tag_name;
    if (isNewer(latestTag, APP_VERSION)) {
      const lastNotified = getPref(PREF_NOTIFIED_VERSION, '');
      if (lastNotified === latestTag && !force) return;

      setPref(PREF_NOTIFIED_VERSION, latestTag);

      const msg = t.updateAvailableMsg.replace('{version}', latestTag);

      notificationState.addNotification({
        type: 'INFO',
        title: t.updateAvailableTitle,
        message: msg,
        detail: release.name || release.tag_name,
        priority: 'high',
        actionLabel: t.updateActionView,
        actionHref: 'app:update',
      });
    }
  } catch {
    // network or timeout failure ignored in background
  }
}
