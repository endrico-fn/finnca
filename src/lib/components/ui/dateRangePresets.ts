import type { TranslationDict } from '$lib/core/i18n/types';
import { todayString } from '$lib/core/format/date';

export function getThisMonthRange(): [string, string] {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const lastDay = new Date(y, now.getMonth() + 1, 0).getDate();
  return [`${y}-${m}-01`, `${y}-${m}-${String(lastDay).padStart(2, '0')}`];
}

export function getLastMonthRange(): [string, string] {
  const now = new Date();
  const y = now.getMonth() === 0 ? now.getFullYear() - 1 : now.getFullYear();
  const m = now.getMonth() === 0 ? 12 : now.getMonth();
  const mStr = String(m).padStart(2, '0');
  const lastDay = new Date(y, m, 0).getDate();
  return [`${y}-${mStr}-01`, `${y}-${mStr}-${String(lastDay).padStart(2, '0')}`];
}

export function getThisYearRange(): [string, string] {
  const y = new Date().getFullYear();
  return [`${y}-01-01`, `${y}-12-31`];
}

export function getLast30DaysRange(): [string, string] {
  const end = new Date();
  const start = new Date();
  start.setDate(start.getDate() - 29);
  return [todayString(start), todayString(end)];
}

export function formatRangeDisplayLabel(from: string, to: string, t: TranslationDict): string {
  if (!from && !to) return t.allTime;

  const [tmFrom, tmTo] = getThisMonthRange();
  if (from === tmFrom && to === tmTo) return t.thisMonth;

  const [lmFrom, lmTo] = getLastMonthRange();
  if (from === lmFrom && to === lmTo) return t.lastMonth;

  const [tyFrom, tyTo] = getThisYearRange();
  if (from === tyFrom && to === tyTo) return t.thisYear;

  const [l30s, l30e] = getLast30DaysRange();
  if (from === l30s && to === l30e) return t.last30Days;

  if (from && to) return `${from} → ${to}`;

  return `${from || '...'} → ${to || '...'}`;
}
