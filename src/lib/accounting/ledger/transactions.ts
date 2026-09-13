import { DEFAULT_FX_RATE } from '../types';
import type { Split, Transaction, Account, VaultData } from '../types';
import { uid } from '../core/math';
import { todayString } from '../core/date';
import { buildChildrenMap, descendantIds } from './accounts';

export function transactionImbalance(tx: Transaction): number {
  return tx.splits.reduce((s, sp) => s + sp.amount, 0);
}

export function isBalanced(tx: Transaction): boolean {
  return transactionImbalance(tx) === 0 && tx.splits.length >= 2;
}

export function ledgerForAccount(
  accountId: string,
  vault: VaultData,
  cmap?: Map<string, string[]>
): { tx: Transaction; split: Split; running: number }[] {
  const cmapActual = cmap ?? buildChildrenMap(vault.accounts);
  const ids = descendantIds(accountId, cmapActual);
  const entries: { tx: Transaction; split: Split }[] = [];
  for (const tx of vault.transactions) {
    for (const sp of tx.splits) if (ids.has(sp.accountId)) entries.push({ tx, split: sp });
  }
  entries.sort((a, b) => a.tx.date.localeCompare(b.tx.date) || a.tx.id.localeCompare(b.tx.id));
  let running = 0;
  return entries.map((e) => {
    running += e.split.amount;
    return { ...e, running };
  });
}

export function validateTransaction(
  tx: Transaction,
  accountsById: Map<string, Account>
): string | null {
  if (!tx.description.trim()) return 'Description required';
  if (tx.splits.length < 2) return 'At least 2 splits required (double-entry)';
  const firstCurrency = tx.currency;
  for (const sp of tx.splits) {
    if (Math.abs(sp.amount) < 1) return 'Split amount cannot be zero';
    if (!accountsById.has(sp.accountId)) return `Account not found: ${sp.accountId}`;
    const acc = accountsById.get(sp.accountId)!;
    if (acc.placeholder) return `Placeholder account cannot be used: ${acc.name}`;
    if (acc.currency !== firstCurrency)
      return `All splits must match transaction currency ${firstCurrency} (account ${acc.name} is ${acc.currency})`;
  }
  const imb = transactionImbalance(tx);
  if (Math.abs(imb) >= 1) return `Unbalanced transaction: imbalance ${imb} minor units`;
  return null;
}

export function migrateLegacy(legacy: unknown, fxRate = DEFAULT_FX_RATE): VaultData {
  const obj = legacy as Record<string, unknown> | null | undefined;
  if (obj && obj.version === 2 && Array.isArray(obj.accounts)) {
    if (!obj.fxRate) obj.fxRate = fxRate;
    if (!obj.updatedAt) obj.updatedAt = new Date().toISOString();
    if (!obj.dashboardPrefs) obj.dashboardPrefs = {};
    return obj as unknown as VaultData;
  }
  const accounts = seedAccounts();
  const txs: Transaction[] = [];
  if (Array.isArray(legacy)) {
    for (const r of legacy as Record<string, unknown>[]) {
      const amt = Math.round(Number(r.nominal) || 0);
      if (!amt) continue;
      txs.push({
        id: uid(),
        date: todayString(),
        description: String(r.deskripsi ?? 'Migrated'),
        currency: 'IDR',
        splits: [
          { id: uid(), accountId: 'asset-cash', amount: amt, reconcile: 'n' },
          { id: uid(), accountId: 'eq-open', amount: -amt, reconcile: 'n' },
        ],
      });
    }
  }
  return {
    version: 2,
    accounts,
    transactions: txs,
    fxRate,
    updatedAt: new Date().toISOString(),
  };
}

function seedAccounts(): Account[] {
  const now = new Date().toISOString();
  const mk = (o: Omit<Account, 'createdAt'>): Account => ({
    ...o,
    createdAt: now,
  });
  const assets: Account = mk({
    id: 'asset',
    code: '1000',
    name: 'Assets',
    type: 'ASSET',
    parentId: null,
    currency: 'IDR',
    placeholder: true,
    hidden: false,
  });
  const cash: Account = mk({
    id: 'asset-cash',
    code: '1010',
    name: 'Cash & Bank',
    type: 'ASSET',
    parentId: 'asset',
    currency: 'IDR',
    placeholder: false,
    hidden: false,
  });
  const recv: Account = mk({
    id: 'asset-recv',
    code: '1020',
    name: 'Receivables',
    type: 'ASSET',
    parentId: 'asset',
    currency: 'IDR',
    placeholder: false,
    hidden: false,
  });
  const liab: Account = mk({
    id: 'liab',
    code: '2000',
    name: 'Liabilities',
    type: 'LIABILITY',
    parentId: null,
    currency: 'IDR',
    placeholder: true,
    hidden: false,
  });
  const payable: Account = mk({
    id: 'liab-pay',
    code: '2010',
    name: 'Payables',
    type: 'LIABILITY',
    parentId: 'liab',
    currency: 'IDR',
    placeholder: false,
    hidden: false,
  });
  const eq: Account = mk({
    id: 'eq',
    code: '3000',
    name: 'Equity',
    type: 'EQUITY',
    parentId: null,
    currency: 'IDR',
    placeholder: true,
    hidden: false,
  });
  const opening: Account = mk({
    id: 'eq-open',
    code: '3010',
    name: 'Opening Balance',
    type: 'EQUITY',
    parentId: 'eq',
    currency: 'IDR',
    placeholder: false,
    hidden: false,
  });
  const inc: Account = mk({
    id: 'inc',
    code: '4000',
    name: 'Income',
    type: 'INCOME',
    parentId: null,
    currency: 'IDR',
    placeholder: true,
    hidden: false,
  });
  const sales: Account = mk({
    id: 'inc-sales',
    code: '4010',
    name: 'Sales',
    type: 'INCOME',
    parentId: 'inc',
    currency: 'IDR',
    placeholder: false,
    hidden: false,
  });
  const exp: Account = mk({
    id: 'exp',
    code: '5000',
    name: 'Expenses',
    type: 'EXPENSE',
    parentId: null,
    currency: 'IDR',
    placeholder: true,
    hidden: false,
  });
  const ops: Account = mk({
    id: 'exp-ops',
    code: '5010',
    name: 'Operational',
    type: 'EXPENSE',
    parentId: 'exp',
    currency: 'IDR',
    placeholder: false,
    hidden: false,
  });
  return [assets, cash, recv, liab, payable, eq, opening, inc, sales, exp, ops];
}
