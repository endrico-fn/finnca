import type { Transaction, Split, ReconcileState } from '$lib/core/types';
import type { Account, AccountType } from '$lib/core/ipc/bindings';

export interface LedgerEntry {
  tx: Transaction;
  split: Split;
  running: number;
}

export interface PeriodStats {
  debit: number;
  credit: number;
  net: number;
  cleared: number;
  count: number;
}

export function isDebitNormal(t: AccountType): boolean {
  return t === 'ASSET' || t === 'EXPENSE';
}

export function checkAbnormalBalance(account: Account | null, balance: number): boolean {
  if (!account || account.placeholder || balance === 0) return false;
  return isDebitNormal(account.account_type) ? balance < 0 : balance > 0;
}

export function buildAccountEntries(
  account: Account | null,
  transactions: Transaction[]
): LedgerEntry[] {
  if (!account || account.placeholder) return [];
  const entries: LedgerEntry[] = [];
  const sortedTxs = [...transactions].sort(
    (a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id)
  );
  let running = 0;
  for (const tx of sortedTxs) {
    for (const sp of tx.splits) {
      if (sp.accountId === account.id) {
        running += sp.amount;
        entries.push({ tx, split: sp, running });
      }
    }
  }
  return entries;
}

export function filterLedgerEntries(
  entries: LedgerEntry[],
  q: string,
  from: string,
  to: string,
  reconcileFilter: 'ALL' | ReconcileState
): LedgerEntry[] {
  let out = entries;
  if (q.trim()) {
    const qq = q.toLowerCase().trim();
    out = out.filter((e) =>
      `${e.tx.description} ${e.tx.num ?? ''} ${e.split.memo ?? ''} ${e.tx.notes ?? ''}`
        .toLowerCase()
        .includes(qq)
    );
  }
  if (from) out = out.filter((e) => e.tx.date >= from);
  if (to) out = out.filter((e) => e.tx.date <= to);
  if (reconcileFilter !== 'ALL') out = out.filter((e) => e.split.reconcile === reconcileFilter);
  return out;
}

export function computeLedgerStats(filtered: LedgerEntry[]): PeriodStats {
  let debit = 0;
  let credit = 0;
  let cleared = 0;
  for (const e of filtered) {
    if (e.split.amount > 0) debit += e.split.amount;
    else credit += -e.split.amount;
    if (e.split.reconcile === 'y') cleared++;
  }
  return {
    debit,
    credit,
    net: debit - credit,
    cleared,
    count: filtered.length,
  };
}

export function computeStatusCounts(
  entries: LedgerEntry[],
  q: string,
  from: string,
  to: string
): Record<'ALL' | ReconcileState, number> {
  let base = entries;
  if (q.trim()) {
    const qq = q.toLowerCase().trim();
    base = base.filter((e) =>
      `${e.tx.description} ${e.tx.num ?? ''} ${e.split.memo ?? ''} ${e.tx.notes ?? ''}`
        .toLowerCase()
        .includes(qq)
    );
  }
  if (from) base = base.filter((e) => e.tx.date >= from);
  if (to) base = base.filter((e) => e.tx.date <= to);

  const counts: Record<'ALL' | ReconcileState, number> = {
    ALL: base.length,
    n: 0,
    c: 0,
    y: 0,
  };
  for (const e of base) counts[e.split.reconcile] += 1;
  return counts;
}
