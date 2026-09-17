import type { JournalEntryView } from '$lib/core/ipc/bindings';

export const APP_NAME = (__APP_NAME__ || 'finnca').toUpperCase();
export const version_app = __APP_VERSION__ || '0.1.0';

export interface Settings {
  auto_lock_mode: 'always' | 'on-reboot';
  boot_id?: string | null;
}

export interface AppStateView {
  configured: boolean;
  unlocked: boolean;
  username: string | null;
  vault_name: string | null;
  vault_path?: string | null;
  settings: Settings;
}

export type Currency = 'IDR' | 'USD';
export type ReconcileState = 'n' | 'c' | 'y';

export interface Split {
  id: string;
  accountId: string;
  amount: number;
  memo?: string;
  action?: string;
  reconcile: ReconcileState;
}

export interface Transaction {
  id: string;
  date: string;
  dueDate?: string;
  settled?: boolean;
  description: string;
  num?: string;
  notes?: string;
  planId?: string;
  currency: Currency;
  fxRateAtTransaction?: number;
  splits: Split[];
}

export interface DraftPostingLine {
  id?: string;
  accountId: string;
  amount: number;
  memo?: string;
  action?: string;
}

export function journalEntryToTransaction(entry: JournalEntryView): Transaction {
  return {
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
      action: p.action ?? undefined,
      reconcile: (p.reconcile as ReconcileState) ?? 'n',
    })),
  };
}
