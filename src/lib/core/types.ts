import type {
  AppStateView as RustAppStateView,
  Settings as RustSettings,
} from '$lib/core/ipc/bindings';

export const APP_NAME = (
  typeof __APP_NAME__ !== 'undefined' ? __APP_NAME__ : 'finnca'
).toUpperCase();
export const APP_VERSION = typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.1.0';
export const APP_SLUG = APP_NAME.toLowerCase().replace(/\s+/g, '-');

let cachedRuntimeVersion: string | null = null;
export async function getRuntimeVersion(): Promise<string> {
  if (cachedRuntimeVersion) return cachedRuntimeVersion;
  try {
    const { getVersion } = await import('@tauri-apps/api/app');
    cachedRuntimeVersion = await getVersion();
    return cachedRuntimeVersion;
  } catch {
    return APP_VERSION;
  }
}

export type Settings = RustSettings;
export type AppStateView = RustAppStateView;

export type Currency = 'IDR' | 'USD';
export type ReconcileState = 'n' | 'c' | 'y';
