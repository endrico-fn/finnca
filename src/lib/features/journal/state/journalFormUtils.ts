import type { Transaction, Split, Currency } from '$lib/core/types';
import type { Account } from '$lib/core/ipc/bindings';
import {
  fromMinor,
  parseStringAmountToMinor,
  formatMinorGrouping,
} from '$lib/core/format/currency';
import { todayString } from '$lib/core/format/date';
import { i18n } from '$lib/core/i18n.svelte';

export function createEmptyTransaction(firstAcc = '', secondAcc = ''): Transaction {
  return {
    id: crypto.randomUUID(),
    date: todayString(),
    dueDate: '',
    settled: false,
    description: '',
    num: '',
    notes: '',
    currency: 'IDR',
    splits: [
      { id: crypto.randomUUID(), accountId: firstAcc, amount: 0, reconcile: 'n' },
      { id: crypto.randomUUID(), accountId: secondAcc, amount: 0, reconcile: 'n' },
    ],
  };
}

export function ensureDraftFields(tx: Transaction): Transaction {
  tx.num ??= '';
  tx.notes ??= '';
  tx.dueDate ??= '';
  tx.settled ??= false;
  return tx;
}

export function computeImbalance(splits: Split[]): number {
  return splits.reduce((acc, s) => acc + s.amount, 0);
}

export function computeTotals(splits: Split[]): { debitTotal: number; creditTotal: number } {
  const debitTotal = splits.filter((s) => s.amount > 0).reduce((a, c) => a + c.amount, 0);
  const creditTotal = -splits.filter((s) => s.amount < 0).reduce((a, c) => a + c.amount, 0);
  return { debitTotal, creditTotal };
}

export function convertSimpleToSplits(
  standardFrom: string,
  standardTo: string,
  amountMinor: number,
  existingSplits: Split[]
): Split[] {
  return [
    {
      id: existingSplits[0]?.id ?? crypto.randomUUID(),
      accountId: standardTo,
      amount: amountMinor,
      reconcile: existingSplits[0]?.reconcile ?? 'n',
    },
    {
      id: existingSplits[1]?.id ?? crypto.randomUUID(),
      accountId: standardFrom,
      amount: -amountMinor,
      reconcile: existingSplits[1]?.reconcile ?? 'n',
    },
  ];
}

export function buildSimpleSplitsWithFee(
  standardFrom: string,
  standardTo: string,
  amountMinor: number,
  feeMinor: number,
  feeAccountId: string,
  existingSplits: Split[]
): Split[] {
  if (feeMinor > 0 && feeAccountId) {
    return [
      {
        id: existingSplits[0]?.id ?? crypto.randomUUID(),
        accountId: standardTo,
        amount: amountMinor,
        reconcile: existingSplits[0]?.reconcile ?? 'n',
      },
      {
        id: crypto.randomUUID(),
        accountId: feeAccountId,
        amount: feeMinor,
        reconcile: 'n',
      },
      {
        id: existingSplits[1]?.id ?? crypto.randomUUID(),
        accountId: standardFrom,
        amount: -(amountMinor + feeMinor),
        reconcile: existingSplits[1]?.reconcile ?? 'n',
      },
    ];
  }
  return convertSimpleToSplits(standardFrom, standardTo, amountMinor, existingSplits);
}

export function extractSimpleFromSplits(
  splits: Split[],
  currency: Currency
): { standardFrom: string; standardTo: string; standardAmount: string } | null {
  if (splits.length !== 2) return null;
  const debitSplit = splits.find((s) => s.amount >= 0);
  const creditSplit = splits.find((s) => s.amount < 0);
  if (debitSplit && creditSplit && debitSplit.amount !== 0) {
    return {
      standardTo: debitSplit.accountId,
      standardFrom: creditSplit.accountId,
      standardAmount: String(fromMinor(currency, Math.abs(debitSplit.amount))),
    };
  }
  return {
    standardTo: splits[0]?.accountId ?? '',
    standardFrom: splits[1]?.accountId ?? '',
    standardAmount: '',
  };
}

export function autoBalanceSplits(splits: Split[], imbalance: number): Split[] {
  if (imbalance === 0) return splits;
  const emptySplit = splits.find((s) => s.amount === 0);
  if (emptySplit) {
    return splits.map((s) => (s.id === emptySplit.id ? { ...s, amount: -imbalance } : s));
  }
  return [
    ...splits,
    {
      id: crypto.randomUUID(),
      accountId: '',
      amount: -imbalance,
      reconcile: 'n',
    },
  ];
}

