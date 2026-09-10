import type { VaultData } from '../types';
import { convertMinor } from '../core/math';
import { DEFAULT_FX_RATE } from '../types';

export interface EnvelopeData {
  accountId: string;
  assigned: number;
  activity: number;
  available: number;
}

export function calculateBudgetMonth(
  vault: VaultData,
  monthPrefix: string
): {
  envelopes: EnvelopeData[];
  totalAssigned: number;
  totalActivity: number;
  toBeBudgeted: number;
} {
  const byId = new Map(vault.accounts.map((a) => [a.id, a]));
  const expenseAccounts = vault.accounts.filter((a) => a.type === 'EXPENSE' && !a.placeholder);
  const envelopes: EnvelopeData[] = [];

  let totalIncomeAllTime = 0;
  
  const activityAllTimeMap = new Map<string, number>();
  const activityThisMonthMap = new Map<string, number>();

  // Single pass aggregation for both Income and Expense
  for (const tx of vault.transactions) {
    const txMonth = tx.date.slice(0, 7);
    const isThisMonth = txMonth === monthPrefix;
    const isPastOrThisMonth = txMonth <= monthPrefix;
    const fx = tx.fxRateAtTransaction || vault.fxRate || DEFAULT_FX_RATE;

    for (const s of tx.splits) {
      const acc = byId.get(s.accountId);
      if (!acc) continue;

      const amt = acc.currency === 'USD' ? convertMinor(s.amount, 'USD', 'IDR', fx) : s.amount;

      if (acc.type === 'INCOME') {
        if (isPastOrThisMonth) {
          totalIncomeAllTime += (amt > 0 ? amt : -amt);
        }
      } else if (acc.type === 'EXPENSE') {
        if (isPastOrThisMonth) activityAllTimeMap.set(acc.id, (activityAllTimeMap.get(acc.id) || 0) + amt);
        if (isThisMonth) activityThisMonthMap.set(acc.id, (activityThisMonthMap.get(acc.id) || 0) + amt);
      }
    }
  }

  const assignedAllTimeMap = new Map<string, number>();
  const assignedThisMonthMap = new Map<string, number>();
  for (const b of vault.budgets || []) {
    if (b.month <= monthPrefix) {
      assignedAllTimeMap.set(b.accountId, (assignedAllTimeMap.get(b.accountId) || 0) + b.amount);
    }
    if (b.month === monthPrefix) {
      assignedThisMonthMap.set(b.accountId, (assignedThisMonthMap.get(b.accountId) || 0) + b.amount);
    }
  }

  let totalAssignedAllTime = 0;
  let totalAssignedThisMonth = 0;
  let totalActivityThisMonth = 0;

  for (const acc of expenseAccounts) {
    const assignedAllTime = assignedAllTimeMap.get(acc.id) || 0;
    const assignedThisMonth = assignedThisMonthMap.get(acc.id) || 0;
    const activityAllTime = activityAllTimeMap.get(acc.id) || 0;
    const activityThisMonth = activityThisMonthMap.get(acc.id) || 0;

    totalAssignedAllTime += assignedAllTime;
    totalAssignedThisMonth += assignedThisMonth;
    totalActivityThisMonth += activityThisMonth;

    const available = assignedAllTime - activityAllTime;

    envelopes.push({
      accountId: acc.id,
      assigned: assignedThisMonth,
      activity: activityThisMonth,
      available,
    });
  }

  const toBeBudgeted = totalIncomeAllTime - totalAssignedAllTime;

  return {
    envelopes,
    totalAssigned: totalAssignedThisMonth,
    totalActivity: totalActivityThisMonth,
    toBeBudgeted,
  };
}
