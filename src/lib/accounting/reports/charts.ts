import type { Account, Transaction, VaultData } from '../types';
import { convertMinor } from '../core/math';
import { buildChildrenMap } from '../ledger/accounts';
import { getHistoricalFx } from './statements';

export interface DailyDataPoint {
  date: string;
  netWorth: number;
  assets: number;
  liabilities: number;
  liquidCash: number;
}

export function historicalDailyBalances(
  vault: VaultData,
  startDate: string,
  endDate: string
): DailyDataPoint[] {
  const cmap = buildChildrenMap(vault.accounts);
  const isLeaf = (id: string) => !cmap.has(id);
  const leafAccounts = vault.accounts.filter((a) => isLeaf(a.id) && !a.placeholder);
  const leafMap = new Map(leafAccounts.map((a) => [a.id, a]));

  const isLiquidCash = (acc: Account) =>
    acc.code.startsWith('101') ||
    acc.code.startsWith('1020') ||
    acc.name.toLowerCase().includes('cash') ||
    acc.name.toLowerCase().includes('kas') ||
    acc.name.toLowerCase().includes('bank');

  let assetsIdr = 0, assetsUsd = 0;
  let liabilitiesIdr = 0, liabilitiesUsd = 0;
  let liquidCashIdr = 0, liquidCashUsd = 0;

  const applySplit = (sp: import('../types').Split) => {
    const acc = leafMap.get(sp.accountId);
    if (!acc) return;
    if (acc.type === 'ASSET') {
      if (acc.currency === 'USD') assetsUsd += sp.amount;
      else assetsIdr += sp.amount;
      if (isLiquidCash(acc)) {
        if (acc.currency === 'USD') liquidCashUsd += sp.amount;
        else liquidCashIdr += sp.amount;
      }
    } else if (acc.type === 'LIABILITY') {
      if (acc.currency === 'USD') liabilitiesUsd -= sp.amount;
      else liabilitiesIdr -= sp.amount;
    }
  };

  const txsByDate = new Map<string, Transaction[]>();
  for (const tx of vault.transactions) {
    if (tx.date < startDate) {
      for (const sp of tx.splits) applySplit(sp);
    } else if (tx.date <= endDate) {
      if (!txsByDate.has(tx.date)) txsByDate.set(tx.date, []);
      txsByDate.get(tx.date)!.push(tx);
    }
  }

  const points: DailyDataPoint[] = [];
  const parseLocalNoon = (s: string) => {
    const [y, m, d] = s.split('-').map(Number);
    return new Date(y, m - 1, d, 12, 0, 0);
  };
  const start = parseLocalNoon(startDate);
  const end = parseLocalNoon(endDate);

  if (isNaN(start.getTime()) || isNaN(end.getTime()) || start > end) {
    return [];
  }

  const cur = new Date(start);
  while (cur <= end) {
    const y = cur.getFullYear();
    const m = String(cur.getMonth() + 1).padStart(2, '0');
    const d = String(cur.getDate()).padStart(2, '0');
    const dStr = `${y}-${m}-${d}`;
    const dayTxs = txsByDate.get(dStr);
    
    if (dayTxs) {
      for (const tx of dayTxs) {
        for (const sp of tx.splits) applySplit(sp);
      }
    }

    const fx = getHistoricalFx(vault, dStr);
    const assets = assetsIdr + convertMinor(assetsUsd, 'USD', 'IDR', fx);
    const liabilities = liabilitiesIdr + convertMinor(liabilitiesUsd, 'USD', 'IDR', fx);
    const liquidCash = liquidCashIdr + convertMinor(liquidCashUsd, 'USD', 'IDR', fx);

    const netWorth = assets - liabilities;
    points.push({
      date: dStr,
      netWorth,
      assets,
      liabilities,
      liquidCash,
    });
    cur.setDate(cur.getDate() + 1);
  }

  return points;
}
