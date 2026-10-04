import type {
  CreateJournalEntryInput,
  PostingInput,
  JournalEntryView,
  Account,
} from '$lib/core/ipc/bindings';
import type { Currency } from '$lib/core/types';
import {
  fromMinor,
  parseStringAmountToMinor,
  formatMinorGrouping,
} from '$lib/core/format/currency';
import { todayString } from '$lib/core/format/date';
import { checkTransferLegs } from './transferValidation';
export {
  checkTransferLegs,
  checkTransferAmounts,
  type TransferAmountBlock,
  type TransferLegsBlock,
} from './transferValidation';
import { i18n } from '$lib/core/i18n.svelte';
import { parseNoteTags } from './journalNoteTags';

export function createTwoLegPostings(fromAcc = '', toAcc = '', amount = 0): PostingInput[] {
  return [
    { id: crypto.randomUUID(), account_id: toAcc, amount, reconcile: 'n' },
    { id: crypto.randomUUID(), account_id: fromAcc, amount: -amount, reconcile: 'n' },
  ];
}

export function createEmptyJournalDraft(
  fromAcc = '',
  toAcc = '',
  currency: Currency = 'IDR'
): CreateJournalEntryInput {
  return {
    id: crypto.randomUUID(),
    date: todayString(),
    description: '',
    notes: '',
    reference_no: null,
    due_date: null,
    currency,
    fx_rate: null,
    postings: [
      { id: crypto.randomUUID(), account_id: toAcc, amount: 0, reconcile: 'n' },
      { id: crypto.randomUUID(), account_id: fromAcc, amount: 0, reconcile: 'n' },
    ],
  };
}

export function createTransferDraft(params: {
  fromId?: string;
  toId?: string;
  amount?: number;
  description?: string;
  notes?: string;
  currency?: Currency;
  date?: string;
}): CreateJournalEntryInput {
  const amt = params.amount ?? 0;
  return {
    id: crypto.randomUUID(),
    date: params.date ?? todayString(),
    description: params.description ?? '',
    notes: params.notes ?? '',
    reference_no: null,
    due_date: null,
    currency: params.currency ?? 'IDR',
    fx_rate: null,
    postings: [
      {
        id: crypto.randomUUID(),
        account_id: params.toId ?? '',
        amount: amt,
        reconcile: 'n',
      },
      {
        id: crypto.randomUUID(),
        account_id: params.fromId ?? '',
        amount: -amt,
        reconcile: 'n',
      },
    ],
  };
}

export function cloneJournalEntry(
  entry: JournalEntryView,
  options?: { copySuffix?: boolean; resetDate?: boolean }
): CreateJournalEntryInput {
  const copySuffix = options?.copySuffix ?? true;
  const resetDate = options?.resetDate ?? true;
  return {
    id: crypto.randomUUID(),
    date: resetDate ? todayString() : entry.date,
    description: copySuffix ? `${entry.description} (Copy)` : entry.description,
    reference_no: null,
    due_date: null,
    currency: entry.currency,
    fx_rate: entry.fx_rate,
    notes: entry.notes ?? '',
    postings: entry.postings.map((p) => ({
      id: crypto.randomUUID(),
      account_id: p.account_id,
      amount: p.amount,
      memo: p.memo,
      reconcile: 'n',
    })),
  };
}

export function createReconciledStatementDraft(params: {
  date: string;
  description: string;
  amount: number;
  currency: Currency;
  bankAccountId: string;
  offsetAccountId: string;
  notes?: string;
}): CreateJournalEntryInput {
  const isIncome = params.amount > 0;
  return {
    id: crypto.randomUUID(),
    date: params.date,
    description: params.description,
    notes: params.notes ?? '[From Bank Statement]',
    reference_no: null,
    due_date: null,
    currency: params.currency,
    fx_rate: null,
    postings: isIncome
      ? [
          {
            id: crypto.randomUUID(),
            account_id: params.bankAccountId,
            amount: params.amount,
            memo: params.description || null,
            reconcile: 'c',
          },
          {
            id: crypto.randomUUID(),
            account_id: params.offsetAccountId,
            amount: -params.amount,
            memo: null,
            reconcile: 'n',
          },
        ]
      : [
          {
            id: crypto.randomUUID(),
            account_id: params.offsetAccountId,
            amount: -params.amount,
            memo: null,
            reconcile: 'n',
          },
          {
            id: crypto.randomUUID(),
            account_id: params.bankAccountId,
            amount: params.amount,
            memo: params.description || null,
            reconcile: 'c',
          },
        ],
  };
}

