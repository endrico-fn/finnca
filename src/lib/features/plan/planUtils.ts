import type { JournalEntryView } from '$lib/core/ipc/bindings';
import type { PaymentPlan } from './state/plan.svelte';

export function isPlanPostedOnDate(
  plan: PaymentPlan,
  date: string,
  entries: JournalEntryView[]
): boolean {
  return entries.some(
    (e) =>
      e.date === date &&
      (e.description.toLowerCase().includes(plan.title.toLowerCase()) ||
        e.postings.some((p) => Math.abs(p.amount) === plan.installmentAmount))
  );
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
