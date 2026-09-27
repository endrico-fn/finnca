import type { CreateJournalEntryInput, JournalEntryView } from '$lib/core/ipc/bindings';
import { todayString } from '$lib/core/format/date';
import { getPref, setPref } from '$lib/core/state/prefs';

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
  confirmConfig = $state<ConfirmModalConfig | null>(null);

  inspectorOpen = $state(false);
  inspectorMode = $state<'transfer' | 'journal'>('transfer');
  inspectorDraft = $state<CreateJournalEntryInput | null>(null);
  inspectorEntry = $state<JournalEntryView | null>(null);
  inspectorIsNew = $state(true);
  inspectorInitialFrom = $state('');
  inspectorInitialTo = $state('');
  inspectorLayout = $state<'docked' | 'modal' | 'adaptive'>(getPref('finnca_inspector_layout', 'adaptive'));

  toggleInspectorLayout(currentEffective?: 'docked' | 'modal') {
    const next = (currentEffective ?? (this.inspectorLayout === 'docked' ? 'docked' : 'modal')) === 'docked' ? 'modal' : 'docked';
    this.inspectorLayout = next;
    setPref('finnca_inspector_layout', next);
  }

  get quickTxOpen() {
    return this.inspectorOpen;
  }
  set quickTxOpen(val: boolean) {
    this.inspectorOpen = val;
  }

  get quickTxDraft() {
    return this.inspectorDraft;
  }
  set quickTxDraft(val: CreateJournalEntryInput | null) {
    this.inspectorDraft = val;
  }

  get quickTxIsNew() {
    return this.inspectorIsNew;
  }
  set quickTxIsNew(val: boolean) {
    this.inspectorIsNew = val;
  }

  get transferModalOpen() {
    return this.inspectorOpen && this.inspectorMode === 'transfer';
  }
  set transferModalOpen(val: boolean) {
    this.inspectorOpen = val;
    if (val) this.inspectorMode = 'transfer';
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
    this.inspectorMode =
      opts?.mode ??
      (opts?.entry && opts.entry.postings.length > 2
        ? 'journal'
        : opts?.draft && opts.draft.postings && opts.draft.postings.length > 2
          ? 'journal'
          : 'transfer');
    this.inspectorInitialFrom = opts?.fromId ?? '';
    this.inspectorInitialTo = opts?.toId ?? '';
    this.inspectorDraft =
      opts?.draft ??
      (opts?.entry
        ? null
        : {
            id: crypto.randomUUID(),
            date: todayString(),
            description: '',
            notes: '',
            currency: 'IDR',
            fx_rate: null,
            postings: [],
          });
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

  openQuickTx(
    initialDraft?: CreateJournalEntryInput | null,
    isNew = true,
    mode: 'transfer' | 'journal' = 'journal'
  ) {
    this.openInspector({
      draft: initialDraft,
      isNew,
      mode:
        initialDraft && initialDraft.postings && initialDraft.postings.length > 2
          ? 'journal'
          : mode,
    });
  }

  closeQuickTx() {
    this.closeInspector();
  }

  openTransfer(fromId = '', toId = '') {
    this.openInspector({
      isNew: true,
      mode: 'transfer',
      fromId,
      toId,
    });
  }

  closeTransfer() {
    this.closeInspector();
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
