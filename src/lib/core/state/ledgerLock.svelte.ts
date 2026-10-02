import { getClosingDateCmd, setClosingDateCmd } from '$lib/core/ipc/bindings';
import { eventBus } from '$lib/core/events/eventBus.svelte';

class LedgerLockState {
  closingDate = $state<string | null>(null);
  loading = $state(false);
  saving = $state(false);

  async load(): Promise<void> {
    this.loading = true;
    try {
      this.closingDate = await getClosingDateCmd();
    } catch {
      this.closingDate = null;
    } finally {
      this.loading = false;
    }
  }

  async setClosingDate(date: string | null): Promise<void> {
    this.saving = true;
    try {
      await setClosingDateCmd(date);
      this.closingDate = date;
      eventBus.emit('transaction:posted', { id: '' });
    } finally {
      this.saving = false;
    }
  }

  isDateLocked(date?: string | null): boolean {
    if (!this.closingDate || !date) return false;
    return date <= this.closingDate;
  }
}

export const ledgerLockState = new LedgerLockState();
export const closingBooksState = ledgerLockState;
