import { getPref, setPref } from '$lib/core/state/prefs';
import { notificationState } from '$lib/core/state/notification.svelte';
import { todayString } from '$lib/core/format/date';
import { formatMinorToDisplay } from '$lib/core/format/currency';
import { fxState } from './state/settings.svelte';
import type { TranslationDict } from '$lib/core/i18n/types';

const FX_ENDPOINT = 'https://open.er-api.com/v6/latest/USD';
const FX_ABORT_MS = 3500;

export async function fetchLiveFxRate(): Promise<number | null> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), FX_ABORT_MS);
  try {
    const res = await fetch(FX_ENDPOINT, { signal: controller.signal });
    if (!res.ok) return null;
    const data = await res.json();
    const rate = Math.round(data?.rates?.IDR ?? 0);
    return rate > 1000 ? rate : null;
  } catch {
    return null;
  } finally {
    clearTimeout(timeoutId);
  }
}

function formatDiffStr(diff: number): string {
  return formatMinorToDisplay(diff, 'IDR', { showSign: true });
}

export async function runDailyFxSync(t: TranslationDict, force = false): Promise<number | null> {
  if (typeof navigator !== 'undefined' && !navigator.onLine) {
    return null;
  }

  const notifToggles = getPref('finnca_notif_toggles', { fx: true });
  if (notifToggles.fx === false && !force) {
    return null;
  }

  const today = todayString();
  if (!force && getPref('finnca_last_fx_sync', '') === today) {
    return fxState.rate;
  }

  const rate = await fetchLiveFxRate();
  if (rate === null) return null;

  const oldRate = fxState.rate;
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
        .replace('{rate}', formatMinorToDisplay(rate, 'IDR'))
        .replace('{diff}', diffStr)
        .replace('{impact}', ''),
      actionHref: '/app/reports',
      actionLabel: t.notifViewPnl,
    });
  }

  return rate;
}
