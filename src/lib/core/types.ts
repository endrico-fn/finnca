import type {
  AppStateView as RustAppStateView,
  Settings as RustSettings,
} from '$lib/core/ipc/bindings';

export const APP_NAME = (
  typeof __APP_NAME__ !== 'undefined' ? __APP_NAME__ : 'finnca'
).toUpperCase();
export const APP_VERSION = typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.1.0';
export const APP_SLUG = APP_NAME.toLowerCase().replace(/\s+/g, '-');

export type Settings = RustSettings;
export type AppStateView = RustAppStateView;

export type Currency = 'IDR' | 'USD';
export type ReconcileState = 'n' | 'c' | 'y';
