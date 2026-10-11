import {
  getAppState,
  unlockVault,
  lockVault,
  clearPendingImportPath,
} from '$lib/core/ipc/bindings';
import type { AppStateView, Settings } from '$lib/core/types';
import { eventBus } from '$lib/core/events/eventBus.svelte';

class SessionStore {
  raw = $state<AppStateView | null>(null);
  loading = $state(true);
  error = $state<string | null>(null);

  isConfigured = $derived(this.raw?.configured ?? false);
  isUnlocked = $derived(this.raw?.unlocked ?? false);
  currentUser = $derived(this.raw?.username ?? null);
  currentVault = $derived(this.raw?.vault_name ?? null);
  vaultPath = $derived(this.raw?.vault_path ?? null);
  settings = $derived<Settings | null>(this.raw?.settings ?? null);
  pendingImportPath = $derived(this.raw?.pending_import_path ?? null);

  setPendingImport(path: string | null): void {
    if (this.raw) {
      this.raw = { ...this.raw, pending_import_path: path };
    }
  }

  async clearPendingImport(): Promise<void> {
    this.setPendingImport(null);
    try {
      await clearPendingImportPath();
    } catch {
      // ignore
    }
  }

  private refreshPromise: Promise<AppStateView> | null = null;

  async refresh(): Promise<AppStateView> {
    if (this.refreshPromise) return this.refreshPromise;
    this.refreshPromise = getAppState()
      .then((s) => {
        const wasUnlocked = this.raw?.unlocked ?? false;
        this.raw = s;
        if (s.unlocked && !wasUnlocked) {
          eventBus.emit('vault:unlocked', {
            vaultName: s.vault_name ?? '',
            username: s.username ?? '',
          });
        }
        return s;
      })
      .catch((err) => {
        this.error = err instanceof Error ? err.message : String(err);
        throw err;
      })
      .finally(() => {
        this.loading = false;
        this.refreshPromise = null;
      });
    return this.refreshPromise;
  }

  async unlock(password: string): Promise<AppStateView> {
    this.loading = true;
    this.error = null;
    try {
      const state = await unlockVault(password);
      this.raw = state;
      if (state.unlocked) {
        eventBus.emit('vault:unlocked', {
          vaultName: state.vault_name ?? '',
          username: state.username ?? '',
        });
      }
      return state;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      this.loading = false;
    }
  }

  async lock(): Promise<AppStateView> {
    this.loading = true;
    try {
      const state = await lockVault();
      this.raw = state;
      eventBus.emit('vault:locked', undefined);
      return state;
    } finally {
      this.loading = false;
    }
  }
}

export const session = new SessionStore();
