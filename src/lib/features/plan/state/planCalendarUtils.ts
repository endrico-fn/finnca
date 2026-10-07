import { i18n } from '$lib/core/i18n.svelte';
import { getPref } from '$lib/core/state/prefs';
import { getCurrencyFactor } from '$lib/core/format/currency';
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

export function formatCompact(minor: number, currency: 'IDR' | 'USD' | string = 'IDR'): string {
  const n = Math.abs(minor);
  const factor = getCurrencyFactor(currency);
  const major = n / factor;
  const locale = getPref('finnca_num_format', 'comma') === 'dot' ? 'id-ID' : 'en-US';
  const fmt = (v: number, decimals: number) =>
    v.toLocaleString(locale, { maximumFractionDigits: decimals, minimumFractionDigits: 0 });
  if (major >= 1_000_000_000) {
    return `${fmt(major / 1_000_000_000, 1)}${i18n.t.compactBillion}`;
  }
  if (major >= 1_000_000) {
    return `${fmt(major / 1_000_000, 1)}${i18n.t.compactMillion}`;
  }
  if (major >= 1_000) {
    return `${fmt(major / 1_000, 0)}${i18n.t.compactThousand}`;
  }
  return String(major);
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
