import {
  rememberKnownVaultCmd,
  setActiveVault as setActiveVaultIpc,
  forgetKnownVaultCmd,
  getKnownVaultsCmd,
  getAppState,
} from '$lib/core/ipc/bindings';
import { removePref } from '$lib/core/state/prefs';
import { eventBus } from '$lib/core/events/eventBus.svelte';

export interface KnownVault {
  id: string; // fallback ID for backward compatibility
  name: string;
  path: string;
  username: string;
  lastOpenedAt: string | null;
}

class VaultRegistryStore {
  knownVaults = $state<KnownVault[]>([]);
  activeVaultId = $state<string | null>(null); // maps to vault path

  getKnownVaults(): KnownVault[] {
    return this.knownVaults;
  }

  getActiveVaultId(): string | null {
    return this.activeVaultId;
  }

  async rememberVault(v: Omit<KnownVault, 'lastOpenedAt'>) {
    const entry = { ...v, lastOpenedAt: new Date().toISOString() };
    await rememberKnownVaultCmd({
      id: entry.id,
      name: entry.name,
      path: entry.path,
      username: entry.username,
      last_opened_at: entry.lastOpenedAt,
    }).catch(() => {});
    await this.syncVaultsFromBackend();
  }

  async setActiveVault(path: string): Promise<boolean> {
    try {
      await setActiveVaultIpc(path);
      this.activeVaultId = path;
      return true;
    } catch (e) {
      console.error('Failed to set active vault:', e);
      return false;
    }
  }

  async forgetVault(idOrPath: string) {
    await forgetKnownVaultCmd(idOrPath).catch(() => {});
    await this.syncVaultsFromBackend();
  }

  async syncVaultsFromBackend(): Promise<KnownVault[]> {
    try {
      const backendVaults = await getKnownVaultsCmd();
      if (Array.isArray(backendVaults)) {
        this.knownVaults = backendVaults.map((bv) => ({
          id: bv.id,
          name: bv.name,
          path: bv.path,
          username: bv.username,
          lastOpenedAt: bv.last_opened_at ?? null,
        }));
      }

      const appState = await getAppState();
      if (appState.vault_path) {
        this.activeVaultId = appState.vault_path;
      } else if (this.knownVaults.length > 0) {
        this.activeVaultId = this.knownVaults[0].path;
      }
    } catch (e) {
      console.error('Failed to sync vaults from backend:', e);
    }
    return this.knownVaults;
  }

  clearVaultRegistry() {
    this.knownVaults = [];
    this.activeVaultId = null;
    removePref('finnca_known_vaults');
    removePref('finnca_active_vault');
    removePref('finnca_avatar');
  }
}

const vaultRegistry = new VaultRegistryStore();

export function getKnownVaults() {
  return vaultRegistry.getKnownVaults();
}
export function getActiveVaultId() {
  return vaultRegistry.getActiveVaultId();
}
export function rememberVault(v: Omit<KnownVault, 'lastOpenedAt'>) {
  return vaultRegistry.rememberVault(v);
}
export function setActiveVault(path: string): Promise<boolean> {
  return vaultRegistry.setActiveVault(path);
}
export function forgetVault(id: string) {
  return vaultRegistry.forgetVault(id);
}
export function syncVaultsFromBackend() {
  return vaultRegistry.syncVaultsFromBackend();
}
export function clearVaultRegistry() {
  return vaultRegistry.clearVaultRegistry();
}

eventBus.on('vault:registry_changed', () => {
  void vaultRegistry.syncVaultsFromBackend();
});

eventBus.on('vault:deleted', () => {
  vaultRegistry.clearVaultRegistry();
});
