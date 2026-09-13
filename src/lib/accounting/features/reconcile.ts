import type { VaultData } from '../types';

export function reconciledBalanceMinor(accountId: string, vault: VaultData): number {
  let sum = 0;
  for (const tx of vault.transactions) {
    for (const sp of tx.splits) {
      if (sp.accountId === accountId && sp.reconcile === 'y') {
        sum = Math.round(sum + Math.round(sp.amount));
      }
    }
  }
  return sum;
}

export function clearedBalanceMinor(
  accountId: string,
  vault: VaultData,
  clearedMap: Record<string, boolean>
): number {
  let sum = reconciledBalanceMinor(accountId, vault);
  for (const tx of vault.transactions) {
    for (const sp of tx.splits) {
      if (sp.accountId === accountId && sp.reconcile !== 'y' && clearedMap[sp.id]) {
        sum = Math.round(sum + Math.round(sp.amount));
      }
    }
  }
  return sum;
}
