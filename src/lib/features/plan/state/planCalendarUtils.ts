import { i18n } from '$lib/core/i18n.svelte';
import type { PaymentPlan, DateEvents } from './plan.svelte';
import type { JournalEntryView } from '$lib/core/ipc/bindings';

export function getDayCashflow(evts: DateEvents | undefined): { in: number; out: number } {
  if (!evts || evts.plans.length === 0) return { in: 0, out: 0 };
  let inTotal = 0;
  let outTotal = 0;
  for (const p of evts.plans) {
    if (p.type === 'RECEIVABLE') inTotal += p.installmentAmount;
    else if (p.type === 'PAYABLE') outTotal += p.installmentAmount;
  }
  return { in: inTotal, out: outTotal };
}

export function getTxPositiveTotal(tx: JournalEntryView): number {
  return tx.postings.filter((s) => s.amount > 0).reduce((acc, sp) => acc + sp.amount, 0);
}

export function formatCompact(minor: number): string {
  const n = Math.abs(minor);
  if (n >= 1_000_000_000) {
    const val = (n / 1_000_000_000).toFixed(1).replace(/\.0$/, '');
    return `${val}${i18n.t.compactBillion}`;
  }
  if (n >= 1_000_000) {
    const val = (n / 1_000_000).toFixed(1).replace(/\.0$/, '');
    return `${val}${i18n.t.compactMillion}`;
  }
  if (n >= 1_000) {
    const val = (n / 1_000).toFixed(0);
    return `${val}${i18n.t.compactThousand}`;
  }
  return String(n);
}

export function computeDayCashflowTotals(plans: PaymentPlan[]): {
  totalIn: number;
  totalOut: number;
  net: number;
} {
  const totalIn = plans
    .filter((p) => p.type === 'RECEIVABLE')
    .reduce((sum, p) => sum + p.installmentAmount, 0);
  const totalOut = plans
    .filter((p) => p.type === 'PAYABLE')
    .reduce((sum, p) => sum + p.installmentAmount, 0);
  return { totalIn, totalOut, net: totalIn - totalOut };
}
