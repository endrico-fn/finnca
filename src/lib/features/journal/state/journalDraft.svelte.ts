import {
  postJournalEntryCmd,
  updateJournalEntryCmd,
  deleteJournalEntryCmd,
  listJournalEntriesCmd,
  getJournalEntryCmd,
  getAccountLedgerCmd,
  getLedgerTotalsCmd,
  type CreateJournalEntryInput,
  type UpdateJournalEntryInput,
  type JournalEntryView,
  type LedgerTotalsView,
  type AccountRunningLedgerItem,
  type PostingInput,
} from '$lib/core/ipc/bindings';
import { AppError } from '$lib/core/ipc/errors';
import { eventBus } from '$lib/core/events/eventBus.svelte';
import type {
  Currency,
  ReconcileState,
  Split,
  Transaction,
  DraftPostingLine,
} from '$lib/core/types';

export type { Currency, ReconcileState, Split, Transaction, DraftPostingLine };

class JournalStore {
  // Recent / list entries
  entries = $state<JournalEntryView[]>([]);
  totals = $state<LedgerTotalsView | null>(null);
  accountLedger = $state<AccountRunningLedgerItem[]>([]);
  activeLedgerAccountId = $state<string | null>(null);

  loading = $state(false);
  error = $state<string | null>(null);

  // Form draft state
  draftDate = $state<string>(new Date().toISOString().split('T')[0]);
  draftDescription = $state<string>('');
  draftNotes = $state<string>('');
  draftCurrency = $state<string>('IDR');
  draftFxRate = $state<number | null>(null);
  draftLines = $state<DraftPostingLine[]>([
    { accountId: '', amount: 0 },
    { accountId: '', amount: 0 },
  ]);

  // Derived indicator for UI helper only (warning hint, NOT business logic)
  draftSum = $derived(this.draftLines.reduce((acc, l) => acc + (Number(l.amount) || 0), 0));
  isDraftBalanced = $derived(this.draftSum === 0);

  resetDraft(): void {
    this.draftDate = new Date().toISOString().split('T')[0];
    this.draftDescription = '';
    this.draftNotes = '';
    this.draftCurrency = 'IDR';
    this.draftFxRate = null;
    this.draftLines = [
      { accountId: '', amount: 0 },
      { accountId: '', amount: 0 },
    ];
    this.error = null;
  }

  addDraftLine(): void {
    this.draftLines.push({ accountId: '', amount: 0 });
  }

  removeDraftLine(index: number): void {
    if (this.draftLines.length > 2) {
      this.draftLines.splice(index, 1);
    }
  }

  async loadEntries(limit = 100, offset = 0): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.entries = await listJournalEntriesCmd(limit, offset);
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async loadTotals(fxRate?: number): Promise<void> {
    try {
      this.totals = await getLedgerTotalsCmd(fxRate);
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
    }
  }

  async loadAccountLedger(accountId: string): Promise<void> {
    this.loading = true;
    this.error = null;
    this.activeLedgerAccountId = accountId;
    try {
      this.accountLedger = await getAccountLedgerCmd(accountId);
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async postDraft(): Promise<JournalEntryView> {
    this.loading = true;
    this.error = null;

    const postings: PostingInput[] = this.draftLines.map((line) => ({
      id: line.id,
      account_id: line.accountId,
      amount: Math.round(line.amount),
      memo: line.memo || null,
      action: line.action || null,
    }));

    const input: CreateJournalEntryInput = {
      date: this.draftDate,
      description: this.draftDescription.trim(),
      notes: this.draftNotes.trim() || null,
      currency: this.draftCurrency,
      fx_rate: this.draftFxRate,
      postings,
    };

    try {
      const result = await postJournalEntryCmd(input);
      this.resetDraft();
      await this.loadEntries();
      await this.loadTotals();
      eventBus.emit('transaction:posted', { id: result.id });
      eventBus.emit('accounts:changed', undefined);
      return result;
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async updateEntry(id: string, input: UpdateJournalEntryInput): Promise<JournalEntryView> {
    this.loading = true;
    this.error = null;
    try {
      const result = await updateJournalEntryCmd(id, input);
      await this.loadEntries();
      await this.loadTotals();
      eventBus.emit('transaction:posted', { id });
      eventBus.emit('accounts:changed', undefined);
      return result;
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  async deleteEntry(id: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      await deleteJournalEntryCmd(id);
      await this.loadEntries();
      await this.loadTotals();
      eventBus.emit('transaction:posted', { id });
      eventBus.emit('accounts:changed', undefined);
    } catch (err) {
      const e = AppError.fromUnknown(err);
      this.error = e.message;
      throw e;
    } finally {
      this.loading = false;
    }
  }

  transactions = $derived.by(() => {
    return this.entries.map((entry): Transaction => ({
      id: entry.id,
      date: entry.date,
      description: entry.description,
      notes: entry.notes ?? undefined,
      currency: entry.currency as Currency,
      fxRateAtTransaction: entry.fx_rate,
      splits: entry.postings.map((p) => ({
        id: p.id,
        accountId: p.account_id,
        amount: p.amount,
        memo: p.memo ?? undefined,
        reconcile: p.reconcile === 'y' ? 'y' : p.reconcile === 'c' ? 'c' : 'n',
      })),
    }));
  });

  async saveTransaction(tx: Transaction): Promise<void> {
    const postings: PostingInput[] = tx.splits.map((s) => ({
      id: s.id || undefined,
      account_id: s.accountId,
      amount: Math.round(s.amount),
      memo: s.memo || null,
      action: null,
      reconcile: s.reconcile === 'y' ? 'y' : s.reconcile === 'c' ? 'c' : null,
    }));

    if (tx.id && this.entries.some((e) => e.id === tx.id)) {
      await this.updateEntry(tx.id, {
        date: tx.date,
        description: tx.description,
        notes: tx.notes || null,
        currency: tx.currency,
        fx_rate: tx.fxRateAtTransaction || null,
        postings,
      });
    } else {
      const res = await postJournalEntryCmd({
        date: tx.date,
        description: tx.description,
        notes: tx.notes || null,
        currency: tx.currency,
        fx_rate: tx.fxRateAtTransaction || null,
        postings,
      });
      await this.loadEntries();
      await this.loadTotals();
      eventBus.emit('transaction:posted', { id: res.id });
      eventBus.emit('accounts:changed', undefined);
    }
  }

  async deleteTransaction(id: string): Promise<void> {
    await this.deleteEntry(id);
  }

  async getEntry(id: string): Promise<JournalEntryView> {
    return getJournalEntryCmd(id);
  }
}

export const journalState = new JournalStore();

eventBus.on('vault:unlocked', () => {
  journalState.loadEntries().catch(() => {});
  journalState.loadTotals().catch(() => {});
});
eventBus.on('vault:locked', () => {
  journalState.resetDraft();
});
eventBus.on('transaction:posted', () => {
  journalState.loadTotals().catch(() => {});
});