export type InspectorViewMode = 'transfer' | 'journal';
export type SimpleCategory = 'TRANSFER' | 'EXPENSE' | 'INCOME' | 'CUSTOM';

export interface InspectorInitialFormState {
  mode: InspectorViewMode;
  simpleCategory: SimpleCategory;
  standardFrom: string;
  standardTo: string;
  standardAmount: string;
  adminFee: string;
  adminFeeAccount: string;
  description: string;
  date: string;
  num: string;
  dueDate: string;
  settled: boolean;
  currency: Currency;
  fxRate: number;
  notes: string;
  postings: PostingInput[];
}

export function initializeInspectorForm(params: {
  entry?: JournalEntryView | null;
  initialDraft?: CreateJournalEntryInput | null;
  initialMode?: InspectorViewMode;
  initialFrom?: string;
  initialTo?: string;
  defaultFxRate: number;
  leafAccounts: Account[];
}): InspectorInitialFormState {
  const {
    entry,
    initialDraft,
    initialMode = 'transfer',
    initialFrom = '',
    initialTo = '',
    defaultFxRate,
    leafAccounts,
  } = params;

  let mode: InspectorViewMode;
  let simpleCategory: SimpleCategory = 'TRANSFER';
  let standardFrom = initialFrom;
  let standardTo = initialTo;
  let standardAmount = '';
  const adminFee = '';
  const adminFeeAccount = '';
  let description: string;
  let date: string;
  let num: string;
  let dueDate: string;
  let settled = false;
  let currency: Currency;
  let fxRate: number;
  let notes: string;
  let postings: PostingInput[];

  if (entry) {
    const tags = parseNoteTags(entry);
    num = tags.ref;
    settled = tags.settled;
    dueDate = tags.dueDate;
    notes = tags.cleanNotes || '';
    description = entry.description;
    date = entry.date;
    currency = (entry.currency as Currency) || 'IDR';
    fxRate = entry.fx_rate ?? defaultFxRate;
    postings = entry.postings.map((p) => ({
      id: p.id,
      account_id: p.account_id,
      amount: p.amount,
      memo: p.memo,
      reconcile: p.reconcile,
    }));

    if (postings.length > 2) {
      mode = 'journal';
    } else {
      mode = 'transfer';
      const simple = extractSimpleFromSplits(postings, currency);
      if (simple) {
        standardTo = simple.standardTo;
        standardFrom = simple.standardFrom;
        standardAmount = simple.standardAmount;
      }
    }
  } else if (initialDraft) {
    description = initialDraft.description || '';
    date = initialDraft.date || todayString();
    num = initialDraft.reference_no || '';
    dueDate = initialDraft.due_date || '';
    notes = initialDraft.notes || '';
    currency = (initialDraft.currency as Currency) || 'IDR';
    fxRate = initialDraft.fx_rate ?? defaultFxRate;
    postings =
      initialDraft.postings && initialDraft.postings.length >= 2
        ? initialDraft.postings.map((p) => ({
            ...p,
            id: p.id || crypto.randomUUID(),
          }))
        : createEmptyJournalDraft('', '', currency).postings;

    if (postings.length > 2) {
      mode = 'journal';
    } else {
      mode = initialMode;
      const simple = extractSimpleFromSplits(postings, currency);
      if (simple && (simple.standardTo || simple.standardFrom)) {
        standardTo = simple.standardTo;
        standardFrom = simple.standardFrom;
        standardAmount = simple.standardAmount;
      }
      if (!standardFrom || !standardTo) {
        const def = resolveDefaultAccounts(
          leafAccounts,
          standardFrom || standardTo || initialFrom || initialTo,
          standardFrom,
          standardTo
        );
        standardFrom = def.standardFrom;
        standardTo = def.standardTo;
      }
    }
  } else {
    mode = initialMode;
    date = todayString();
    description = '';
    num = '';
    dueDate = '';
    settled = false;
    currency = 'IDR';
    fxRate = defaultFxRate;
    notes = '';
    standardAmount = '';
    standardFrom = initialFrom;
    standardTo = initialTo;
    if (!standardFrom || !standardTo) {
      const def = resolveDefaultAccounts(
        leafAccounts,
        initialFrom || initialTo,
        standardFrom,
        standardTo
      );
      standardFrom = def.standardFrom;
      standardTo = def.standardTo;
    }
    const toAccId = standardTo || leafAccounts[1]?.id || '';
    const fromAccId = standardFrom || leafAccounts[0]?.id || '';
    postings = [
      { id: crypto.randomUUID(), account_id: toAccId, amount: 0, reconcile: 'n' },
      { id: crypto.randomUUID(), account_id: fromAccId, amount: 0, reconcile: 'n' },
    ];
  }

  const fromAcc = leafAccounts.find((a) => a.id === standardFrom);
  const toAcc = leafAccounts.find((a) => a.id === standardTo);
  if (fromAcc && toAcc) {
    if (fromAcc.account_type === 'INCOME' && toAcc.account_type === 'ASSET') {
      simpleCategory = 'INCOME';
    } else if (fromAcc.account_type === 'ASSET' && toAcc.account_type === 'EXPENSE') {
      simpleCategory = 'EXPENSE';
    } else if (
      (fromAcc.account_type === 'ASSET' || fromAcc.account_type === 'LIABILITY') &&
      (toAcc.account_type === 'ASSET' || toAcc.account_type === 'LIABILITY')
    ) {
      simpleCategory = 'TRANSFER';
    } else {
      simpleCategory = 'CUSTOM';
    }
  }

  return {
    mode,
    simpleCategory,
    standardFrom,
    standardTo,
    standardAmount,
    adminFee,
    adminFeeAccount,
    description,
    date,
    num,
    dueDate,
    settled,
    currency,
    fxRate,
    notes,
    postings,
  };
}

