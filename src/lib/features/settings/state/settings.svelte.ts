import { eventBus } from '$lib/core/events/eventBus.svelte';
import { session } from '$lib/core/state/session.svelte';
import { renameUser, renameVault } from '$lib/core/ipc/bindings';
import { i18n } from '$lib/core/i18n.svelte';
import { notificationState } from '$lib/core/state/notification.svelte';

export type SettingsTab = 'general' | 'finance' | 'security' | 'data';

export { fxState, DEFAULT_FX_RATE, type FxHistoryEntry } from '$lib/core/state/fx.svelte';

class GeneralSettingsState {
  username = $state('');
  vaultName = $state('');
  saving = $state(false);
  error = $state('');
  success = $state('');

  init() {
    this.username = session.currentUser || '';
    this.vaultName = session.currentVault || '';
    this.error = '';
    this.success = '';
  }

  get isDirty(): boolean {
    const origUser = session.currentUser || '';
    const origVault = session.currentVault || '';
    return (
      (this.username.trim() !== '' && this.username.trim() !== origUser) ||
      (this.vaultName.trim() !== '' && this.vaultName.trim() !== origVault)
    );
  }

  get canSave(): boolean {
    if (!this.isDirty || this.saving) return false;
    if (this.username.trim().length < 2) return false;
    if (this.vaultName.trim().length < 1) return false;
    return true;
  }

  async save(): Promise<boolean> {
    if (!this.canSave) return false;
    this.saving = true;
    this.error = '';
    this.success = '';

    try {
      let stateChanged = false;
      const origUser = session.currentUser || '';
      const origVault = session.currentVault || '';

      if (this.username.trim() !== origUser) {
        const state = await renameUser(this.username.trim());
        session.raw = state;
        stateChanged = true;
      }

      if (this.vaultName.trim() !== origVault) {
        const state = await renameVault(this.vaultName.trim());
        session.raw = state;
        stateChanged = true;
      }

      if (stateChanged) {
        eventBus.emit('vault:registry_changed', undefined);
      }

      this.username = session.currentUser || '';
      this.vaultName = session.currentVault || '';
      this.success = i18n.t.settingsSavedOk;
      notificationState.addNotification({
        type: 'INFO',
        priority: 'low',
        title: i18n.t.settings,
        message: i18n.t.settingsSavedOk,
      });

      setTimeout(() => {
        this.success = '';
      }, 3000);
      return true;
    } catch (e: unknown) {
      this.error = e instanceof Error ? e.message : String(e);
      notificationState.addNotification({
        type: 'LEDGER_INTEGRITY',
        priority: 'high',
        title: i18n.t.settings,
        message: this.error,
      });
      return false;
    } finally {
      this.saving = false;
    }
  }
}

export const generalSettingsState = new GeneralSettingsState();
