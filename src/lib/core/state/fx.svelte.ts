import { getPref, setPref } from '$lib/core/state/prefs';
import { eventBus } from '$lib/core/events/eventBus.svelte';
import { todayString } from '$lib/core/format/date';

export interface FxHistoryEntry {
  date: string;
  rate: number;
}

export const DEFAULT_FX_RATE = 16000;

class FxStore {
  rate = $state<number>(DEFAULT_FX_RATE);
  history = $state<FxHistoryEntry[]>([]);

  constructor() {
    this.loadFromStorage();
  }

  loadFromStorage() {
    this.rate = getPref('finnca_fx_rate', DEFAULT_FX_RATE);
    this.history = getPref('finnca_fx_history', []);
  }

  setRate(newRate: number) {
    this.rate = newRate;
    setPref('finnca_fx_rate', newRate);
    const today = todayString();
    const filtered = this.history.filter((h) => h.date !== today);
    this.history = [...filtered, { date: today, rate: newRate }];
    setPref('finnca_fx_history', this.history);
    eventBus.emit('fx:rate_changed', { rate: newRate });
  }
}

export const fxState = new FxStore();
