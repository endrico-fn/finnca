import { getPref, setPref } from '$lib/core/state/prefs';

export type SettingsTab = 'general' | 'finance' | 'security' | 'data';

export interface FxHistoryEntry {
  date: string;
  rate: number;
}

class FxStore {
  rate = $state<number>(16000);
  history = $state<FxHistoryEntry[]>([]);

  constructor() {
    this.loadFromStorage();
  }

  loadFromStorage() {
    this.rate = getPref('finnca_fx_rate', 16000);
    this.history = getPref('finnca_fx_history', []);
  }

  setRate(newRate: number) {
    this.rate = newRate;
    setPref('finnca_fx_rate', newRate);
    const today = new Date().toISOString().slice(0, 10);
    const filtered = this.history.filter((h) => h.date !== today);
    this.history = [...filtered, { date: today, rate: newRate }];
    setPref('finnca_fx_history', this.history);
  }
}

export const fxState = new FxStore();