export function computeImbalance(postings: PostingInput[]): number {
  return postings.reduce((acc, p) => acc + (Number(p.amount) || 0), 0);
}

export function resolveDefaultAccounts(
  leaves: Account[],
  preferredId: string | undefined,
  currentFrom: string,
  currentTo: string
): { standardFrom: string; standardTo: string } {
  let standardFrom = currentFrom;
  let standardTo = currentTo;
  if (leaves.length < 2) return { standardFrom, standardTo };

  const assetFallback = (excludeId?: string) =>
    leaves.find((a) => a.account_type === 'ASSET' && a.id !== excludeId) ?? leaves[0];

  if (preferredId) {
    const pref = leaves.find((a) => a.id === preferredId);
    if (pref) {
      if (pref.account_type === 'EXPENSE') {
        standardTo = pref.id;
        standardFrom = assetFallback(pref.id)?.id ?? '';
        return { standardFrom, standardTo };
      } else if (pref.account_type === 'INCOME') {
        standardFrom = pref.id;
        standardTo = assetFallback(pref.id)?.id ?? '';
        return { standardFrom, standardTo };
      } else if (pref.account_type === 'ASSET') {
        standardFrom = pref.id;
        const expAcc =
          leaves.find((a) => a.account_type === 'EXPENSE' && a.id !== pref.id) ?? leaves[1];
        standardTo = expAcc?.id ?? '';
        return { standardFrom, standardTo };
      } else if (pref.account_type === 'LIABILITY') {
        standardFrom = pref.id;
        const expAcc =
          leaves.find((a) => a.account_type === 'EXPENSE' && a.id !== pref.id) ??
          leaves.find((a) => a.id !== pref.id);
        standardTo = expAcc?.id ?? '';
        return { standardFrom, standardTo };
      } else if (pref.account_type === 'EQUITY') {
        standardFrom = pref.id;
        standardTo = assetFallback(pref.id)?.id ?? '';
        return { standardFrom, standardTo };
      } else {
        standardTo = pref.id;
        standardFrom = assetFallback(pref.id)?.id ?? '';
        return { standardFrom, standardTo };
      }
    }
  }

  if (!standardFrom) {
    standardFrom = assetFallback()?.id ?? '';
  }
  if (!standardTo || standardTo === standardFrom) {
    const second =
      leaves.find((a) => a.id !== standardFrom && a.account_type === 'EXPENSE') ??
      leaves.find((a) => a.id !== standardFrom);
    if (second) standardTo = second.id;
  }
  return { standardFrom, standardTo };
}

