import type { Transaction } from './journalDraft.svelte';

export function filterTransactions(
  transactions: Transaction[],
  q: string,
  from: string,
  to: string,
  accFilter: string
): Transaction[] {
  const query = q.trim().toLowerCase();
  return transactions
    .filter((t) => {
      if (
        query &&
        !`${t.description} ${t.num ?? ''} ${t.notes ?? ''}`.toLowerCase().includes(query)
      ) {
        return false;
      }
      if (from && t.date < from) return false;
      if (to && t.date > to) return false;
      if (accFilter && !t.splits.some((s) => s.accountId === accFilter)) return false;
      return true;
    })
    .sort((a, b) => b.date.localeCompare(a.date));
}

export function filterBaseTransactions(
  transactions: Transaction[],
  q: string,
  from: string,
  to: string
): Transaction[] {
  const query = q.trim().toLowerCase();
  return transactions.filter((t) => {
    if (
      query &&
      !`${t.description} ${t.num ?? ''} ${t.notes ?? ''}`.toLowerCase().includes(query)
    ) {
      return false;
    }
    if (from && t.date < from) return false;
    if (to && t.date > to) return false;
    return true;
  });
}

export function computeAccountTxCounts(baseTxs: Transaction[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const t of baseTxs) {
    const seen: Record<string, true> = {};
    for (const s of t.splits) {
      if (seen[s.accountId]) continue;
      seen[s.accountId] = true;
      counts[s.accountId] = (counts[s.accountId] ?? 0) + 1;
    }
  }
  return counts;
}

export interface JournalTotals {
  idrDebit: number;
  idrCredit: number;
  usdDebit: number;
  usdCredit: number;
}

export function computeJournalTotals(filtered: Transaction[]): JournalTotals {
  let idrDebit = 0;
  let idrCredit = 0;
  let usdDebit = 0;
  let usdCredit = 0;

  for (const t of filtered) {
    if (t.currency === 'IDR') {
      for (const s of t.splits) {
        if (s.amount > 0) idrDebit += s.amount;
        else if (s.amount < 0) idrCredit -= s.amount;
      }
    } else if (t.currency === 'USD') {
      for (const s of t.splits) {
        if (s.amount > 0) usdDebit += s.amount;
        else if (s.amount < 0) usdCredit -= s.amount;
      }
    }
  }

  return { idrDebit, idrCredit, usdDebit, usdCredit };
}
