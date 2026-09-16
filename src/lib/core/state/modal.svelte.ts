import type { Transaction } from '$lib/core/types';
import { todayString } from '$lib/core/format/date';

export interface ConfirmModalConfig {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void | Promise<void>;
  onCancel?: () => void;
}

class ModalState {
  commandPaletteOpen = $state(false);
  healthPulseOpen = $state(false);
  quickTxOpen = $state(false);
  quickTxDraft = $state<Transaction | null>(null);
  quickTxIsNew = $state(true);
  transferModalOpen = $state(false);
  confirmConfig = $state<ConfirmModalConfig | null>(null);

  openCommandPalette() {
    this.commandPaletteOpen = true;
  }

  closeCommandPalette() {
    this.commandPaletteOpen = false;
  }

  toggleCommandPalette() {
    this.commandPaletteOpen = !this.commandPaletteOpen;
  }

  openHealthPulse() {
    this.healthPulseOpen = true;
  }

  closeHealthPulse() {
    this.healthPulseOpen = false;
  }

  openQuickTx(initialDraft?: Transaction | null, isNew = true) {
    this.quickTxIsNew = isNew;
    this.quickTxDraft = initialDraft || {
      id: crypto.randomUUID(),
      date: todayString(),
      dueDate: '',
      settled: false,
      description: '',
      num: '',
      notes: '',
      currency: 'IDR',
      splits: [],
    };
    this.quickTxOpen = true;
  }

  closeQuickTx() {
    this.quickTxOpen = false;
    this.quickTxDraft = null;
    this.quickTxIsNew = true;
  }

  openTransfer() {
    this.transferModalOpen = true;
  }

  closeTransfer() {
    this.transferModalOpen = false;
  }

  confirm(config: ConfirmModalConfig) {
    this.confirmConfig = config;
  }

  closeConfirm() {
    if (this.confirmConfig?.onCancel) {
      this.confirmConfig.onCancel();
    }
    this.confirmConfig = null;
  }
}

export const modalState = new ModalState();
