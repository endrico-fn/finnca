import { getAppState, getBootId } from '$lib/core/ipc/bindings';
import { getPref, setPref } from '$lib/core/state/prefs';
import { eventBus } from '$lib/core/events/eventBus.svelte';
import type { Settings } from '$lib/core/types';

export class LockPolicyStore {
  lastActivity = $state(Date.now());
  timeoutMinutes = $state('15');
  mode = $state<Settings['auto_lock_mode']>('always');
  private fastUnlockResult: boolean | null = null;

  constructor() {
    this.loadFromStorage();
    eventBus.on('vault:locked', () => {
      this.resetBootIdCheck();
    });
    eventBus.on('vault:unlocked', () => {
      this.resetBootIdCheck();
    });
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
    if (this.mode !== 'on-reboot') return true;
    if (this.fastUnlockResult !== null) return this.fastUnlockResult;

    let expectedBootId = savedBootId;
    if (!expectedBootId) {
      try {
        const state = await getAppState();
        expectedBootId = state.settings.boot_id;
      } catch {
        this.fastUnlockResult = false;
        return false;
      }
    }
    if (!expectedBootId || expectedBootId.trim().length === 0) {
      this.fastUnlockResult = false;
      return false;
    }
    try {
      const currentBootId = await getBootId();
      const matches = currentBootId.trim() === expectedBootId.trim();
      this.fastUnlockResult = matches;
      return matches;
    } catch {
      this.fastUnlockResult = false;
      return false;
    }
  }

  resetBootIdCheck() {
    this.fastUnlockResult = null;
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