export function updateSplitAmount(
  splits: Split[],
  splitId: string,
  field: 'debit' | 'credit',
  val: string,
  currency: Currency
): Split[] {
  const minor = parseStringAmountToMinor(val, currency);
  const amount = field === 'debit' ? minor : -minor;
  return splits.map((s) => {
    if (s.id === splitId) {
      return { ...s, amount };
    }
    if (splits.length === 2) {
      return { ...s, amount: -amount };
    }
    return s;
  });
}

export function resolvePresetAccounts(
  preset: { expenseKeyword: string; assetKeyword?: string },
  leafAccounts: Account[]
): { standardTo?: string; standardFrom?: string } {
  const to = leafAccounts.find(
    (a) =>
      a.account_type === 'EXPENSE' &&
      a.name.toLowerCase().includes(preset.expenseKeyword.toLowerCase())
  );
  const assetKw = preset.assetKeyword ?? 'bca';
  const from = leafAccounts.find(
    (a) =>
      a.account_type === 'ASSET' &&
      (a.name.toLowerCase().includes(assetKw.toLowerCase()) ||
        a.name.toLowerCase().includes('cash') ||
        a.name.toLowerCase().includes('kas'))
  );
  return { standardTo: to?.id, standardFrom: from?.id };
}

export interface DraftValidationResult {
  valid: boolean;
  error?: string;
}

export function validateDraft(
  draft: Transaction,
  expanded: boolean,
  standardFrom: string,
  standardTo: string,
  standardAmount: string,
  adminFee = '',
  adminFeeAccount = '',
  accountsById?: Map<string, Account>
): DraftValidationResult {
  if (!draft.description.trim()) {
    return { valid: false, error: i18n.t.txDescRequired };
  }
  if (!expanded) {
    if (!standardFrom || !standardTo) {
      return { valid: false, error: i18n.t.selectSourceTarget };
    }
    if (standardFrom === standardTo) {
      return { valid: false, error: i18n.t.sourceTargetSame };
    }
    const minor = parseStringAmountToMinor(standardAmount, draft.currency);
    if (minor <= 0) {
      return { valid: false, error: i18n.t.txAmountPositive };
    }
    const feeMinor = parseStringAmountToMinor(adminFee || '', draft.currency);
    if (adminFeeAccount && feeMinor <= 0) {
      return { valid: false, error: i18n.t.adminFeeInvalid };
    }
    if (!adminFeeAccount && feeMinor > 0) {
      return { valid: false, error: i18n.t.adminFeeAccountRequired };
    }
    if (feeMinor > 0 && adminFeeAccount) {
      if (adminFeeAccount === standardFrom || adminFeeAccount === standardTo) {
        return { valid: false, error: i18n.t.sourceTargetSame };
      }
      const feeAcc = accountsById?.get(adminFeeAccount);
      if (feeAcc && feeAcc.account_type !== 'EXPENSE') {
        return { valid: false, error: i18n.t.adminFeeAccountRequired };
      }
    }
    if (accountsById) {
      const fromAcc = accountsById.get(standardFrom);
      const toAcc = accountsById.get(standardTo);
      const feeAcc = adminFeeAccount ? accountsById.get(adminFeeAccount) : undefined;
      const curs = new Set(
        [fromAcc?.currency, toAcc?.currency, ...(feeAcc ? [feeAcc.currency] : [])].filter(Boolean)
      );
      if (curs.size > 1 || (fromAcc && fromAcc.currency !== draft.currency)) {
        return {
          valid: false,
          error: i18n.t.simpleCurrencyMismatch
            .replace('{from}', fromAcc?.name ?? '')
            .replace('{fromCur}', fromAcc?.currency ?? '')
            .replace('{to}', toAcc?.name ?? '')
            .replace('{toCur}', toAcc?.currency ?? ''),
        };
      }
    }
    return { valid: true };
  }
  for (const s of draft.splits) {
    if (!s.accountId) {
      return { valid: false, error: i18n.t.allSplitsNeedAccount };
    }
  }
  const imbalance = computeImbalance(draft.splits);
  if (imbalance !== 0) {
    return {
      valid: false,
      error: i18n.t.unbalancedTx
        .replace('{amount}', formatMinorGrouping(Math.abs(imbalance), draft.currency))
        .replace('{currency}', draft.currency),
    };
  }
  return { valid: true };
}
