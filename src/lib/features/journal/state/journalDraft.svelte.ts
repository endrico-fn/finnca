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
import type { Currency, ReconcileState } from '$lib/core/types';
import { todayString } from '$lib/core/format/date';

export type { Currency, ReconcileState, PostingInput };

class JournalStore {
  entries = $state.raw<JournalEntryView[]>([]);
  totals = $state<LedgerTotalsView | null>(null);
  accountLedger = $state.raw<AccountRunningLedgerItem[]>([]);
  activeLedgerAccountId = $state<string | null>(null);

  loading = $state(false);
  error = $state<string | null>(null);

  draftDate = $state<string>(todayString());
  draftDescription = $state<string>('');
  draftNotes = $state<string>('');
  draftCurrency = $state<string>('IDR');
  draftFxRate = $state<number | null>(null);
  draftLines = $state<PostingInput[]>([
    { account_id: '', amount: 0 },
    { account_id: '', amount: 0 },
  ]);

  draftSum = $derived(this.draftLines.reduce((acc, l) => acc + (Number(l.amount) || 0), 0));
  isDraftBalanced = $derived(this.draftSum === 0);

  resetDraft(): void {
    this.draftDate = todayString();
    this.draftDescription = '';
    this.draftNotes = '';
    this.draftCurrency = 'IDR';
    this.draftFxRate = null;
    this.draftLines = [
      { account_id: '', amount: 0 },
      { account_id: '', amount: 0 },
    ];
    this.error = null;
  }

  addDraftLine(): void {
    this.draftLines.push({ account_id: '', amount: 0 });
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
    return this.saveEntry({
      date: this.draftDate,
      description: this.draftDescription.trim(),
      notes: this.draftNotes.trim() || null,
      currency: this.draftCurrency,
      fx_rate: this.draftFxRate,
      postings: this.draftLines.map((l) => ({
        id: l.id,
        account_id: l.account_id,
        amount: Math.round(l.amount),
        memo: l.memo || null,
        action: l.action || null,
      })),
    });
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

  async saveEntry(entry: CreateJournalEntryInput): Promise<JournalEntryView> {
    this.loading = true;
    this.error = null;

    const postings: PostingInput[] = entry.postings.map((p) => ({
      id: p.id || undefined,
      account_id: p.account_id,
      amount: Math.round(p.amount),
      memo: p.memo || null,
      action: p.action || null,
      reconcile: p.reconcile === 'y' ? 'y' : p.reconcile === 'c' ? 'c' : null,
    }));

    try {
      let result: JournalEntryView;
      if (entry.id && this.entries.some((e) => e.id === entry.id)) {
        result = await updateJournalEntryCmd(entry.id, {
          date: entry.date,
          description: entry.description,
          notes: entry.notes || null,
          currency: entry.currency || 'IDR',
          fx_rate: entry.fx_rate ?? null,
          postings,
        });
      } else {
        result = await postJournalEntryCmd({
          date: entry.date,
          description: entry.description,
          notes: entry.notes || null,
          currency: entry.currency || 'IDR',
          fx_rate: entry.fx_rate ?? null,
          postings,
        });
      }
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
