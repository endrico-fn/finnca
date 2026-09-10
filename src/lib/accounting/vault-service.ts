import * as api from '$lib/api';
import type { VaultData } from './types';
import { migrateLegacy } from './finance';
import { DEFAULT_FX_RATE } from './types';

export async function loadVault(): Promise<VaultData> {
  const raw = await api.readVaultData();
  if (!raw || !raw.trim()) throw new Error('vault empty — refusing to overwrite');
  let parsed: any;
  try {
    parsed = JSON.parse(raw);
  } catch {
    parsed = [];
  }
  const migrated = migrateLegacy(parsed, DEFAULT_FX_RATE);
  if (!migrated.fxRate) migrated.fxRate = DEFAULT_FX_RATE;
  return migrated;
}

export async function saveVault(data: VaultData): Promise<void> {
  data.updatedAt = new Date().toISOString();
  await api.writeVaultData(JSON.stringify(data));
}
