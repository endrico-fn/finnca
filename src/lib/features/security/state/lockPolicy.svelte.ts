import { getAppState } from '$lib/core/ipc/bindings';
import { getPref, setPref } from '$lib/core/state/prefs';
import type { Settings } from '$lib/core/types';

export class LockPolicyStore {
  lastActivity = $state(Date.now());
  timeoutMinutes = $state('15');
  mode = $state<Settings['auto_lock_mode']>('always');
  private bootIdChecked = false;

  constructor() {
    this.loadFromStorage();
  }

  loadFromStorage() {
    this.timeoutMinutes = getPref('finnca_lock_timeout', '15');
  }

  saveTimeout(timeout: string) {
    this.timeoutMinutes = timeout;
    setPref('finnca_lock_timeout', timeout);
  }

  touchActivity() {
    this.lastActivity = Date.now();
  }

  getInactivityTimeoutMs(): number {
    if (this.timeoutMinutes === 'never') return Infinity;
    const minutes = parseInt(this.timeoutMinutes, 10);
    return (isNaN(minutes) || minutes < 1 ? 15 : minutes) * 60 * 1000;
  }

  async checkBootIdFastUnlock(savedBootId?: string | null): Promise<boolean> {
    if (this.bootIdChecked || !savedBootId || this.mode !== 'on-reboot') return true;
    this.bootIdChecked = true;
    try {
      const state = await getAppState();
      const currentBootId = state.settings.boot_id ?? '';
      return currentBootId.trim() === savedBootId.trim();
    } catch {
      return true;
    }
  }

  resetBootIdCheck() {
    this.bootIdChecked = false;
  }

  initInactivityWatcher(onTimeout: () => void | Promise<void>): () => void {
    if (typeof window === 'undefined') return () => {};

    const handleUserInteraction = () => {
      this.touchActivity();
    };

    const intervalId = setInterval(() => {
      if (this.mode === 'on-reboot') return;
      if (Date.now() - this.lastActivity > this.getInactivityTimeoutMs()) {
        void onTimeout();
      }
    }, 10000);

    window.addEventListener('keydown', handleUserInteraction);
    window.addEventListener('mousemove', handleUserInteraction);
    window.addEventListener('mousedown', handleUserInteraction);
    window.addEventListener('scroll', handleUserInteraction, true);
    window.addEventListener('touchstart', handleUserInteraction);

    return () => {
      clearInterval(intervalId);
      window.removeEventListener('keydown', handleUserInteraction);
      window.removeEventListener('mousemove', handleUserInteraction);
      window.removeEventListener('mousedown', handleUserInteraction);
      window.removeEventListener('scroll', handleUserInteraction, true);
      window.removeEventListener('touchstart', handleUserInteraction);
    };
  }
}

export const lockPolicy = new LockPolicyStore();
