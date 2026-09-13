import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { AppStateView } from './types';

async function withTimeout<T>(p: Promise<T>, ms: number, label: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(`${label} timed out after ${ms}ms`)), ms);
  });
  return Promise.race([p, timeout]).finally(() => clearTimeout(timer));
}

export function getAppState() {
  return withTimeout(invoke<AppStateView>('get_app_state'), 10_000, 'get_app_state');
}

export function createVault(name: string, path: string) {
  return withTimeout(invoke<AppStateView>('create_vault', { name, path }), 30_000, 'create_vault');
}

export function createAccount(username: string, password: string) {
  return withTimeout(
    invoke<AppStateView>('create_account', { username, password }),
    120_000,
    'create_account'
  );
}

export function unlockVault(password: string) {
  return withTimeout(invoke<AppStateView>('unlock', { password }), 120_000, 'unlock');
}

export function lockVault() {
  return invoke<AppStateView>('lock');
}

export function importVault(path: string, password: string) {
  return withTimeout(
    invoke<AppStateView>('import_vault', { path, password }),
    120_000,
    'import_vault'
  );
}

export function getBootId() {
  return withTimeout(invoke<string>('get_boot_id'), 5_000, 'get_boot_id');
}

export function readVaultData() {
  return withTimeout(invoke<string>('read_vault_data'), 60_000, 'read_vault_data');
}

export function writeVaultData(records: string) {
  return withTimeout(invoke<void>('write_vault_data', { records }), 60_000, 'write_vault_data');
}

export function setAutoLockMode(mode: 'always' | 'on-reboot') {
  return invoke<AppStateView>('set_auto_lock_mode', { mode });
}

export function renameUser(username: string) {
  return invoke<AppStateView>('rename_user', { username });
}

export function renameVault(name: string) {
  return invoke<AppStateView>('rename_vault', { name });
}

export function changePassword(oldPassword: string, newPassword: string) {
  return withTimeout(
    invoke<AppStateView>('change_password', {
      oldPassword,
      newPassword,
    }),
    120_000,
    'change_password'
  );
}

export function openVaultFolder() {
  return invoke<void>('open_vault_folder');
}

export function deleteVaultAndAccount(password: string) {
  return withTimeout(
    invoke<AppStateView>('delete_vault_and_account', { password }),
    30_000,
    'delete_vault'
  );
}

export async function pickDirectory(): Promise<string | null> {
  const dir = await open({ directory: true, multiple: false });
  return typeof dir === 'string' ? dir : null;
}

export function getKnownVaultsBackend() {
  return invoke<
    Array<{
      id: string;
      name: string;
      path: string;
      username: string;
      last_opened_at?: string | null;
    }>
  >('get_known_vaults_cmd');
}

export function rememberKnownVaultBackend(entry: {
  id: string;
  name: string;
  path: string;
  username: string;
  last_opened_at?: string | null;
}) {
  return invoke<void>('remember_known_vault_cmd', { entry });
}

export function forgetKnownVaultBackend(idOrPath: string) {
  return invoke<void>('forget_known_vault_cmd', { idOrPath });
}

export function setActiveVault(path: string) {
  return invoke<AppStateView>('set_active_vault', { path });
}

export function createVaultBackup(destDir: string) {
  return invoke<void>('export_vault_backup_folder', { destDir });
}
