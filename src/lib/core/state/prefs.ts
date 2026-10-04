export interface PrefSchema {
  finnca_locale: 'en' | 'id';
  finnca_fx_rate: number;
  finnca_fx_history: Array<{ date: string; rate: number }>;
  finnca_notif_toggles: Record<string, boolean>;
  finnca_num_format: 'dot' | 'comma';
  finnca_date_format: 'iso' | 'slash';
  finnca_last_fx_sync: string;
  finnca_known_vaults: unknown[];
  finnca_active_vault: string;
  finnca_avatar: string;
  finnca_snoozed_notifs: Record<string, number>;
  finnca_lock_timeout: string;
  finnca_last_update_check: number;
  finnca_last_notified_version: string;
}

export type PrefKey = keyof PrefSchema;

function isBrowser(): boolean {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

export function getPref<K extends PrefKey>(key: K, fallback: PrefSchema[K]): PrefSchema[K] {
  if (!isBrowser()) return fallback;
  try {
    const raw = window.localStorage.getItem(key);
    if (raw === null) return fallback;

    if (typeof fallback === 'string') {
      return raw as PrefSchema[K];
    }
    if (typeof fallback === 'number') {
      const parsed = Number(raw);
      return (Number.isFinite(parsed) ? parsed : fallback) as PrefSchema[K];
    }
    return (JSON.parse(raw) as PrefSchema[K]) ?? fallback;
  } catch {
    return fallback;
  }
}

export function setPref<K extends PrefKey>(key: K, value: PrefSchema[K]): void {
  if (!isBrowser()) return;
  try {
    if (typeof value === 'string') {
      window.localStorage.setItem(key, value);
    } else {
      window.localStorage.setItem(key, JSON.stringify(value));
    }
  } catch (err) {
    console.error(`[Prefs] Error setting "${key}":`, err);
  }
}

export function removePref<K extends PrefKey>(key: K): void {
  if (!isBrowser()) return;
  try {
    window.localStorage.removeItem(key);
  } catch (err) {
    console.error(`[Prefs] Error removing "${key}":`, err);
  }
}

export function clearPrefs(keys?: PrefKey[]): void {
  if (!isBrowser()) return;
  try {
    if (keys && keys.length > 0) {
      for (const k of keys) {
        window.localStorage.removeItem(k);
      }
    } else {
      window.localStorage.clear();
    }
  } catch (err) {
    console.error('[Prefs] Error clearing preferences:', err);
  }
}