export function computeTotals(postings: PostingInput[]): {
  debitTotal: number;
  creditTotal: number;
} {
  const debitTotal = postings.filter((p) => p.amount > 0).reduce((a, c) => a + c.amount, 0);
  const creditTotal = -postings.filter((p) => p.amount < 0).reduce((a, c) => a + c.amount, 0);
  return { debitTotal, creditTotal };
}

export function convertSimpleToSplits(
  standardFrom: string,
  standardTo: string,
  amountMinor: number,
  existingPostings: PostingInput[]
): PostingInput[] {
  return [
    {
      id: existingPostings[0]?.id ?? crypto.randomUUID(),
      account_id: standardTo,
      amount: amountMinor,
      reconcile: existingPostings[0]?.reconcile ?? 'n',
    },
    {
      id: existingPostings[1]?.id ?? crypto.randomUUID(),
      account_id: standardFrom,
      amount: -amountMinor,
      reconcile: existingPostings[1]?.reconcile ?? 'n',
    },
  ];
}

export function buildSimpleSplitsWithFee(
  standardFrom: string,
  standardTo: string,
  amountMinor: number,
  feeMinor: number,
  feeAccountId: string,
  existingPostings: PostingInput[]
): PostingInput[] {
  if (feeMinor > 0 && feeAccountId) {
    return [
      {
        id: existingPostings[0]?.id ?? crypto.randomUUID(),
        account_id: standardTo,
        amount: amountMinor,
        reconcile: existingPostings[0]?.reconcile ?? 'n',
      },
      {
        id: crypto.randomUUID(),
        account_id: feeAccountId,
        amount: feeMinor,
        reconcile: 'n',
      },
      {
        id: existingPostings[1]?.id ?? crypto.randomUUID(),
        account_id: standardFrom,
        amount: -(amountMinor + feeMinor),
        reconcile: existingPostings[1]?.reconcile ?? 'n',
      },
    ];
  }
  return convertSimpleToSplits(standardFrom, standardTo, amountMinor, existingPostings);
}

export function extractSimpleFromSplits(
  postings: PostingInput[],
  currency = 'IDR'
): { standardFrom: string; standardTo: string; standardAmount: string } | null {
  if (postings.length !== 2) return null;
  const debitPost = postings.find((p) => p.amount > 0);
  const creditPost = postings.find((p) => p.amount < 0);
  if (debitPost && creditPost) {
    return {
      standardTo: debitPost.account_id,
      standardFrom: creditPost.account_id,
      standardAmount: String(fromMinor(currency, Math.abs(debitPost.amount))),
    };
  }
  return {
    standardTo: postings[0]?.account_id ?? '',
    standardFrom: postings[1]?.account_id ?? '',
    standardAmount: '',
  };
}

export function autoBalanceSplits(postings: PostingInput[], imbalance: number): PostingInput[] {
  if (imbalance === 0) return postings;
  const emptyPost = postings.find((p) => p.amount === 0);
  if (emptyPost) {
    return postings.map((p) => (p.id === emptyPost.id ? { ...p, amount: -imbalance } : p));
  }
  return [
    ...postings,
    {
      id: crypto.randomUUID(),
      account_id: '',
      amount: -imbalance,
      reconcile: 'n',
    },
  ];
}

export function updateSplitAmount(
  postings: PostingInput[],
  postingId: string,
  field: 'debit' | 'credit',
  val: string,
  currency = 'IDR'
): PostingInput[] {
  const minor = parseStringAmountToMinor(val, currency);
  const amount = field === 'debit' ? minor : -minor;
  return postings.map((p) => {
    if (p.id === postingId) {
      return { ...p, amount };
    }
    if (postings.length === 2) {
      return { ...p, amount: -amount };
    }
    return p;
  });
}

export interface PresetInput {
  desc: string;
  expenseKeywords: string[];
  expenseCodes?: string[];
  assetKeywords?: string[];
  equityCodes?: string[];
  equityKeywords?: string[];
  fromAccountId?: string;
  toAccountId?: string;
  amountMajor?: string;
  currency?: string;
}

export interface RecentPreset {
  desc: string;
  fromAccountId?: string;
  toAccountId?: string;
  amountMinor: number;
  currency: string;
  count: number;
  lastDate: string;
}

