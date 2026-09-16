import { getPref, setPref } from '$lib/core/state/prefs';
import { notificationState } from '$lib/core/state/notification.svelte';
import { fetchLiveFxRate } from './fxService';
import { fxState } from './state/settings.svelte';
import type { TranslationDict } from '$lib/core/i18n/types';

function formatDiffStr(diff: number): string {
  const abs = Math.abs(diff).toLocaleString('id-ID');
  return diff > 0 ? `+Rp ${abs}` : `-Rp ${abs}`;
}

export async function runDailyFxSync(t: TranslationDict, force = false): Promise<number | null> {
  if (typeof navigator !== 'undefined' && !navigator.onLine) {
    return null;
  }

  const notifToggles = getPref('finnca_notif_toggles', { fx: true });
  if (notifToggles.fx === false && !force) {
    return null;
  }

  const today = new Date().toISOString().slice(0, 10);
  if (!force && getPref('finnca_last_fx_sync', '') === today) {
    return fxState.rate;
  }

  const rate = await fetchLiveFxRate();
  if (rate === null) return null;

  const oldRate = fxState.rate || getPref('finnca_fx_rate', 16000);
  const diff = rate - oldRate;

  fxState.setRate(rate);
  setPref('finnca_last_fx_sync', today);

  const pctDiff = oldRate > 0 ? Math.abs(diff / oldRate) * 100 : 0;
  if (oldRate > 0 && (pctDiff >= 0.75 || Math.abs(diff) >= 100)) {
    const diffStr = formatDiffStr(diff);
    notificationState.addNotification({
      type: 'FX_ALERT',
      priority: Math.abs(diff) >= 200 ? 'high' : 'medium',
      title: diff > 0 ? t.fxAlertIncrease : t.fxAlertDecrease,
      message: t.notifFxMoveMsg
        .replace('{rate}', rate.toLocaleString('id-ID'))
        .replace('{diff}', diffStr)
        .replace('{impact}', ''),
      actionHref: '/app/reports',
      actionLabel: t.notifViewPnl,
    });
  }

  return rate;
}
