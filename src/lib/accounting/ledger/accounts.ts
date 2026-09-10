import type { Account, VaultData } from '../types';

export function buildChildrenMap(accounts: Account[]): Map<string, string[]> {
  const m = new Map<string, string[]>();
  for (const a of accounts) {
    if (!a.parentId) continue;
    const arr = m.get(a.parentId) ?? [];
    arr.push(a.id);
    m.set(a.parentId, arr);
  }
  return m;
}

export function descendantIds(accountId: string, childrenMap: Map<string, string[]>): Set<string> {
  const out = new Set<string>([accountId]);
  const stack = [accountId];
  while (stack.length) {
    const cur = stack.pop()!;
    const kids = childrenMap.get(cur) ?? [];
    for (const k of kids) {
      if (!out.has(k)) {
        out.add(k);
        stack.push(k);
      }
    }
  }
  return out;
}

export function accountBalanceMinor(
  accountId: string,
  vault: VaultData,
  childrenMap?: Map<string, string[]>
): number {
  const cmap = childrenMap ?? buildChildrenMap(vault.accounts);
  const ids = descendantIds(accountId, cmap);
  let sum = 0;
  for (const tx of vault.transactions) {
    for (const sp of tx.splits) if (ids.has(sp.accountId)) sum += sp.amount;
  }
  return sum;
}

export function accountBalanceMinorFiltered(
  accountId: string,
  vault: VaultData,
  from?: string,
  to?: string,
  cmap?: Map<string, string[]>
): number {
  const cmapActual = cmap ?? buildChildrenMap(vault.accounts);
  const ids = descendantIds(accountId, cmapActual);
  const inRange = (d: string) => (!from || d >= from) && (!to || d <= to);
  let sum = 0;
  for (const tx of vault.transactions) {
    if (!inRange(tx.date)) continue;
    for (const sp of tx.splits) if (ids.has(sp.accountId)) sum += sp.amount;
  }
  return sum;
}

export function getAccountPath(acc: Account, byId: Map<string, Account>): string {
  const parts: string[] = [acc.name];
  let cur: Account | undefined = acc;
  while (cur?.parentId && byId.has(cur.parentId)) {
    const parent: Account = byId.get(cur.parentId)!;
    parts.unshift(parent.name);
    cur = parent;
  }
  const rootType =
    acc.type === 'ASSET'
      ? 'Assets'
      : acc.type === 'LIABILITY'
        ? 'Liabilities'
        : acc.type === 'EQUITY'
          ? 'Equity'
          : acc.type === 'INCOME'
            ? 'Income'
            : 'Expenses';
  if (parts.length > 0 && parts[0].toLowerCase() !== rootType.toLowerCase()) {
    parts.unshift(rootType);
  }
  return parts.join(' > ');
}

export function getAccountCleanPath(accId: string, byId: Map<string, Account>): string {
  const acc = byId.get(accId);
  if (!acc) return '';
  return getAccountPath(acc, byId);
}
