import { getPref, setPref } from '$lib/core/state/prefs';
import { notificationState } from '$lib/core/state/notification.svelte';
import { fetchLiveFxRate } from './fxService';
import type { TranslationDict } from '$lib/core/i18n/types';

function formatDiffStr(diff: number): string {
  const abs = Math.abs(diff).toLocaleString('id-ID');
  return diff > 0 ? `+Rp ${abs}` : `-Rp ${abs}`;
}

export async function runDailyFxSync(t: TranslationDict): Promise<void> {
  const today = new Date().toISOString().slice(0, 10);
  if (getPref('finnca_last_fx_sync', '') === today) return;

  const rate = await fetchLiveFxRate();
  if (rate === null) return;

  const oldRate = getPref('finnca_fx_rate', 0);
  const diff = rate - oldRate;

  setPref('finnca_fx_rate', rate);
  setPref('finnca_last_fx_sync', today);

  const history = getPref('finnca_fx_history', []);
  history.push({ date: today, rate });
  const trimmed = history.slice(-30);
  setPref('finnca_fx_history', trimmed);

  if (oldRate > 0 && diff !== 0) {
    const diffStr = formatDiffStr(diff);
    notificationState.addNotification({
      type: 'FX_ALERT',
      priority: diff > 0 ? 'medium' : 'high',
      title: diff > 0 ? t.fxAlertIncrease : t.fxAlertDecrease,
      message: t.notifFxMoveMsg
        .replace('{rate}', rate.toLocaleString())
        .replace('{diff}', diffStr)
        .replace('{impact}', ''),
      actionHref: '/app/reports',
      actionLabel: t.notifViewPnl,
    });
  }
}
