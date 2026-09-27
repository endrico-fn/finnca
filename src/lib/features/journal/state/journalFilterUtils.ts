import type { JournalEntryView } from '$lib/core/ipc/bindings';

export type JournalStatusFilter = 'all' | 'due' | 'overdue';

export function isEntrySettled(t: JournalEntryView): boolean {
  return (t.notes?.includes('[Settled]') || t.due_date?.includes('[Settled]')) ?? false;
}

export function getCleanDueDate(t: JournalEntryView): string | null {
  const raw = t.due_date ?? t.notes?.match(/\[Due:\s*([^\]]+)\]/)?.[1];
  if (!raw) return null;
  const clean = raw.replace(/\s*\[Settled\]/i, '').trim();
  return clean || null;
}

export function filterTransactions(
  transactions: JournalEntryView[],
  q: string,
  from: string,
  to: string,
  accFilter: string,
  statusFilter: JournalStatusFilter = 'all'
): JournalEntryView[] {
  const query = q.trim().toLowerCase();
  const today = new Date().toISOString().slice(0, 10);
  return transactions
    .filter((t) => {
      if (
        query &&
        !`${t.description} ${t.notes ?? ''} ${t.reference_no ?? ''}`.toLowerCase().includes(query)
      ) {
        return false;
      }
      if (from && t.date < from) return false;
      if (to && t.date > to) return false;
      if (accFilter && !t.postings.some((p) => p.account_id === accFilter)) return false;

      if (statusFilter === 'due') {
        const due = getCleanDueDate(t);
        if (!due || isEntrySettled(t)) return false;
      } else if (statusFilter === 'overdue') {
        const due = getCleanDueDate(t);
        if (!due || isEntrySettled(t) || due >= today) return false;
      }

      return true;
    })
    .sort((a, b) => b.date.localeCompare(a.date));
}

export function filterBaseTransactions(
  transactions: JournalEntryView[],
  q: string,
  from: string,
  to: string,
  statusFilter: JournalStatusFilter = 'all'
): JournalEntryView[] {
  const query = q.trim().toLowerCase();
  const today = new Date().toISOString().slice(0, 10);
  return transactions.filter((t) => {
    if (
      query &&
      !`${t.description} ${t.notes ?? ''} ${t.reference_no ?? ''}`.toLowerCase().includes(query)
    ) {
      return false;
    }
    if (from && t.date < from) return false;
    if (to && t.date > to) return false;

    if (statusFilter === 'due') {
      const due = getCleanDueDate(t);
      if (!due || isEntrySettled(t)) return false;
    } else if (statusFilter === 'overdue') {
      const due = getCleanDueDate(t);
      if (!due || isEntrySettled(t) || due >= today) return false;
    }

    return true;
  });
}

export function computeAccountTxCounts(baseTxs: JournalEntryView[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const t of baseTxs) {
    const seen: Record<string, true> = {};
    for (const p of t.postings) {
      if (seen[p.account_id]) continue;
      seen[p.account_id] = true;
      counts[p.account_id] = (counts[p.account_id] ?? 0) + 1;
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

export function computeJournalTotals(filtered: JournalEntryView[]): JournalTotals {
  let idrDebit = 0;
  let idrCredit = 0;
  let usdDebit = 0;
  let usdCredit = 0;

  for (const t of filtered) {
    if (t.currency === 'IDR') {
      for (const p of t.postings) {
        if (p.amount > 0) idrDebit += p.amount;
        else if (p.amount < 0) idrCredit -= p.amount;
      }
    } else if (t.currency === 'USD') {
      for (const p of t.postings) {
        if (p.amount > 0) usdDebit += p.amount;
        else if (p.amount < 0) usdCredit -= p.amount;
      }
    }
  }

  return { idrDebit, idrCredit, usdDebit, usdCredit };
}
