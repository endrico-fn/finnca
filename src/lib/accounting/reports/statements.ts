import type { Account, VaultData } from '../types';
import { convertMinor } from '../core/math';
import {
  accountBalanceMinor,
  accountBalanceMinorFiltered,
  buildChildrenMap,
} from '../ledger/accounts';
import { todayString } from '../core/date';
import { generateGhostTransactions } from '../features/planning';
import { calculateFxRevaluation } from '../features/fx';

export function getHistoricalFx(vault: VaultData, date?: string): number {
  if (!date || !vault.fxHistory || vault.fxHistory.length === 0) return vault.fxRate;
  const history = [...vault.fxHistory].sort((a, b) => b.date.localeCompare(a.date));
  const entry = history.find((e) => e.date <= date);
  return entry ? entry.rate : vault.fxRate;
}

export function buildAsOfVault(vault: VaultData, asOf?: string): VaultData {
  if (!asOf) return vault;
  let ghostTxs: import('../types').Transaction[] = [];
  const today = todayString();
  if (asOf > today) {
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    const tomorrowStr = todayString(tomorrow);
    ghostTxs = generateGhostTransactions(vault, tomorrowStr, asOf);
  }
  return {
    ...vault,
    transactions: [...vault.transactions.filter((t) => t.date <= asOf), ...ghostTxs],
  };
}

export function trialBalance(
  vault: VaultData,
  cmap?: Map<string, string[]>,
  asOf?: string
): { account: Account; debit: number; credit: number }[] {
  const cmapActual = cmap ?? buildChildrenMap(vault.accounts);
  const fx = getHistoricalFx(vault, asOf);
  const filteredVault = buildAsOfVault(vault, asOf);

  return vault.accounts
    .filter((a) => !a.placeholder && !cmapActual.has(a.id))
    .map((a) => {
      const raw = asOf
        ? accountBalanceMinorFiltered(a.id, filteredVault, undefined, asOf, cmapActual)
        : accountBalanceMinor(a.id, filteredVault, cmapActual);
      const bal = a.currency === 'USD' ? convertMinor(raw, 'USD', 'IDR', fx) : raw;
      const debit = bal > 0 ? bal : 0;
      const credit = bal < 0 ? Math.abs(bal) : 0;
      return { account: a, debit, credit };
    })
    .filter((r) => r.debit !== 0 || r.credit !== 0);
}

export function totalDebitsCredits(rows: { debit: number; credit: number }[]): {
  debit: number;
  credit: number;
} {
  return rows.reduce((acc, r) => ({ debit: acc.debit + r.debit, credit: acc.credit + r.credit }), {
    debit: 0,
    credit: 0,
  });
}

export function incomeStatement(
  vault: VaultData,
  from?: string,
  to?: string
): { income: number; expense: number; net: number } {
  let income = 0,
    expense = 0;
  const inRange = (d: string) => (!from || d >= from) && (!to || d <= to);
  const byId = new Map(vault.accounts.map((a) => [a.id, a]));
  for (const tx of vault.transactions) {
    if (!inRange(tx.date)) continue;
    for (const sp of tx.splits) {
      const acc = byId.get(sp.accountId);
      if (!acc) continue;
      const amt =
        acc.currency === 'USD'
          ? convertMinor(sp.amount, 'USD', 'IDR', tx.fxRateAtTransaction || vault.fxRate)
          : sp.amount;
      if (acc.type === 'INCOME') income += -amt;
      if (acc.type === 'EXPENSE') expense += amt;
    }
  }
  return { income, expense, net: income - expense };
}

export function balanceSheet(
  vault: VaultData,
  asOf?: string,
  cmap?: Map<string, string[]>
): {
  assets: number;
  liabilities: number;
  equity: number;
  netIncome: number;
  unrealizedFx: number;
  discrepancy: number;
  balanced: boolean;
} {
  const cmapActual = cmap ?? buildChildrenMap(vault.accounts);
  const filteredVault = buildAsOfVault(vault, asOf);
  const fx = getHistoricalFx(vault, asOf);

  let assets = 0,
    liabilities = 0,
    equity = 0;

  const imbalances = new Map<string, number>();
  const isLeaf = (id: string) => !cmapActual.has(id);

  for (const a of vault.accounts) {
    if (!isLeaf(a.id) || a.placeholder) continue;
    const raw = accountBalanceMinor(a.id, filteredVault, cmapActual);
    const bal = a.currency === 'USD' ? convertMinor(raw, 'USD', 'IDR', fx) : raw;

    if (a.type === 'ASSET') assets += bal;
    else if (a.type === 'LIABILITY') liabilities += -bal;
    else if (a.type === 'EQUITY') equity += -bal;

    const current = imbalances.get(a.currency) || 0;
    imbalances.set(a.currency, current + raw);
  }

  const { net } = incomeStatement(filteredVault);
  const netIdr = net;
  const { totalUnrealizedGain } = calculateFxRevaluation({ ...filteredVault, fxRate: fx });
  const discrepancy = assets - liabilities - equity - netIdr - totalUnrealizedGain;
  const balanced = Array.from(imbalances.values()).every((v) => v === 0) && discrepancy === 0;

  return {
    assets,
    liabilities,
    equity,
    netIncome: netIdr,
    unrealizedFx: totalUnrealizedGain,
    discrepancy,
    balanced,
  };
}
