export const APP_NAME = (__APP_NAME__ || 'finnca').toUpperCase();
export const version_app = __APP_VERSION__ || '0.1.0';

export interface Settings {
  auto_lock_mode: 'always' | 'on-reboot';
  boot_id?: string | null;
}

export interface AppStateView {
  configured: boolean;
  unlocked: boolean;
  username: string | null;
  vault_name: string | null;
  vault_path?: string | null;
  settings: Settings;
}