export function buildRecentPresets(entries: JournalEntryView[], limit = 6): RecentPreset[] {
  const byDesc = new Map<string, RecentPreset>();
  const ordered = [...entries].sort((a, b) => (a.date < b.date ? 1 : -1));
  for (const t of ordered) {
    const key = t.description.trim().toLowerCase();
    if (!key) continue;
    const hit = byDesc.get(key);
    if (hit) {
      hit.count += 1;
      continue;
    }
    const toPosting = t.postings.find((p) => p.amount > 0);
    const fromPosting = t.postings.find((p) => p.amount < 0);
    byDesc.set(key, {
      desc: t.description.trim(),
      fromAccountId: fromPosting?.account_id,
      toAccountId: toPosting?.account_id,
      amountMinor: toPosting ? toPosting.amount : 0,
      currency: t.currency,
      count: 1,
      lastDate: t.date,
    });
  }
  return [...byDesc.values()]
    .sort((a, b) => b.count - a.count || (a.lastDate < b.lastDate ? 1 : -1))
    .slice(0, limit);
}

export function resolvePresetAccounts(
  preset: {
    expenseKeyword?: string;
    expenseKeywords?: string[];
    expenseCodes?: string[];
    assetKeyword?: string;
    assetKeywords?: string[];
    equityCodes?: string[];
    equityKeywords?: string[];
  },
  leafAccounts: Account[]
): { standardTo?: string; standardFrom?: string } {
  const expenseKws =
    preset.expenseKeywords ?? (preset.expenseKeyword ? [preset.expenseKeyword] : []);
  const to = leafAccounts.find(
    (a) =>
      a.account_type === 'EXPENSE' &&
      ((preset.expenseCodes && preset.expenseCodes.some((c) => a.code.startsWith(c))) ||
        expenseKws.some((kw) => a.name.toLowerCase().includes(kw.toLowerCase())))
  );
  const assetKws =
    preset.assetKeywords ??
    (preset.assetKeyword ? [preset.assetKeyword] : ['bca', 'bank', 'cash', 'kas']);
  const from =
    leafAccounts.find(
      (a) =>
        a.account_type === 'ASSET' &&
        assetKws.some((kw) => a.name.toLowerCase().includes(kw.toLowerCase()))
    ) ?? leafAccounts.find((a) => a.account_type === 'ASSET');
  if (preset.equityCodes?.length || preset.equityKeywords?.length) {
    const eqKws = preset.equityKeywords ?? [];
    const eq = leafAccounts.find(
      (a) =>
        a.account_type === 'EQUITY' &&
        ((preset.equityCodes && preset.equityCodes.some((c) => a.code.startsWith(c))) ||
          eqKws.some((kw) => a.name.toLowerCase().includes(kw.toLowerCase())))
    );
    if (eq) return { standardTo: from?.id, standardFrom: eq.id };
  }
  return { standardTo: to?.id, standardFrom: from?.id };
}

export interface DraftValidationResult {
  valid: boolean;
  error?: string;
}

export function validateDraft(
  draft: CreateJournalEntryInput,
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
  const cur = draft.currency || 'IDR';
  if (!expanded) {
    const minor = parseStringAmountToMinor(standardAmount, cur);
    const feeMinor = parseStringAmountToMinor(adminFee || '', cur);
    const legsBlock = checkTransferLegs(standardFrom, standardTo, minor, adminFeeAccount, feeMinor);
    if (legsBlock) {
      const message =
        legsBlock === 'MISSING_SIDE'
          ? i18n.t.selectSourceTarget
          : legsBlock === 'SAME_SIDE' || legsBlock === 'FEE_IS_SIDE'
            ? i18n.t.sourceTargetSame
            : legsBlock === 'BAD_AMOUNT'
              ? i18n.t.txAmountPositive
              : legsBlock === 'FEE_BAD_AMOUNT'
                ? i18n.t.adminFeeInvalid
                : i18n.t.adminFeeAccountRequired;
      return { valid: false, error: message };
    }
    if (feeMinor > 0 && adminFeeAccount) {
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
      if (curs.size > 1 || (fromAcc && fromAcc.currency !== cur)) {
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
  for (const p of draft.postings) {
    if (!p.account_id) {
      return { valid: false, error: i18n.t.allSplitsNeedAccount };
    }
  }
  const imbalance = computeImbalance(draft.postings);
  if (imbalance !== 0) {
    return {
      valid: false,
      error: i18n.t.unbalancedTx
        .replace('{amount}', formatMinorGrouping(Math.abs(imbalance), cur))
        .replace('{currency}', cur),
    };
  }
  return { valid: true };
}
