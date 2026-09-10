import type { VaultData } from '../types';

export function reconciledBalanceMinor(accountId: string, vault: VaultData): number {
  return vault.transactions.reduce((sum, tx) => {
    const s = tx.splits.find((sp) => sp.accountId === accountId && sp.reconcile === 'y');
    return sum + (s ? s.amount : 0);
  }, 0);
}

export function clearedBalanceMinor(
  accountId: string,
  vault: VaultData,
  clearedMap: Record<string, boolean>
): number {
  const starting = reconciledBalanceMinor(accountId, vault);
  return vault.transactions.reduce((sum, tx) => {
    const s = tx.splits.find((sp) => sp.accountId === accountId && clearedMap[sp.id]);
    return sum + (s ? s.amount : 0);
  }, starting);
}
