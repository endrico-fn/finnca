import * as api from '$lib/api';
import type { AppStateView } from '$lib/types';
import { ledger } from '$lib/accounting/store.svelte';

class AppStore {
  appState = $state<AppStateView | null>(null);
  loading = $state(true);
  private refreshPromise: Promise<void> | null = null;

  async refresh() {
    if (this.refreshPromise) return this.refreshPromise;
    this.refreshPromise = api
      .getAppState()
      .then((s) => {
        this.appState = s;
      })
      .finally(() => {
        this.loading = false;
        this.refreshPromise = null;
      });
    return this.refreshPromise;
  }

  async unlock(password: string) {
    this.appState = await api.unlockVault(password);
  }

  async lock() {
    this.appState = await api.lockVault();
    ledger.clear();
  }
}

export const store = new AppStore();
