import type { PaymentPlan, VaultData } from '../types';
import { todayString, matchesMonthlyDay } from '../core/date';
import { i18n } from '$lib/i18n.svelte';

export interface PlanProgress {
  plan: PaymentPlan;
  paidAmount: number;
  remainingAmount: number;
  progressPercent: number;
  isSettled: boolean;
  installmentsPaidCount: number;
}

export function calculatePlanProgress(plan: PaymentPlan, vault: VaultData): PlanProgress {
  const planTitleLower = plan.title.toLowerCase().trim();
  let paidAmount = 0;
  let count = 0;

  for (const t of vault.transactions) {
    const matchById = t.planId === plan.id;
    let isMatch = matchById;

    if (!t.planId && !isMatch) {
      const matchDesc = t.description.toLowerCase().includes(planTitleLower);
      const matchNotes = t.notes ? t.notes.toLowerCase().includes(planTitleLower) : false;
      isMatch = matchDesc || matchNotes;
    }

    if (isMatch) {
      const matchAccounts =
        matchById ||
        t.splits.some(
          (s) => s.accountId === plan.toAccountId || s.accountId === plan.fromAccountId
        );
      if (matchAccounts) {
        const relevantSplits = t.splits.filter(
          (s) => s.accountId === plan.toAccountId || s.accountId === plan.fromAccountId
        );
        const amt = relevantSplits
          .filter((s) => s.amount > 0)
          .reduce((sum, sp) => sum + sp.amount, 0);
        if (amt > 0) {
          paidAmount += amt;
          count++;
        }
      }
    }
  }

  const total = plan.totalAmount > 0 ? plan.totalAmount : 1;
  const progressPercent = Math.min(100, Math.trunc((paidAmount / total) * 100));
  const remainingAmount = Math.max(0, plan.totalAmount - paidAmount);
  const isSettled = paidAmount >= plan.totalAmount && plan.totalAmount > 0;

  return {
    plan,
    paidAmount,
    remainingAmount,
    progressPercent,
    isSettled,
    installmentsPaidCount: count,
  };
}

export function determinePlanStatus(
  plan: PaymentPlan,
  vault: VaultData
): import('../types').PlanStatus {
  if (plan.status === 'ARCHIVED') return 'ARCHIVED';

  const prog = calculatePlanProgress(plan, vault);
  if (prog.isSettled) return 'COMPLETED';

  if (plan.dueDate) {
    const today = todayString();
    if (plan.dueDate < today) return 'OVERDUE';
  }

  return 'ACTIVE';
}

export function suggestInstallmentOptions(
  totalAmount: number,
  interestRateAnnual: number = 0
): { count: number; amount: number; frequency: string; totalInterest: number }[] {
  const options: { count: number; amount: number; frequency: string; totalInterest: number }[] = [];

  const monthlyOptions = [3, 6, 12, 24];
  for (const count of monthlyOptions) {
    if (interestRateAnnual > 0) {
      const r = interestRateAnnual / 100 / 12;
      const r_plus_1_pow_n = Math.pow(1 + r, count);
      const amount = Math.floor((totalAmount * (r * r_plus_1_pow_n)) / (r_plus_1_pow_n - 1));
      const totalInterest = amount * count - totalAmount;
      if (amount > 0) options.push({ count, amount, frequency: 'MONTHLY', totalInterest });
    } else {
      const amount = Math.ceil(totalAmount / count);
      if (amount > 0) options.push({ count, amount, frequency: 'MONTHLY', totalInterest: 0 });
    }
  }

  return options;
}

export function generateGhostTransactions(
  vault: import('../types').VaultData,
  fromDate: string,
  toDate: string
): import('../types').Transaction[] {
  const ghosts: import('../types').Transaction[] = [];
  if (!vault.plans) return ghosts;

  const activePlans = vault.plans.filter(
    (p) => determinePlanStatus(p, vault) === 'ACTIVE' || determinePlanStatus(p, vault) === 'OVERDUE'
  );

  if (activePlans.length === 0 || fromDate > toDate) return ghosts;

  const [fY, fM, fD] = fromDate.split('-').map(Number);
  const [tY, tM, tD] = toDate.split('-').map(Number);

  const start = new Date(fY, fM - 1, fD);
  const end = new Date(tY, tM - 1, tD);

  const current = new Date(start);
  const generateUUID = () => Math.random().toString(36).substring(2, 15);

  while (current <= end) {
    const y = current.getFullYear();
    const m = String(current.getMonth() + 1).padStart(2, '0');
    const d = String(current.getDate()).padStart(2, '0');
    const dateStr = `${y}-${m}-${d}`;
    const dayNum = current.getDate();
    const localDay = current.getDay(); // 0 = Sun, 1 = Mon...

    for (const p of activePlans) {
      if (p.startDate && dateStr < p.startDate) continue;

      let match = false;
      if (p.dueDate === dateStr) {
        match = true;
      } else if (!p.dueDate || dateStr <= p.dueDate) {
        if (p.frequency === 'DAILY') match = true;
        else if (p.frequency === 'WEEKLY') {
          const [sY, sM, sD] = (p.startDate || fromDate).split('-').map(Number);
          const startLocalDay = new Date(sY, sM - 1, sD).getDay();
          if (startLocalDay === localDay) match = true;
        } else if (
          p.frequency === 'MONTHLY' &&
          p.dayOfMonth != null &&
          matchesMonthlyDay(y, Number(m), p.dayOfMonth, dayNum)
        ) {
          match = true;
        }
      }

      if (match) {
        ghosts.push({
          id: `ghost-${p.id}-${dateStr}`,
          date: dateStr,
          description: `${i18n.t.forecastBadge} ${p.title}`,
          currency: 'IDR',
          notes: i18n.t.ghostTxNotes,
          splits: [
            {
              id: generateUUID(),
              accountId: p.fromAccountId,
              amount: -p.installmentAmount,
              reconcile: 'n',
            },
            {
              id: generateUUID(),
              accountId: p.toAccountId,
              amount: p.installmentAmount,
              reconcile: 'n',
            },
          ],
          planId: p.id,
        });
      }
    }

    current.setDate(current.getDate() + 1);
  }

  return ghosts;
}
