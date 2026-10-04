import type { CreateJournalEntryInput, JournalEntryView } from '$lib/core/ipc/bindings';
import { createEmptyJournalDraft } from '$lib/features/journal/state/journalFormUtils';

export interface ConfirmModalConfig {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void | Promise<void>;
  onCancel?: () => void;
}

export interface InspectorOpenOptions {
  draft?: CreateJournalEntryInput | null;
  entry?: JournalEntryView | null;
  isNew?: boolean;
  mode?: 'transfer' | 'journal';
  fromId?: string;
  toId?: string;
}

class ModalState {
  commandPaletteOpen = $state(false);
  healthPulseOpen = $state(false);
  settingsOpen = $state(false);
  confirmConfig = $state<ConfirmModalConfig | null>(null);

  get isAnyModalOpen() {
    return (
      this.inspectorOpen ||
      this.commandPaletteOpen ||
      this.settingsOpen ||
      this.healthPulseOpen ||
      this.confirmConfig !== null
    );
  }

  inspectorOpen = $state(false);
  inspectorMode = $state<'transfer' | 'journal'>('transfer');
  inspectorDraft = $state<CreateJournalEntryInput | null>(null);
  inspectorEntry = $state<JournalEntryView | null>(null);
  inspectorIsNew = $state(true);
  inspectorInitialFrom = $state('');
  inspectorInitialTo = $state('');

  get isTransferMode() {
    return this.inspectorOpen && this.inspectorMode === 'transfer';
  }

  openSettings() {
    this.settingsOpen = true;
  }

  closeSettings() {
    this.settingsOpen = false;
  }

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

  openInspector(opts?: InspectorOpenOptions) {
    this.inspectorEntry = opts?.entry ?? null;
    this.inspectorIsNew = opts?.isNew ?? (opts?.entry ? false : true);
    const hasManyPostings =
      Boolean(opts?.entry && opts.entry.postings.length > 2) ||
      Boolean(opts?.draft && opts.draft.postings && opts.draft.postings.length > 2);
    this.inspectorMode = hasManyPostings ? 'journal' : (opts?.mode ?? 'transfer');
    this.inspectorInitialFrom = opts?.fromId ?? '';
    this.inspectorInitialTo = opts?.toId ?? '';
    this.inspectorDraft =
      opts?.draft ??
      (opts?.entry ? null : createEmptyJournalDraft(opts?.fromId ?? '', opts?.toId ?? ''));
    this.inspectorOpen = true;
  }

  closeInspector() {
    this.inspectorOpen = false;
    this.inspectorDraft = null;
    this.inspectorEntry = null;
    this.inspectorInitialFrom = '';
    this.inspectorInitialTo = '';
    this.inspectorIsNew = true;
  }

  /**
   * Explicit intent helper to open inspector for creating a transfer between accounts.
   */
  openTransfer(fromId = '', toId = '') {
    this.openInspector({
      isNew: true,
      mode: 'transfer',
      fromId,
      toId,
    });
  }

  /**
   * Explicit intent helper to open inspector for recording a new transaction entry.
   */
  openNewEntry(draft?: CreateJournalEntryInput | null) {
    this.openInspector({
      draft,
      isNew: true,
      mode: draft && draft.postings && draft.postings.length <= 2 ? 'transfer' : 'journal',
    });
  }

  /**
   * Explicit intent helper to open inspector for editing an existing journal entry.
   */
  openExistingEntry(entry: JournalEntryView) {
    this.openInspector({
      entry,
      isNew: false,
    });
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
