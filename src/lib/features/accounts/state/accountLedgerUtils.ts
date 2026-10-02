import type {
  Account,
  AccountType,
  AccountRunningLedgerItem,
  ReconcileStatus,
} from '$lib/core/ipc/bindings';

export type LedgerEntry = AccountRunningLedgerItem;

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

export function filterLedgerEntries(
  entries: AccountRunningLedgerItem[],
  q: string,
  from: string,
  to: string,
  reconcileFilter: 'ALL' | ReconcileStatus
): AccountRunningLedgerItem[] {
  let out = entries;
  if (q.trim()) {
    const qq = q.toLowerCase().trim();
    out = out.filter((e) =>
      `${e.description} ${e.memo ?? ''} ${e.offset_account ?? ''}`.toLowerCase().includes(qq)
    );
  }
  if (from) out = out.filter((e) => e.date >= from);
  if (to) out = out.filter((e) => e.date <= to);
  if (reconcileFilter !== 'ALL') out = out.filter((e) => e.reconcile === reconcileFilter);
  return out;
}

export function computeLedgerStats(filtered: AccountRunningLedgerItem[]): PeriodStats {
  let debit = 0;
  let credit = 0;
  let cleared = 0;
  for (const e of filtered) {
    if (e.amount > 0) debit += e.amount;
    else credit += -e.amount;
    if (e.reconcile === 'y') cleared++;
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
  entries: AccountRunningLedgerItem[],
  q: string,
  from: string,
  to: string
): Record<'ALL' | ReconcileStatus, number> {
  let base = entries;
  if (q.trim()) {
    const qq = q.toLowerCase().trim();
    base = base.filter((e) =>
      `${e.description} ${e.memo ?? ''} ${e.offset_account ?? ''}`.toLowerCase().includes(qq)
    );
  }
  if (from) base = base.filter((e) => e.date >= from);
  if (to) base = base.filter((e) => e.date <= to);

  const counts: Record<'ALL' | ReconcileStatus, number> = {
    ALL: base.length,
    n: 0,
    c: 0,
    y: 0,
  };
  for (const e of base) {
    if (e.reconcile in counts) {
      counts[e.reconcile] += 1;
    }
  }
  return counts;
}
